# Galileo 多 IP 并行执行方案（更新版）

> 面向当前仓库的盲发策略、multi-leg 引擎与落地体系设计的多 IP 扩展计划，确保满足“配置简单、零成本抽象、性能优先”三大原则。

---

## 1. 现状梳理

### 1.1 盲发（单腿）策略链路
- `BlindStrategy`（`src/strategy/blind_strategy.rs`）在每个 tick 里按顺序生成 `QuoteTask`。
- `StrategyEngine::handle_action`（`src/engine/mod.rs:257`）接到 `Action::Quote(Vec<QuoteTask>)` 后逐个 `await self.process_task(task)`；没有真正的并发。
- `TradeProfile`（`src/engine/types.rs`）只有 `amounts` 和 `process_delay`，无法描述买腿→卖腿的批次配对。

### 1.2 Multi-leg 架构
- `MultiLegOrchestrator`（`src/multi_leg/orchestrator.rs`）为每个腿 provider 提供 `plan` 接口。
- `MultiLegRuntime`（`src/multi_leg/runtime.rs`）会并行构建多条腿组合，并在 `plan_pair_batch_with_profit` 里调度 Titan / Ultra / DFlow / 其它腿。
- Titan WebSocket 在 `api::titan::TitanWsClient` 中实现，配有额外的 `ConcurrencyPolicy`（`runtime.rs:330`）控制并发。
- 当前 multi-leg 也复用全局 `reqwest::Client`，没有 IP 绑定能力。

### 1.3 HTTP / WS 客户端
- `api::dflow::DflowApiClient`、`api::ultra::UltraApiClient`、`lander::*` 等均以 `reqwest::Client` 实例发送请求。
- `reqwest::ClientBuilder` 支持 `local_address`，但目前统一用默认值（无绑定）。
- Titan WebSocket 通过 `tokio_tungstenite::connect_async_tls_with_config` 建立连接，没有指定本地 IP。

### 1.4 观测现状
- `monitoring::events` 仅记录 Quote/Swap/Lander 的数量、延迟；缺少 `local_ip`、429 次数等字段。
- rate limit/backoff 分布在各模块内部，没有统一的状态或指标。

### 1.5 配置现状
- `BotConfig`（`src/config/types.rs`）未包含网络项，所有网络行为受 `global.proxy`、`engine.*`、`lander.*` 控制。
- 多 IP feature 目标是只增加 `bot.network.enable_multiple_ip` 和可选 `bot.network.manual_ips`，保持配置简明。

---

## 2. 设计目标

1. **智能并发**：从 `blind_strategy.base_mints[*].trade_size_range` 推导买/卖腿批次，对每个批次根据 IP 数量自动规划并发度；确保买腿完成后同批 IP 立即切换卖腿。
2. **多策略复用**：盲发与 multi-leg 共用一套 `IpAllocator`，LegProvider / QuoteExecutor / SwapFetcher / Lander 在执行前统一申请 `IpLease`。Titan WebSocket 使用的 IP 允许其它 HTTP 请求复用。
3. **运行时自适应**：除 `enable_multiple_ip` 外不再增加配置项；当 IP 数 < 任务数时自动排队，IP 数 > 任务数时保持余量；multi-leg 的并发腿数也由 IP 池决定。
4. **限流友好**：命中 429 / timeout 时对具体 IP 做冷却（指数退避），其余 IP 不受影响；记录冷却原因和时长指标。
5. **落地一致性**：lander 层将 endpoint 与 IP 做一致性哈希绑定，避免 Jito / Staked 多 endpoint 相互干扰。
6. **观测完备**：所有网络调用带上 `local_ip`、`task_kind`、`result` 指标；Tracing 中追加 `local_ip` 字段。

---

## 3. 核心模块设计

### 3.1 `network` 基础设施（新增）
- 目录：`src/network/`
- 组件：
  - `IpInventory`: 启动时探测 IP（自动/手动列表），过滤 loopback、down 接口、重复 IP；守则要求仅依赖标准库 + `/sys/class/net`/`getifaddrs`。
  - `IpSlot`: 保存 `ip: IpAddr`、`state: AtomicIpState`（Idle/Busy/CoolingDown/LongLived）、`stats: IpStats`（滚动窗口）。
  - `IpBoundClientPool`: 为每个 IP 懒加载 `reqwest::Client`，复用当前 proxy / TLS 设置；支持 IPv4/IPv6。
  - `TitanSocketFactory`: 根据 `IpLease` 绑定本地 IP 创建 `tokio::net::TcpStream`。
- Prometheus 指标：`galileo_network_ip_inventory_total{ip}`。

### 3.2 `IpAllocator` 与 `IpLease`
- 外部接口：
  ```rust
  enum IpTaskKind {
      QuoteBuy,
      QuoteSell,
      SwapInstructions,
      LanderSubmit { endpoint_hash: u64 },
      MultiLegLeg { aggregator: AggregatorKind, side: LegSide },
      TitanWsConnect,
  }
  enum IpLeaseMode { Ephemeral, SharedLongLived }
  ```
- `acquire(kind)` 返回 `Option<IpLease>`；`IpLease` 附带 `mode`。
- `mark_result(lease_id, outcome)` 更新统计&退避：429 -> 冷却 500ms 起指数增长；timeout -> 250ms；其他错误按需处理。
- 对 `TitanWsConnect` 返回 `SharedLongLived`，意味着该 IP 同时可服务其它 `IpTaskKind`。

### 3.3 盲发调度改造
- 在 `StrategyContext` 增加 `plan_batches(ip_budget)`，输出结构：
  ```rust
  struct QuoteBatchPlan {
      batch_id: u64,
      mint: Pubkey,
      trade_size: u64,
      tasks: [QuoteTask; 2], // buy + sell
  }
  ```
- 新增 `engine::quote_dispatcher`：
  - 使用 `FuturesUnordered` 根据 `QuoteBatchPlan` 发起并行请求；
  - 买腿完成后复用同一批 IP 立即执行卖腿；
  - 额外 IP 轮询剩余批次（Round Robin）。
- `QuoteExecutor::round_trip` 接受 `IpLeaseHandle`，将其传递给 `DflowApiClient` / `JupiterApiClient`。

### 3.4 multi-leg 集成
- `MultiLegRuntime` 在 `plan_pair_batch_with_profit` 前查询 IP 预算，控制并发腿组合数量；对 Titan legs 与 HTTP legs 都使用 `IpAllocator`。
- `UltraLegProvider`、`DFlowLegProvider`、其它 HTTP legs：
  - 注入 `Arc<IpBoundClientPool>` 和 `Arc<IpAllocator>`;
  - 在调用 `client.order` / `client.quote` 前申请 `IpLease` 并选择对应 `reqwest::Client`。
- Titan：
  - `TitanWsClient::connect_with_local_ip(ip)` 使用 `tokio::net::TcpSocket::bind(ip)` -> `connect` -> `client_async_tls`；
  - 连接成功后将 `IpLease` 标记为长连接共享；断线或手动关闭时释放；
  - 仍保留 `ConcurrencyPolicy` 的 titan_stream_limit / titan_debounce，用于限制订阅节奏。
- 确保 Titan 所在 IP 可服务其它 HTTP 请求：`SharedLongLived` 模式不阻塞 `QuoteBuy` 等任务申请同一 IP。

### 3.5 Swap Fetcher 与 Lander
- `SwapInstructionFetcher`、`LanderFactory`、`JitoLander`、`StakedLander` 改为在执行前申请 `IpLease`。
- Lander 发请求时通过 `endpoint_hash` 选择 IP，成功后上报 `IpOutcome`。
- 落地指标：`galileo_ip_requests_total{ip,task="lander",endpoint, result}`。

### 3.6 统一观测
- 在 `monitoring::events` 添加：
  - `galileo_ip_requests_total{task,ip,result}`（Counter）
  - `galileo_ip_inflight{ip}`（Gauge）
  - `galileo_ip_cooldown_total{ip,reason}`（Counter）
  - `galileo_ip_latency_ms_bucket{task,ip}`（Histogram）
- Tracing span 扩展字段：`local_ip`、`ip_task_kind`。
- 文档 `docs/networking/multi_ip.md` 说明如何启用 Prometheus、如何查看仪表盘。

---

## 4. 需求→实现映射

| 需求 | 对应实现 |
| --- | --- |
| 配置简单，仅开关 + 手动列表 | `BotConfig.network` 只包含 `enable_multiple_ip` 与 `manual_ips` |
| 盲发买卖腿对称并发 | `QuoteBatchPlan` + `quote_dispatcher` |
| 多余 IP 轮询利用 | `quote_dispatcher` 中的 Round Robin 调度 |
| multi-leg 与盲发共用调度 | `IpAllocator`/`IpBoundClientPool` 注入到 LegProvider 与 Lander |
| Titan 使用不同 IP 建立 WS，但仍共享 | `TitanWsClient::connect_with_local_ip` + `SharedLongLived` 租约 |
| 避免 429 压力 | `IpAllocator::mark_result` 冷却 + metrics |
| 可观测 | 新增 Prometheus 指标 + tracing 字段 |

---

## 5. 实施阶段

### 阶段 1：配置与探测
1. 扩展 `BotConfig`，添加 `NetworkConfig`；更新 `galileo.yaml` 模板。
2. 实现 `IpInventory` + 自动枚举逻辑；单元测试覆盖手动列表/重复过滤。
3. 输出日志（info）列出可用 IP，便于排查。

### 阶段 2：调度核心（盲发）
1. 引入 `QuoteBatchPlan`、`quote_dispatcher`。
2. `StrategyEngine::handle_action` 改为并发执行；`QuoteExecutor` 接受 `IpLeaseHandle`。
3. `monitoring::events` 记录 quote 批次、IP 信息。

### 阶段 3：HTTP 客户端池
1. 实现 `IpBoundClientPool`，支持 proxy / TLS / UA 继承。
2. `DflowApiClient`、`UltraApiClient`、`JupiterApiClient`、`lander::*` 重构构造函数以接收 `Arc<IpBoundClientPool> + Arc<IpAllocator>`。
3. CLI `prepare_swap_components` 注入上述组件。

### 阶段 4：multi-leg & Titan
1. `MultiLegRuntime` 获取 `Arc<IpAllocator>`，根据 IP 数控制并发。
2. 各 `LegProvider` 接入 IP 池。
3. `TitanWsClient` 增加本地 IP 绑定；连接生命周期管理 `SharedLongLived` 租约。
4. 集成测试（mock server）验证 Titan WS 与 HTTP 同 IP 不冲突。

### 阶段 5：Lander 与退避
1. `SwapInstructionFetcher`、`LanderStack` 完成 IP 接入与一致性哈希。
2. 完善 `IpAllocator::mark_result` 冷却策略，添加指标。
3. Prometheus 文档、Grafana 面板模板。

### 阶段 6：验收
1. 单元测试：`IpAllocator` 退避、`QuoteBatchPlan` 配对、Titan socket 绑定。
2. 多 IP 本地压测脚本（mock DFlow / Lander）评估吞吐与 429。
3. 更新 `docs/strategy_arch.md`、`ROADMAP.md`、`titan_plus_dflow_design.md` 等文档引用。

---

## 6. 验证指标
- **性能**：Quote 吞吐、平均/分位延迟；Swap 指令耗时；落地成功率。
- **限流改善**：`rate_limited` counter；冷却次数与时长。
- **IP 利用率**：`galileo_ip_utilization_ratio`（可由 inflight / slots 计算）；Titan 长连占用情况。
- **稳定性**：长时间运行的错误率、冷却恢复成功率。

---

## 7. 风险与缓解
| 风险 | 描述 | 缓解措施 |
| --- | --- | --- |
| 某些网络环境禁止绑定特定 IP | `bind` 失败 | 提供 manual override + 黑名单；失败时回退默认 IP |
| 大量 IP 导致 `reqwest::Client` 过多 | 内存/握手开销 | 客户端懒加载，长时间未用释放；共享 TLS 池 |
| Titan 长连持有 IP 导致饥饿 | 其它任务无法获得 IP | 使用 `SharedLongLived` 模式，允许复用；定期检测空闲释放 |
| Quote 批次错乱 | 买卖腿未对应 | `QuoteBatchPlan` 携带 `batch_id`，卖腿前校验买腿已完成 |
| multi-leg 与盲发互相争抢 IP | 调度不均 | `IpAllocator` 支持任务优先级/权重；监控 IP 争用情况 |
| 观测缺口 | 难以排查 | 指标 + tracing 附带 `local_ip` / `task` / `outcome` |

---

## 8. 里程碑
1. **M1**：配置+探测+盲发并发原型；完成基本指标。
2. **M2**：HTTP 客户端池 + 退避 + multi-leg 集成。
3. **M3**：Titan WS 绑定 + lander 一致性哈希 + 压测 + 文档。

---

## 9. 参考实现提示
- `reqwest::ClientBuilder::local_address` 可直接绑定 `SocketAddr`；IPv6 支持需检测接口。
- Titan 可使用 `tokio::net::TcpSocket::new_v4()`，`bind(ip, 0)` 后 `connect`，再传入 `MaybeTlsStream`.
- `FuturesUnordered` + `tokio::sync::Semaphore` 控制批次并发，避免 task 爆炸。
- `StrategyContext`、`LegProvider` 等 trait 改动保持泛型接口，避免引入 `Box<dyn>`。

---

通过上述方案，多 IP 能力将在盲发、multi-leg、Titan、lander 等关键链路中统一发挥作用，同时保证配置简单、抽象零成本、性能观测健全。下一步可按照阶段 1 实现配置解析与 IP 自动探测。***
