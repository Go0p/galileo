# Galileo 策略与执行架构

> 目标：在保证极致性能的前提下，通过 trait + 泛型实现零成本抽象，统一多种落地通道，实现可观测、可扩展的套利引擎。本文描述整个套利链路的模块职责、目录规划、扩展规则与监控要求。

## 1. 核心设计目标
- **高性能**：端到端延迟控制在 150~300ms；Quote 环节并发拉满，避免任何阻塞等待；不对 Quote 做缓存，保证发现收益实时。
- **零成本抽象**：关键路径全部采用 trait + 泛型，编译期单态化，避免动态分发；落地通道、策略逻辑均可在编译时拼装。
- **优雅分层**：策略（业务意图）与引擎（调度、资源、容错）彻底解耦；落地器通过统一接口提供多实现；监控模块贯穿全链路。
- **易扩展**：新增策略只需新增 `src/strategy/<name>.rs`；新增落地器或风控策略不用触及现有核心逻辑。
- **强观测**：统一的 metrics / tracing / logging 埋点策略，能够追踪一笔套利机会从 Quote 到落地的每一步。

## 2. 模块分层与目录约定

```
src/
├── engine/            # 调度、worker、风险、数据管线
│   ├── context.rs     # EngineContext<TL: Lander>
│   ├── quote.rs       # QuoteExecutor/Router
│   ├── swap.rs        # SwapInstructionFetcher
│   ├── builder.rs     # TransactionBuilder
│   ├── scheduler.rs   # Tokio/Rayon 协调
│   └── mod.rs
├── strategy/          # 业务策略(Spam/Blind/未来策略)
│   ├── spam.rs
│   ├── blind.rs
│   ├── backrun.rs     # 例：未来扩展
│   └── mod.rs         # 仅做 re-export
├── lander/            # 上链通道 trait + 实现
│   ├── traits.rs      # pub trait Lander
│   ├── jito.rs
│   ├── third_party.rs
│   ├── rpc.rs
│   ├── staked.rs
│   └── mod.rs
├── monitoring/        # metrics/tracing/logging 封装
│   ├── metrics.rs
│   ├── tracing.rs
│   ├── events.rs      # 自定义事件上报
│   └── mod.rs
└── ...
```

- **策略命名规范**：`src/strategy/<策略名>.rs`，例如 `spam.rs` / `blind.rs`，便于后续增量添加。
- **Engine 层**：仅关心「如何高效执行」，不关心套利意图；对外暴露 `StrategyEngine<TL: Lander>`。
- **共享类型**：放在 `types.rs`、`config.rs` 等主题化文件中，不在 `mod.rs` 内定义结构体或 trait。
- **监控模块**：与 Engine、Strategy、Lander 均解耦，只通过事件/回调传递指标数据。

## 3. Jupiter Quote → Swap → Lander 执行链路

### 3.1 Quote 阶段
- 依赖本地 Jupiter 二进制，针对配置文件中的 `base_mint` 反复 Quote；配置项 `mints` 仅表示可能的中间市场（A→B→A 时的 B）。
- 输入/输出 mint 均固定为 `base_mint`，通过两次 Quote（正向、反向）确认利润，Quote 请求参数应直接来自配置（`request_params`）。
- Quote 频率极高，不启用缓存；失败快速回退，使用 `try_send` + 降级通道防止积压。
- `QuoteExecutor` 负责构造请求、执行 HTTP/gRPC 调用、解析响应，并同步写入监控。

### 3.2 盈利判定
- `ProfitEvaluator` 接收双向 Quote 结果，执行手续费、滑点、tip 开销评估。
- 若盈利则将两次 Quote 的返回值（含 `swapMode`, `route_plan` 等）封装为 `SwapOpportunity` 推入 Engine。
- 可配置的风控门限：`min_profit`, `max_slippage`, `cooldown` 等均在 Strategy 层完成判定，Engine 只负责并发调度。

### 3.3 Swap 指令获取
- `SwapInstructionFetcher` 通过 `/swap-instructions` API 获取 Jupiter 返回的核心指令集。
- 按策略要求在指令序列前/后插入扩展指令：
  - Memo（调试 / 审计）
  - Jito tip（需额外小费账户）
  - 第三方加速器 tip
  - 闪电贷账户指令
- 指令装配结果形成 `TransactionPlan`，继续交由 Builder。

### 3.4 交易打包
- `TransactionBuilder` 对 `TransactionPlan` 执行：
  1. 按 blockhash 缓存设置最近的 `recent_blockhash`；
  2. 填充 signer（来自配置 `bot.identity`）；
  3. 计算 CU、优先费，若策略重写 `compute_unit_price` 则在此应用；
  4. 输出 `PreparedTxn<TL::Payload>`，其中 `Payload` 由具体 Lander 决定。
- 该阶段完全泛型化，允许通过 trait 约束在编译期选择不同的打包策略。

### 3.5 Lander 发送
- `EngineContext` 调用泛型参数 `TL: Lander` 的实现，将交易提交到对应上链服务。
- 所有落地器返回 `LanderOutcome`（成功 / 失败 / 可重试 / 不可重试），Engine 根据结果决定重试策略或熔断。

## 4. Lander Trait 设计

```rust
pub trait Lander {
    type Payload<'a>: Send + 'a;
    type Response: Send;

    fn name(&self) -> &'static str;

    fn prepare<'a>(
        &'a self,
        tx: &'a solana_sdk::transaction::VersionedTransaction,
        ctx: &'a LanderContext,
    ) -> Result<Self::Payload<'a>, LanderError>;

    fn submit(
        &self,
        payload: Self::Payload<'_>,
        deadline: Deadline,
    ) -> impl Future<Output = Result<Self::Response, LanderError>> + Send;
}
```

- **零成本抽象**：Engine 使用 `impl<L: Lander> StrategyEngine<L>`，编译器为每种落地器生成专用代码。
- **实现类型**：
  - `JitoLander`：插入 bundle tip，支持 Jito 特有的认证与 bundle API。
  - `RpcLander`：使用标准 RPC `sendTransaction`，可选优先费。
  - `StakedLander`：面向 stake-driven 服务，添加 stake tip 指令。
  - `ThirdPartyLander`：扩展额外指令或认证信息（例如自建中继）。
- **调度策略**：Engine 可持有多个 Lander，实现多播或容错，例如 `Primary<Jito> + Fallback<Rpc>`。
- **配置映射**：`lander.yaml` 定义默认落地器与优先费参数，通过 `LanderFactory` 在启动时注入。

## 5. Engine / Strategy 解耦模式

```rust
pub trait Strategy {
    type Event;
    fn on_market_event(&mut self, event: &Self::Event, ctx: &mut StrategyContext) -> Action;
}

pub struct StrategyEngine<L: Lander> {
    lander: L,
    scheduler: Scheduler,
    quote_executor: QuoteExecutor,
    swap_fetcher: SwapInstructionFetcher,
    tx_builder: TransactionBuilder<L>,
    observers: ObserverRegistry,
}
```

- **策略层**：仅处理业务意图（何时 Quote、如何判定利润、如何调度多 base mint）。文件独立（`spam.rs`, `blind.rs`）。支持通过泛型将策略绑定到 Engine，例如 `Engine<SpamStrategy, JitoLander>`。
- **Engine 层**：负责资源调度、并发、重试、失败隔离、监控上报；完全独立于具体策略。
- **上下文对象**：`StrategyContext` 向策略暴露 Quote 请求句柄、异步任务派发、风险决策接口等；策略无法直接访问底层实现，只能通过 trait。
- **任务通道**：继续沿用 README 中的建议：`async_channel`（Spam）、`tokio::mpsc`（Blind）、`rayon` 处理 >10ms 计算。
- **扩展流程**：新增策略 -> 新建文件 -> 实现 `Strategy` trait -> 在 orchestrator 中注册。

## 6. 监控贯穿设计

- **统一事件模型**：每个关键步骤向 `ObserverRegistry` 发送结构化事件（QuoteStart/QuoteEnd, SwapFetched, TxBuilt, LanderSubmitted, LanderFailed）。
- **Metrics**：通过 `metrics` crate 暴露
  - `quote.latency_ms{strategy=spam}`
  - `swap.fetch.success_total`
  - `lander.submit.latency_ms{lander=jito}`
  - `profit.estimated_lamports`
  - `engine.retry.count{reason=tip_fail}`
- **Tracing**：使用 `tracing::span!` 在一次套利流程中传播 `trace_id`，Lander 回执后关闭 span。
- **Logging**：结构化日志附带 `slot`, `blockhash`, `dex`, `tip`, `profit`, `lander`。
- **告警**：监控模块支持阈值告警（Quote 失败率、落地失败率、利润分布异常）。

## 7. 性能敏感点与优化策略

- **Quote 优先级**：Quote 是最长耗时环节，Engine 应支持并发批量发起，并在配置中限制最大并行数；避免所有策略共用单队列导致阻塞。
- **无缓存策略**：保持 Quote 实时性，除非后续需要冷启动快照，否则不做任何本地缓存。
- **线程模型**：沿用 README 指导——异步 IO 与 rayon 计算分离；计算密集型逻辑（盈利判断、路径评估）放入 rayon 持久线程池。
- **背压机制**：`try_send` + 丢弃旧任务，Strategies 按 `process_delay` 控制节奏；Engine 提供统一的 `RateLimiter` 工具。
- **Blockhash / 账户状态**：通过 DashMap 管理，更新频次与 Engine 解耦；监控中暴露 freshness。

## 8. 配置映射（节选）

| 配置项 | 影响模块 | 描述 |
| --- | --- | --- |
| `strategy.spam.*` / `strategy.blind.*` | `src/strategy/*.rs` | 策略节奏、利润阈值、禁用状态；策略模块读取后决定是否发起 Quote。 |
| `bot.request_params.*` | `engine::quote` | 构造 Jupiter Quote 请求的默认参数。 |
| `lander.enable` / `lander.type` | `lander::*` | 选择默认 Lander 实现与启用列表。 |
| `lander.tips` | `engine::builder` + `lander::*` | 配置 tip 账户、优先费；Builder 根据策略覆盖 CU。 |
| `controls.over_trade_process_delay_ms` | `engine::scheduler` | 控制任务节奏，防止过载。 |
| `monitoring.*` | `monitoring::*` | 指标上报端点、采样率、日志级别等。 |

## 9. 迭代路线建议

1. **P0**：落地 `lander` 目录与 `Lander` trait，Jito/RPC 实现先行；引擎现有逻辑迁入 `engine::*`。
2. **P1**：重构 `strategy` 目录，Spam/Blind 拆分为独立文件并实现统一 `Strategy` trait。
3. **P2**：补齐监控模块，统一事件流；梳理 metrics 名称，接入现有监控体系。
4. **P3**：引入多落地器编排（主用 Jito + 备选 RPC/Staked），完善失败重试策略。
5. **P4**：为未来 Back-run 策略预留接口，扩展策略 orchestrator，补充测试与压测脚本。

> 通过以上分层与规范，可在不牺牲性能的情况下，实现落地通道与策略的组合爆炸，同时保持代码库的可维护性与可观测性。
