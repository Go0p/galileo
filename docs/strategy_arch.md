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

## 4. Lander 栈设计

```rust
pub struct LanderStack {
    landers: Vec<LanderVariant>,
    max_retries: usize,
}

impl LanderStack {
    pub async fn submit(
        &self,
        prepared: &PreparedTransaction,
        deadline: Deadline,
    ) -> Result<LanderReceipt, LanderError> { /* ... */ }
}
```

- **零成本抽象**：落地类型以 `LanderVariant`（枚举）编译期收敛，运行时仅做 `match`；未引入 `Box<dyn>`。
- **实现类型**：
  - `RpcLander`：复用共享 `RpcClient` 的 `send_transaction`。
  - `JitoLander`：编码 bundle 并向配置的 Jito endpoints 提交。
  - `StakedLander`：与 `RpcLander` 一样提交 JSON-RPC 交易，只是路由到经过质押的专用节点，逻辑保持一致。
- **调度策略**：`LanderFactory` 读取 `lander.yaml` 与策略侧 `enable_landers` 顺序，构造 `LanderStack`，并根据 `max_retries`（Spam）或单次尝试（Blind）决定重试节奏。
- **监控与日志**：每次尝试都会通过 `monitoring::events::lander_*` 打点，方便排查失败原因与成功路径。

## 5. Engine / Strategy 解耦模式

```rust
pub trait Strategy {
    type Event;
    fn on_market_event(&mut self, event: &Self::Event, ctx: StrategyContext) -> Action;
}

pub struct StrategyEngine<S: Strategy> {
    landers: LanderStack,
    scheduler: Scheduler,
    quote_executor: QuoteExecutor,
    swap_fetcher: SwapInstructionFetcher,
    tx_builder: TransactionBuilder,
    strategy: S,
}
```

- **策略层**：只关注调度顺序/节奏（例如 Spam 随机打散对、Blind 顺序遍历），通过 `StrategyContext` 请求引擎执行 Quote。
- **Engine 层**：负责资源调度、盈利判定、交易构建、落地与监控打点，细节完全从策略抽离；当配置 `bot.dry_run = true` 或 CLI 以 `galileo dry-run` 启动时，交易会在构建后记录日志并跳过落地提交，用于安全演练。
- **上下文对象**：策略仅能调用 `schedule_pair_all_amounts` 等接口，无法越级访问底层实现，保证安全与解耦。
- **扩展流程**：新增策略 -> 新建文件实现 `Strategy` -> 在 `main` 中按模式注册即可。

## 6. 监控贯穿设计

- **统一事件模型**：每个关键步骤向 `ObserverRegistry` 发送结构化事件（QuoteStart/QuoteEnd, SwapFetched, TxBuilt, LanderSubmitted, LanderFailed）。
- **事件实现**：`monitoring::events` 模块封装 Quote/Swap/Transaction/Lander 的结构化 `tracing` 打点，Engine 调用即可在日志和指标系统中统一呈现。
- **Prometheus 指标**：`monitoring::events` 会在配置启用时同步通过 `metrics` 宏上报关键指标。
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

## 10. 性能分析与 Hotpath 集成

- **依赖声明**：在根 `Cargo.toml` 中追加
  ```toml
  [dependencies]
  hotpath = { version = "0.4", optional = true }

  [features]
  hotpath = ["dep:hotpath", "hotpath/hotpath"]
  hotpath-alloc-bytes-total = ["hotpath/hotpath-alloc-bytes-total"]
  hotpath-alloc-count-total = ["hotpath/hotpath-alloc-count-total"]
  hotpath-off = ["hotpath/hotpath-off"]
  ```
  所有代码需通过 `cfg(feature = "hotpath")` 宏保护，默认不开启零开销。
- **函数与代码块打点**：在 `engine`、`strategy` 等关键路径函数上使用
  ```rust
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  async fn evaluate_pair(...) { ... }

  #[cfg(feature = "hotpath")]
  hotpath::measure_block!("swap_fetch", {
      // 需要排查的临时代码块
  });
  ```
  Tokio 启动函数保持 `#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [95, 99]))]`，避免手动管理 Guard。
- **启用方式**：采用 `cargo run --features=hotpath` 打印时延分布；如需内存分配分析，额外启用 `hotpath-alloc-bytes-total` 或 `hotpath-alloc-count-total`，同时将 Tokio 运行时切换到 `current_thread`（参考 Hotpath README 示例），以保证 TLS 统计准确。
- **输出落地**：Hotpath 默认在进程退出时输出表格。对 CI/基准测试，可使用 `GuardBuilder` 指定 `Format::Json` 并将结果重定向到文件，结合现有 `monitoring` 模块统一归档。
- **定位策略**：建议优先包裹 `QuoteExecutor::round_trip`、`ProfitEvaluator::evaluate`、`TransactionBuilder::build` 等耗时函数，并通过 `measure_block!` 区分落地重试、Jito/RPC 分支耗时，以便快速识别瓶颈。

## 11. Prometheus 指标接入

- **配置入口**：在 `galileo.yaml` 中新增（或继承默认）监控端口配置：

  ```yaml
  bot:
    prometheus:
      enable: true
      listen: "0.0.0.0:9898"
  ```

  启动后，galileo 会在 `listen` 地址暴露 `/metrics`，Jupiter 本身仍通过 `jupiter.core.metrics_port` 对外提供自身指标。

- **指标示例（带有 `strategy`、`lander` 标签）**：
  - `galileo_quote_total{strategy, result}`
  - `galileo_quote_latency_ms_bucket`
  - `galileo_opportunity_detected_total{strategy}`
  - `galileo_swap_compute_unit_limit_bucket{strategy}`
  - `galileo_transaction_built_total{strategy}`
  - `galileo_lander_attempt_total{strategy, lander}`
  - `galileo_lander_success_total{strategy, lander}`、`galileo_lander_failure_total{strategy, lander}`

- **抓取与看板**：
  1. 在 Prometheus `scrape_configs` 中新增 job 指向 galileo 监听端口；Jupiter 端口（默认 `18081`）也保持抓取。
  2. Grafana 可基于上述指标构建“机会发现/成功率、Quote 延迟、Lander 成功率”等看板。结合 Hotpath 报告可快速定位瓶颈。

- **性能注意事项**：仅在配置中启用 Prometheus 时才会初始化 exporter 并真正上报指标；默认关闭时，`metrics` 调用会落入空实现，不影响主套利流程。
