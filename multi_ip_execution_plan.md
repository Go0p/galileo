# Galileo 多 IP 与 Quote 并发重构方案

> 本文在既有讨论的基础上，统一规划“单 IP 并发 Quote”“多 IP 资源池”以及 Titan 特殊约束的落地路径。目标是以最少的配置实现高吞吐、低延迟、可观测且易调优的执行链路，允许破坏性重构旧组件。

---

## 1. 现状回顾（痛点聚焦）
- **盲发策略**：`StrategyEngine::handle_action` 串行消费 `QuoteTask`，即使 `TradeProfile.amounts` 配置了多个 size，也会依次等待每条报价完成。
- **process_delay**：`MintSchedule::mark_dispatched` 在取出规模时立即推进下一次调度；因此当前节奏是“单 size → 等待 delay → 下一个 size”，无法在相同 delay 内尝试更多机会。
- **Multi-leg & Titan**：`MultiLegRuntime` 已有腿级别的并发，但仍缺少对本地 IP 的控制；Titan WebSocket 自带并发上限（最多两条 size 流），且连接阶段无法绑定指定 IP。
- **网络客户端**：DFlow / Ultra / Jupiter / lander 全部复用默认 `reqwest::Client`，无法区分来源 IP；Titan WebSocket 也未指定本地端口。
- **观测**：`monitoring::events` 未记录 `local_ip`、429 / timeout 等细节，排障困难。

---

## 2. 设计原则
1. **零成本抽象**：公共组件使用 trait + 泛型，避免在关键路径出现 `Box<dyn>`。
2. **有界并发**：并发度由显式配置和运行时资源（IP 数、Titan 限制）共同决定，所有调度都要落在 `tokio::Semaphore` 或自定义队列上。
3. **节奏透明**：`process_delay` 仍然定义“批次级节奏”，并发只在单次批次内展开；调度器返回的下一次唤醒时间保持准确。
4. **观测优先**：所有新模块必须自带 metrics / tracing 字段，以 `local_ip`、`batch_id`、`task_kind` 为核心标签。
5. **破坏性重构可接受**：删除旧的串行逻辑和多余的配置项，确保新方案干净易读。

---

## 3. 核心概念与组件

### 3.1 IP 资源池（`src/network/` 新增）
- `IpInventory`：启动时探测可用 IP（自动扫描 + 手动白名单 + 黑名单），过滤 loopback/down 接口；失败时回退默认 IP 并告警。
- `IpSlot`：封装 `IpAddr`、状态（Idle / Busy / CoolingDown / LongLived）、滚动统计（请求数、429、timeout）。
- `IpBoundClientPool<T>`：为每个 IP 懒加载 HTTP 客户端；`T: HttpClientFactory` 泛型允许 DFlow / Ultra / Jupiter / Lander 复用同一套池子。内部复用 TLS 证书与代理配置，保持零拷贝 buffer。
- `TitanSocketFactory`：基于 `IpLease` 构建绑定指定 IP 的 `TcpSocket`，供 Titan WebSocket 使用。

### 3.2 `IpAllocator`
```rust
enum IpTaskKind {
    QuoteBuy,
    QuoteSell,
    SwapInstruction,
    LanderSubmit { endpoint_hash: u64 },
    MultiLegLeg { aggregator: AggregatorKind, side: LegSide },
    TitanWsConnect,
}

enum IpLeaseMode {
    Ephemeral,        // HTTP 请求，生命周期受 future 控制
    SharedLongLived,  // Titan WS 等长连接，可与其他任务共享
}
```
- `acquire(kind, permit)`：阻塞式申请，返回 `IpLease`；`permit` 由上层并发控制传入（防止 lease 泄漏）。
- `IpLease` 持有 IP 元数据和引用计数，`Drop` 时自动归还；允许在 buy → sell 阶段复用同一 lease。
- `mark_result(lease_id, outcome)`：对 429、timeout、网络错误等做 IP 级退避（指数冷却起点 250–500ms），并产生日志/指标。
- 单 IP 模式下 `IpAllocator` 仍允许并发 lease，除非配置了 `per_ip_inflight_limit`；退避逻辑会控制下一轮请求的节奏。

### 3.3 Quote 批次与并发调度
- `QuoteBatchPlan`（新增数据结构）：
  ```rust
  struct QuoteBatchPlan {
      batch_id: u64,
      pair: TradePair,
      size: u64,
      process_delay: Duration,
  }
  ```
- `StrategyContext::drain_ready_batches(parallelism_hint)`：
  - 在 `process_delay` 允许范围内尽可能提取多条 size，数量受 `parallelism_hint` 控制。
  - 每个 batch 标记 `next_ready = now + process_delay`，保证节奏与旧逻辑一致。
- `QuoteDispatcher`（`src/engine/quote_dispatcher.rs` 新增）：
  - 使用 `tokio::Semaphore` + `FuturesUnordered` 实现有界并发，容量由 `effective_parallelism` 决定（见 4.1）。
  - 对每个 batch：先 `acquire(IpTaskKind::QuoteBuy)`，买腿报价完成后复用 lease 执行卖腿。
  - 支持可选的 `quote_batch_interval_ms`：当配置或退避逻辑要求时，在批次之间 sleep。

### 3.4 Titan 特殊处理
- Titan provider 在 `MultiLegRuntime` 中通过 `TitanControl` 维护信号量，最大并发 2；新增的 `quote_parallelism` 对 Titan 分支强制 clamp。
- Titan WebSocket 使用 `IpAllocator::acquire(TitanWsConnect)` 获取 `SharedLongLived` lease；长连接空闲时保持 `Idle` 状态，允许 HTTP 任务复用。

### 3.5 Lander / Swap / Multi-leg 集成
- `SwapPreparer`、`TransactionBuilder`、`LanderStack` 全部改为依赖 `Arc<IpAllocator>` 与对应的 client pool。
- `LanderStack::submit_plan` 在 `DispatchStrategy::OneByOne` 下使用 endpoint 哈希绑定 IP，确保同一路径的重试落在同一 IP 上，减少 RPC ban。
- Multi-leg 各腿 provider 调用 `IpAllocator`，保证 quote 与 swap 行为都遵循全局资源限制。

---

## 4. 并发度计算与节奏控制

### 4.1 `quote_parallelism`（新增配置）
- 配置路径：`engine.quote_parallelism`（全局默认）+ 各引擎 `quote_config.parallelism_override`（可选）。
- 取值含义：
  - `auto`：`min(trade_sizes.len(), ip_capacity, backend_limit)`。
  - 正整数：显式上限，仍与 `ip_capacity`、`backend_limit` 取最小值。
  - 缺省 / `1`：退化为串行。
- `ip_capacity` = `IpAllocator` 中处于 `Idle` 或 `LongLived` 状态的 slot 数；单 IP 模式仍返回 `>=1`。
- `backend_limit`：
  - DFlow / Ultra / Jupiter：默认 `usize::MAX`，可在配置中设置软上限。
  - Titan：固定为 2。

### 4.2 单 IP 并发策略
- `IpAllocator` 允许同一 IP 发起多个并发请求（上限可配置）。退避逻辑在 `mark_result` 中调节每个 IP 的下次可用时间，实现“一个 IP 并发 + 节奏控制”。
- 当频繁出现 429 时，`IpAllocator` 会缩减该 IP 的 effective capacity，并通过 metrics `galileo_ip_cooldown_total{ip}` 暴露。

### 4.3 `quote_batch_interval_ms`
- 配置路径：`engine.quote_batch_interval_ms`（可选），默认为 0。
- 含义：一个批次全部完成后，强制等待该时间再开始下一批；用于在单 IP 场景下拉长速率窗口。
- 当 `IpAllocator` 报告退避状态时，调度器可动态放大该间隔，避免热 IP 被继续打爆。

---

## 5. 配置变更
```yaml
bot:
  network:
    enable_multiple_ip: true
    manual_ips: []          # 可选，显式白名单
    per_ip_inflight_limit: null  # 可选，每个 IP 的并发上限
    cooldown_ms:
      rate_limited_start: 500
      timeout_start: 250

engine:
  quote_parallelism: auto
  quote_batch_interval_ms: 0

ultra:
  quote_config:
    parallelism_override: null   # 可选，覆盖全局

dflow:
  quote_config:
    parallelism_override: 4
```
- `enable_multiple_ip = false` 时，`IpAllocator` 创建单个 slot；并发逻辑仍可运行，但全部请求共享同一 IP。
- 旧的手写并发逻辑（若存在）全部删除，统一走新调度入口。

---

## 6. 观测与告警
- Prometheus 指标（新增）：
  - `galileo_ip_inventory_total{ip}`
  - `galileo_ip_inflight{ip}`、`galileo_ip_cooldown_total{ip,reason}`
  - `galileo_quote_batch_duration_seconds{strategy,backend}`
  - `galileo_quote_rate_limited_total{backend,ip}`
  - `galileo_lander_submission{lander,ip,result}`
- Tracing 扩展字段：`batch_id`、`local_ip`、`quote_parallelism`、`ip_cooldown_ms`。
- 文档更新：`docs/strategy_arch.md` 解释如何使用 `quote_parallelism`、`enable_multiple_ip` 调优；`galileo.yaml` 样例同步。

---

## 7. 实施阶段

### 阶段 1：基础设施
1. 创建 `src/network/`，实现 `IpInventory`、`IpSlot`、`IpBoundClientPool`。
2. 扩展配置解析，支持 `bot.network.*` 字段；启动时打印 IP 列表。

### 阶段 2：并发调度重构
1. 新增 `QuoteBatchPlan`、`StrategyContext::drain_ready_batches`。
2. 实现 `QuoteDispatcher`，在 `handle_action` 中接入，删除旧串行 for-loop。
3. `QuoteExecutor::round_trip`、`DflowApiClient`、`UltraApiClient`、`JupiterApiClient` 接受 `IpLeaseHandle`。

### 阶段 3：退避与观测
1. `IpAllocator::mark_result` 接入所有 HTTP / WS / RPC 调用，完善 cooldown 策略。
2. 补充 metrics、tracing 字段，更新 `monitoring::events`。

### 阶段 4：Multi-leg & Titan
1. `MultiLegRuntime` 使用 `IpAllocator` 控制腿级并发；Titan provider 绑定本地 IP。
2. Titan 控制器对 `quote_parallelism` clamp，并为超额配置打印 warn。
3. 集成测试覆盖 Titan 双 size 限制、IP 共享逻辑。

### 阶段 5：Lander & 落地
1. `SwapPreparer`、`TransactionBuilder`、`LanderStack` 接入 IP 池，完成 endpoint 粘性哈希。
2. 对 Jito / Staked 落地路径增加指标与退避。

### 阶段 6：压测与验收
1. 本地 mock 服务器模拟 429 / timeout，验证单 IP 与多 IP 下的退避行为。
2. 记录 Quote 吞吐、落地成功率、IP 利用率等指标，写入 PR。
3. 更新 `docs/strategy_arch.md`、`ROADMAP.md`、`titan_plus_dflow_design.md` 等引用。

---

## 8. 验证指标
- **性能**：Quote 吞吐、p50/p99 延迟、swap 构建耗时、落地成功率。
- **限流**：429 次数、退避时长、单 IP冷却比例。
- **资源**：`galileo_ip_inflight / galileo_ip_inventory`、Titan 长连接占用时长。
- **稳定性**：长跑测试中的错误率、冷却恢复成功率。

---

## 9. 风险与缓解
| 风险 | 描述 | 缓解措施 |
| --- | --- | --- |
| 特定环境禁止绑定某些 IP | `bind` 失败或权限不足 | 支持黑名单/回退默认 IP，启动时显式告警 |
| 单 IP 并发导致瞬间 429 | 节奏过猛 | 结合 `quote_batch_interval_ms` + `mark_result` 冷却；监控 429 指标 |
| Titan 长连占用导致饥饿 | 其它任务无法获取 IP | `SharedLongLived` 允许共享；空闲检测 + 定期释放 |
| Quote 批次错配 | 买卖腿调度错乱 | `batch_id` + 状态机校验；卖腿前检查买腿成功 |
| 多策略争用 IP | 盲发与 multi-leg 互相抢 slot | `IpTaskKind` 分级权重 + 指标监控；必要时引入优先级队列 |
| 客户端数量膨胀 | 大量 IP -> 大量 `reqwest::Client` | 懒加载 + LRU 释放；共享全局 TLS 池 |

---

通过上述重构，我们可以在单 IP 场景下安全地提高并发，必要时再打开多 IP 进一步扩容；所有策略、落地与监控链路都在同一资源池内协作，保证高性能与可观测性。下一步按阶段 1 启动实现。***
