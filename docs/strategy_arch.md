# Galileo 策略引擎架构草案（Spam / Blind）

> 目标：在 `galileo` 内构建生产级别的套利引擎，为 Spam 与 Blind 策略提供高性能、可扩展、可观测的执行框架。Back-run 将在完成 gRPC 监听、DEX 事件流等前置工作后独立推进。

## 1. 总体目标
- **低延迟**：端到端（行情→报价→组包→落地）延迟控制在 150~300ms。
- **高吞吐**：Spam 可持续 50+ qps 的报价 & bundle 提交；Blind 在多套利规模组合下保持稳定执行。
- **高可靠**：内建风控、重试与降级机制，避免单一组件故障导致套利挂起。
- **模块化**：策略层（Spam/Blind）共享底层组件（行情、缓存、落地、风控）。
- **易观测**：提供细粒度 metrics、分布式 tracing、结构化日志。

## 2. 模块划分

```
┌──────────────────────────────────────────────────────────┐
│                    Strategy Orchestrator                 │
│   ┌─────────────┬─────────────┬─────────────┬──────────┐ │
│   │ Spam Engine │ BlindEngine │ RiskEngine  │ Metrics  │ │
│   └──────┬──────┴──────┬──────┴──────┬──────┴──────┬───┘ │
└──────────┼─────────────┼─────────────┼─────────────┼─────┘
           │             │             │             │
   ┌───────▼──────┐┌─────▼─────┐┌──────▼──────┐┌─────▼──────┐
   │ Quote Router ││ CachePool ││ Tx Builder  ││ Bundle Tx  │
   │ (Jupiter)    ││ (Dash/Moka││ (Solana V0) ││ Sender     │
   └───────┬──────┘└─────┬─────┘└──────┬──────┘└─────┬──────┘
           │             │             │             │
   ┌───────▼──────┐┌─────▼─────┐┌──────▼──────┐┌─────▼──────┐
   │ Market Feed  ││ Task Sched││ Fee Manager ││ Failure Mgr│
   │ (RPC/gRPC)   ││ (Tokio/   ││ (Tips/PF)   ││ (Retry/Ban)│
   │              ││ Rayon)    ││             ││             │
   └──────────────┘└───────────┘└────────────┘└─────────────┘
```

### 2.1 Market Feed
- 来源：`global.rpc_url` / `global.yellowstone_grpc_url`。
- 作用：收集最新 slot、blockhash、account data、orderbook 深度。
- 输出数据结构：
  - `BlockhashContext`（用于 bundle 构建与签名）。
  - `MarketSnapshot`：围绕 base mint（配置中的套利基准 token，例如 WSOL/USDC）与中间 token (`intermedium.mints`) 的成交深度。
- 策略约束说明：
- Quote 的输入/输出 mint 均来自策略定义的基准资产或 `blind.base_mints`，中间资产集合 (`intermedium.mints`) 由配置决定，可形成 A→B→A 或 A→B→C→A 等多跳结构。
  - Spam/Blind 均依赖实时行情，不对 quote 结果做缓存。

### 2.2 状态存储（非 Quote 缓存）
- `DashMap`：维护 blockhash、最新 slot、bundle 结果、token metadata 等轻量状态。
- `Moka`（可选）：存放 orderbook / geyser 补丁等冷数据，提高稳态读取效率。
- 不缓存 quote：高频报价必须实时命中 Jupiter，以避免利润遗漏或使用过期价格。
- 落地路径：必要时将关键快照（slot、blockhash、失败统计）写入 `sled`，保证故障恢复能力。

### 2.3 Quote Router
- 将策略请求映射到 Jupiter Quote API。
- 参数对齐：
  - `onlyDirectRoutes`：Spam 强制 `true`。
  - `restrictIntermediateTokens`：Blind/Spam 继承 `request_params`。
  - `dexes` 白名单：基于 `request_params.included_dexes` + 策略自定义 `enable_dexs`。
- 快速失败策略：
  - `timeout_ms`（继承 `bot.request_timeout_ms`）。
  - `retry`：Spam 提供 2 次快速重试；Blind 仅在失败率升高时启用重试。
- 预留回调：quote 返回后立刻写 metrics、触发下一次调度（不落缓存）。

### 2.4 Task Scheduler
- 生产者：Spam & Blind 策略主循环。
- 消费者：报价执行 worker（Tokio runtime）+ 计算密集 worker（Rayon）。
- 通道设计：
  - `async_channel::bounded` -> Spam 使用 `try_send`；队列满时直接丢弃旧任务（避免阻塞）。
  - `tokio::mpsc::channel` -> Blind 使用 低水位 prefetch。
- 背压策略：结合 `controls.over_trade_process_delay_ms` 与 `blind.base_mints[*].process_delay`。

### 2.5 Tx Builder
- 输入：`SwapInstructionsResponse`、策略上下文（memo、小费、风控限制）。
- 功能：
  - 组装 compute budget 指令（尊重 `request_params.dynamic_compute_unit_limit`）。
  - 附加策略备注 (`global.instruction.memo` / `blind.memo`)。
- 追加指令：
    - Memo：`global.instruction.memo` 或策略自定内容。
    - Tip/优先费：Spam 使用 `spam.compute_unit_price_micro_lamports` / Jito tip；Blind 可读取 `BlindConfig` 静态 tip。
    - 额外组件：闪电贷、风控检查等指令在该阶段注入。
- 处理 Landers：根据 `enable_landers` 选择 Jito、Staked 等发送通道。
  - Blockhash 管理：支持 RPC 获取或 gRPC 订阅（`bot.get_block_hash_by_grpc`）。
- 输出：`PreparedBundle { slot, blockhash, transactions }`。

### 2.6 Bundle Sender
- 多路发送：
  - `JitoSender`：HTTP bundle API（`lander.yaml`）。
  - `RPCSender`：常规 `send_transaction`（默认/`staked` 共享实现）。
- 重试与超时：
  - Spam：重试 `spam.max_retries` 次，失败放入 `FailureMgr`，记录 metrics。
  - Blind：单次失败直接降级到下一落地通道，防止阻塞。

### 2.7 Risk & Failure Manager
- 风控：
  - 余额检查：`global.wallet.min_sol_balance`。
  - 滑点/容错：确保 `profit - tip - prioritization_fee >= threshold`。
  - 冷却时间：Blind 参照 `blind.base_mints[*].sending_cooldown`，Spam 使用全局 `controls.over_trade_process_delay_ms`。
- 失败处理：
  - Blacklist Dex：当连续失败超过阈值时，将 dex 加入临时黑名单。
  - Retry 队列（有限大小）：避免洪泛造成雪崩。

## 3. 策略细节

### 3.1 Spam
- 配置来源：`galileo.yaml` → `spam` 段。
- 核心特点：
  - **基准资产循环**：输入 / 输出 mint 由策略配置控制，可形成两跳或多跳结构（如 A→B→A、A→B→C→A），中间代币来自 `intermedium.mints` 或策略段落中声明的 hop 列表。
  - **直连路线**：强制 `onlyDirectRoutes = true`，追求最短路径延迟。
  - **Landers**：根据 `spam.enable_landers` 选择落地（默认 Jito / Staked）。
  - **优先费**：使用 `spam.compute_unit_price_micro_lamports`；如为 0，则按利润比例动态计算。
  - **预检查**：为压缩延迟，默认 `skip_user_accounts_rpc_calls = true`（若配置允许）。
  - **重试**：`spam.max_retries` 控制；失败记录日志（`enable_log`）。

执行流程：
1. 轮询基准资产对（A→B→A），连续两次 quote。
2. 立即计算利润，满足 `min_profit_threshold_lamports` 时进入执行链。
3. 使用首个 quote 的原始 JSON 作为 payload 调用 `/swap-instructions`。
4. 在返回指令末尾附加 memo / tip / 闪电贷等附加指令。
5. 组装 bundle 并通过 Lander 提交，记录 `spam.profit_lamports`、`spam.bundle_latency_ms` 等指标。

### 3.2 Blind
- 配置来源：`galileo.yaml` → `blind` 段。
- 核心特点：
  - **交易规模**：基于 `base_mints[*].trade_size_range` 与 `trade_range_count` 生成多组交易量。
  - **Route 类型**：支持 `2hop` 与 `3hop`，在 Quote Router 中映射成 `onlyDirectRoutes=false` 且 `three_hop_mints` 作为中间币白名单。
  - **利润阈值**：`min_quote_profit` + 可选 `TipCalculator`。
  - **节奏控制**：`process_delay` 决定每对 mint 对应请求的间隔；`sending_cooldown` 管理同一 mint 的最小发送间隔。
  - **Memo**：使用 `blind.memo` 或 `global.instruction.memo`。

执行流程：
1. 对每个 `base_mint` 创建独立 worker，按 `trade_size_range` 生成交易规模。
2. 若配置了 `three_hop_mints`，在 Quote Router 中插入 `restrictIntermediateTokens=false` 且 `onlyDexes` 添加三跳路径必要的 dex。
3. 按 `process_delay` 轮询 quote，基于缓存结果判断是否继续请求新的 quote。
4. 收益超过 `min_quote_profit` 后进入执行链，依据 `sending_cooldown` 检查是否放行。
5. Bundle 发送时可选择 Staked 通道（若在 `enable_landers` 中配置），默认 Jito。

## 4. 配置映射汇总

| 配置项 | 模块 | 行为 |
| --- | --- | --- |
| `request_params.*` | Quote Router / Tx Builder | 构建 quote/swap 请求默认值。 |
| `spam.enable` | Strategy Orchestrator | 控制 Spam Engine 是否激活。 |
| `spam.skip_preflight` | Tx Builder | 组装 bundle 时控制模拟策略。 |
| `spam.compute_unit_price_micro_lamports` | Fee Manager | 覆写 CU 优先费。 |
| `blind.base_mints[*].trade_size_range` | Blind Engine | 生成交易规模队列。 |
| `blind.base_mints[*].route_types` | Quote Router | 切换 2/3-hop 报价。 |
| `blind.base_mints[*].three_hop_mints` | Quote Router | 扩展 intermediate token 白名单。 |
| `global.wallet.min_sol_balance` | Risk Manager | 余额低于阈值时暂停策略。 |
| `bot.enable_simulation` | Tx Builder | 决定是否执行预模拟或直接发送 bundle。 |

## 5. 指标与日志

- Metrics
  - `strategy.quote.latency_ms{strategy=spam}` 等。
  - `strategy.bundle.success_total` / `failure_total`。
  - `strategy.cache.hit_ratio{layer=hot|cold}`。
  - `strategy.tip.lamports_sum`。
- 日志
  - 结构化：`strategy::spam`, `strategy::blind`, `risk`, `bundle`.
  - 关键字段：`slot`, `blockhash`, `dex`, `profit`, `tip`, `lander`.
- Tracing
  - 采用 `tracing` span，将一次套利机会标记 `trace_id=<bundle_id>`。

## 6. 实施路线

1. **P0**：抽象 `StrategyContext`（包含 cache、quote router、tx builder、bundle sender）。
2. **P1**：实现 Spam Engine → Quote→Swap→Bundle 完整链路，覆盖重试/日志/metrics。
3. **P2**：实现 Blind Engine → 多 worker 架构、三跳支持、节奏控制。
4. **P3**：接入 Risk Manager（余额、滑点、失败降级）。
5. **P4**：补充压测脚本与指标面板，整理 README/Docs。

> 说明：Back-run 依赖的 gRPC DEX 监听与池子事件归档不在当前草案范围，将在 Spam/Blind 稳定后迭代。
