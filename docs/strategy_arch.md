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
├── engine/
│   ├── assembly/
│   │   ├── bundle.rs          # InstructionBundle 分段结构
│   │   └── decorators/        # ComputeBudget / Guard / Flashloan / Profit 装饰器
│   ├── runtime/
│   │   └── strategy/
│   │       ├── mod.rs         # 生命周期入口 + 配置
│   │       ├── blind.rs       # 盲发批处理
│   │       ├── quote.rs       # 并发报价调度
│   │       ├── swap.rs        # 装配落地流水线
│   │       └── multi_leg.rs   # 多腿机会聚合
│   ├── quote.rs               # QuoteExecutor / ProfitEvaluator
│   ├── swap_preparer.rs       # 聚合器指令预处理
│   ├── planner.rs             # TxVariantPlanner + DispatchPlan
│   ├── builder.rs             # TransactionBuilder
│   └── ...
├── instructions/              # 协议指令原语（无业务状态）
│   ├── compute_budget.rs
│   ├── flashloan/
│   │   ├── error.rs
│   │   ├── marginfi.rs
│   │   └── types.rs
│   ├── guards/
│   │   └── lighthouse/
│   │       ├── mod.rs
│   │       ├── account_delta.rs
│   │       ├── guard.rs
│   │       └── memory_write.rs
│   └── jupiter/
│       ├── route.rs
│       ├── route_v2.rs
│       ├── swaps.rs
│       └── types.rs
├── strategy/                  # 业务策略（盲发 / 复制等）
│   ├── blind/
│   ├── pure_blind/
│   ├── common/
│   └── mod.rs                 # 仅做 re-export
├── lander/                    # 上链通道 trait + 实现
├── monitoring/                # metrics / tracing / logging 封装
└── ...
```

- **策略目录**：`src/strategy/<name>/` 组织策略实现，`mod.rs` 负责 re-export 并标注维护者。
- **Engine 层**：`StrategyEngine<S>` 负责协调 Quote → Assembly → Landing，全链路依赖在构造时注入。
- **共享类型**：通用结构体与配置放在 `engine/types.rs`、`config/types.rs` 等主题化文件，不在 `mod.rs` 内定义。
- **指令原语**：所有协议 `Instruction` 构造逻辑集中在 `instructions/`，保持 engine/strategy 纯粹。
- **监控模块**：与 Engine、Strategy、Lander 解耦，通过 `monitoring::events` 输出日志与指标。

## 3. 聚合器 Quote → Swap → Lander 执行链路

### 3.1 Quote 阶段
- 引擎通过远端聚合器 API（DFlow / Ultra / Kamino）按配置的 `base_mint` 批量 Quote；`mints` 仍表示潜在的中间市场（A→B→A 时的 B）。
- 输入/输出 mint 均固定为 `base_mint`，通过两次 Quote（正向、反向）确认利润，Quote 请求参数应直接来自配置（`request_params`）。
- Quote 频率极高，不启用缓存；失败快速回退，使用 `try_send` + 降级通道防止积压。
- `QuoteExecutor` 负责构造请求、执行 HTTP/gRPC 调用、解析响应，并同步写入监控。
- 当前引擎内置 DFlow / Kamino / Ultra 三类聚合器，通过 `galileo.engine.backend` 切换。Kamino 直接复用 `/kswap/all-routes` 响应内的指令集，不再额外调用 swap-instructions API。

### 3.2 盈利判定
- `ProfitEvaluator` 接收双向 Quote 结果，执行手续费、滑点、tip 开销评估。
- 若盈利则将两次 Quote 的返回值（含 `swapMode`, `route_plan` 等）封装为 `SwapOpportunity` 推入 Engine。
- 可配置的风控门限：`min_profit`, `max_slippage`, `cooldown` 等均在 Strategy 层完成判定，Engine 只负责并发调度。

### 3.3 指令装配流水线
- `SwapPreparer` 根据 `SwapOpportunity` 调用 DFlow / Ultra / Kamino 等聚合器，返回 `SwapInstructionsVariant`（含指令、ALT、compute budget、tip 等元数据）。
- 引擎将响应转换为 `InstructionBundle`，按 `compute_budget` / `prefix` / `main` / `post` 四段组织，避免后续反复拷贝。
- `DecoratorChain` 顺序执行装饰器：
  - `FlashloanDecorator`：若 `MarginfiFlashloanManager` 可用，则调用 `instructions::flashloan::marginfi::MarginfiFlashloan` 将主体指令包裹在 Begin/Borrow/Repay/End 中。
  - `ComputeBudgetDecorator`：写回 compute unit limit / price，保持 bundle 与 `SwapInstructionsVariant` 一致。
  - `GuardBudgetDecorator`：合并基础费用、Jito tip、Lighthouse guard 预算。
  - `ProfitGuardDecorator`：注入 Lighthouse token 守护，记录 guard 相关元数据。
- 每个阶段都会触发 `monitoring::events::assembly_*` 打点，记录装饰器数量、CU 变化、guard/tip/flashloan 是否生效等细节，便于排障与性能分析。

### 3.4 交易打包
- `InstructionBundle` 完成装配后交给 `TransactionBuilder` 构建 `PreparedTransaction`：
  1. 读取最新 `blockhash` 与 `slot`；
  2. 根据 `EngineIdentity` 与策略参数写入签名、优先费、tip；
  3. 汇总最终指令（ComputeBudget + Prefix + Main + Post）与 ALT。
- Builder 保持零成本抽象，不直接感知策略或装饰器，只消费整理好的 bundle。

### 3.5 交易变体与发送计划
- `TxVariantPlanner` 基于 `PreparedTransaction`、`DispatchStrategy` 与落地器容量生成 `DispatchPlan`：
  - `AllAtOnce`：产出 1 个 `TxVariant`，所有落地器共享同一笔交易；
  - `OneByOne`：按落地端点生成多个 `TxVariant`，为 tip/优先费微调留下空间。
- `DispatchPlan` 交由 `LanderStack::submit_plan` 执行，统一处理 IP 退避、重试与超时，并让 `variant_id` 在监控中可追踪。
- Jito / Lightning 等落地器会结合 `variant_id` 轮换 tip 钱包，同时利用 uuid 池保证速率配额。

## 4. Lander 栈设计

```rust
pub struct LanderStack {
    landers: Vec<LanderVariant>,
    max_retries: usize,
}

impl LanderStack {
    pub fn variant_layout(&self, strategy: DispatchStrategy) -> Vec<usize> { /* ... */ }

    pub async fn submit_plan(
        &self,
        plan: &DispatchPlan,
        deadline: Deadline,
        strategy_name: &str,
    ) -> Result<LanderReceipt, LanderError> { /* ... */ }
}
```

- **零成本抽象**：落地类型以 `LanderVariant`（枚举）编译期收敛，运行时仅做 `match`；未引入 `Box<dyn>`。
- **实现类型**：
  - `RpcLander`：复用共享 `RpcClient` 的 `send_transaction`。
  - `JitoLander`：编码 bundle 并向配置的 Jito endpoints 提交。
  - `StakedLander`：与 `RpcLander` 一样提交 JSON-RPC 交易，只是路由到经过质押的专用节点，逻辑保持一致。
- **调度策略**：`LanderFactory` 读取 `lander.yaml` 与策略侧 `enable_landers` 顺序，构造 `LanderStack`。Engine 根据配置选择 `DispatchStrategy`（AllAtOnce/OneByOne），先由 `TxVariantPlanner` 生成 `DispatchPlan`，再委托 `LanderStack::submit_plan` 执行落地。
- **策略细节**：
  - `AllAtOnce`：对每个落地器并发推送同一笔交易，最节省费用；任一落地成功后即终止剩余请求。
  - `OneByOne`：为各端点生成独立 `variant_id`，并按 `(variant, endpoint)` 顺序快速提交，可规避部分速率限制；Jito 端会为不同变体轮换 tip 钱包，同时通过 uuid 池在 query string 加入 `uuid`。
- **监控与日志**：每次尝试都会通过 `monitoring::events::lander_*` 打点，方便排查失败原因与成功路径。

## 5. Engine / Strategy 解耦模式

```rust
pub trait Strategy {
    type Event;

    fn name(&self) -> &'static str;

    fn on_market_event(
        &mut self,
        event: &Self::Event,
        ctx: StrategyContext<'_>,
    ) -> StrategyDecision;
}
```

- **策略层**：聚焦调度顺序/节奏（如 Blind 批次、PureBlind 复制），通过 `StrategyContext` 唤起 Quote、刷新配置或回收资源，最终返回 `StrategyDecision` 供引擎执行。
- **引擎层组成**：
  - `quote.rs` + `quote_dispatcher.rs`：管理多源 Quote 并发、IP 退避、机会封装。
  - `swap_preparer.rs`：针对 DFlow / Ultra / Kamino 等后端生成 `SwapInstructionsVariant`。
  - `runtime/strategy/{quote,swap}.rs`：驱动报价批次、执行计划、装配流水线。
  - `assembly/`：`InstructionBundle` + `DecoratorChain` 抽象，确保装饰器顺序统一。
  - `planner.rs`：`TxVariantPlanner` 依据落地策略产出 `DispatchPlan`。
  - `runtime/lighthouse.rs` + `instructions::guards::lighthouse/`：负责利润守护 runtime 与指令原语。
  - `plugins/flashloan/`：闪电贷 Manager / 账户确保，指令原语位于 `instructions::flashloan/`。
- **运行流程**：策略只暴露事件处理入口，引擎负责资源调度、盈利判定、指令装配、交易构建、落地与监控打点；当 `bot.dry_run.enable = true`（或 CLI `galileo dry-run`）时，所有 RPC 请求与交易落地都会改用 `bot.dry_run.rpc_url` 指向的本地/沙箱节点，落地器强制回退为 RPC，便于本地回放。
- **扩展方式**：新增策略 → 在 `src/strategy/<name>/` 下实现 `Strategy` + 配置解析 → 在 `strategy::mod.rs` 中注册即可。

## 6. 监控贯穿设计

- **统一事件模型**：每个关键步骤向 `ObserverRegistry` 发送结构化事件（QuoteStart/QuoteEnd, SwapFetched, TxBuilt, LanderSubmitted, LanderFailed）。
- **事件实现**：`monitoring::events` 模块封装 Quote/Swap/Transaction/Lander 的结构化 `tracing` 打点，Engine 调用即可在日志和指标系统中统一呈现。
- **Prometheus 指标**：启用 `bot.prometheus.enable` 后，`monitoring::events` 会通过 `metrics` 宏上报下列核心指标族（列出基础名称，Prometheus 会自动衍生 `_bucket/_count/_sum`）：
  - 报价链路：`galileo_quote_total{strategy,result}`、`galileo_quote_latency_ms{strategy}`、`galileo_opportunity_detected_total{strategy}`、`galileo_opportunity_profit_lamports{strategy}`。
  - 指令装配：`galileo_assembly_pipeline_total{base_mint}`、`galileo_assembly_decorator_total{decorator,base_mint}`、`galileo_assembly_compute_units{decorator}`、`galileo_assembly_guard_lamports{decorator}`、`galileo_assembly_instruction_span{base_mint}`、`galileo_assembly_decorator_failures_total{decorator}`、`galileo_assembly_flashloan_total{base_mint}`。
  - 闪电贷：`galileo_flashloan_applied_total{strategy,protocol}`、`galileo_flashloan_amount_lamports{strategy,protocol}`、`galileo_flashloan_inner_instruction_count{strategy,protocol}`。
  - 交易与落地：`galileo_swap_compute_unit_limit{strategy,local_ip}`、`galileo_swap_prioritization_fee_lamports{strategy,local_ip}`、`galileo_transaction_built_total{strategy}`、`galileo_lander_attempt_total{strategy,lander,variant}`、`galileo_lander_submission_total{strategy,lander,variant,result}`、`galileo_lander_success_total{strategy,lander}`、`galileo_lander_failure_total{strategy,lander}`。
  - 资源预检与 IP 管理：`galileo_accounts_precheck_total{strategy,result}`、`galileo_accounts_precheck_mints{strategy}`、`galileo_accounts_precheck_created{strategy}`、`galileo_accounts_precheck_skipped{strategy}`、`galileo_ip_inventory_total{task}`、`galileo_ip_inflight{task,local_ip}`、`galileo_ip_cooldown_total{task}`、`galileo_ip_cooldown_ms{task}`。
  - Copy 工具链：`galileo_copy_queue_depth{wallet}`、`galileo_copy_queue_dropped_total{wallet}`、`galileo_copy_wallet_refresh_total{wallet}`、`galileo_copy_wallet_accounts{wallet}`。
- **Tracing**：使用 `tracing::span!` 在一次套利流程中传播 `trace_id`，Lander 回执后关闭 span。
- **Logging**：结构化日志附带 `slot`, `blockhash`, `dex`, `tip`, `profit`, `lander`。
- **告警**：监控模块支持阈值告警（Quote 失败率、落地失败率、利润分布异常）。

## 7. 性能敏感点与优化策略

- **Quote 优先级**：Quote 是最长耗时环节，Engine 应支持并发批量发起，并在配置中限制最大并行数；避免所有策略共用单队列导致阻塞。
- **无缓存策略**：保持 Quote 实时性，除非后续需要冷启动快照，否则不做任何本地缓存。
- **线程模型**：沿用 README 指导——异步 IO 与 rayon 计算分离；计算密集型逻辑（盈利判断、路径评估）放入 rayon 持久线程池。
- **背压机制**：`try_send` + 丢弃旧任务，Quote 节奏由 `engine.<backend>.quote_config.cadence` 控制，Engine 依据配置限制并发与冷却时间。
- **Blockhash / 账户状态**：通过 DashMap 管理，更新频次与 Engine 解耦；监控中暴露 freshness。

## 8. 配置映射（节选）

| 配置项 | 影响模块 | 描述 |
| --- | --- | --- |
| `blind_strategy.*` / `back_run_strategy.*` | `src/strategy/*.rs` | 策略节奏、利润阈值、禁用状态；策略模块读取后决定是否发起 Quote。 |
| `lander.enable` / `lander.type` | `lander::*` | 选择默认 Lander 实现与启用列表。 |
| `lander.sending_strategy` | `engine::planner` + `lander::stack` | 选择 `AllAtOnce` 或 `OneByOne` 调度策略，决定交易变体数量与分发方式。 |
| `lander.tips` | `engine::builder` + `lander::*` | 配置 tip 账户、优先费；Builder 根据策略覆盖 CU。 |
| `engine.<backend>.quote_config.cadence.*` | `engine::quote_dispatcher` | 配置调度槽位与节奏：`max_concurrent_slots` 决定同时运行的槽位数量，`inter_batch_delay_ms` 为同槽位两次 quote 的间隔，`cycle_cooldown_ms` 为整轮完成后的休息时间，可通过 `per_base_mint` 覆写。 |
| `monitoring.*` | `bot.prometheus` | Prometheus 开关与监听地址；启用后暴露 `/metrics`，指标由 `monitoring::events` 输出。 |

示例：

```yaml
lander:
  sending_strategy: "AllAtOnce"   # 或 "OneByOne"
  jito:
    enabled_strategys:
      - uuid
      - multi_ips
      - forward
    uuid_setting:
      config:
        - uuid: "7dc...966"
          rate_limit: 5
      endpoints:
        - https://ny.mainnet.block-engine.jito.wtf/api/v1/bundles
        - https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/bundles
    multi_ips_setting:
      tips_wallet:
        init_wallet_size: 1000
        auto_generate_interval_ms: 30000
        auto_generate_count: 200
        refill_threshold: 1800
      endpoints:
        - https://ny.mainnet.block-engine.jito.wtf/api/v1/bundles
    forward_setting:
      endpoints:
        - https://jito-worker.example.com/bundles

# 说明
- `enabled_strategys` 控制同时启用的落地策略，`uuid` / `multi_ips` / `forward` 三种可并行尝试，同一变体在 `AllAtOnce` 下会全部发出，在 `OneByOne` 下按 endpoint 拆分。
- `uuid_setting` 与旧版 `uuid_config` 行为一致，额外允许为 uuid 专属 endpoint 列表；若速率冷却中，调度器会跳过本轮发送，避免触发强制包。
- `multi_ips_setting` 会通过钱包池管理临时钱包，主交易先向临时钱包转入 `tip + 0.001` SOL，随后由临时钱包在同一个 bundle 内向官方 Jito tip wallet 转账并归集剩余余额。
- `tips_wallet.refill_threshold` 指定「低于多少个钱包才触发补充」的阈值，避免后台任务无限增生。设置为 `0` 时保持旧行为、每个周期都补充 `auto_generate_count` 个。
- `forward_setting` 支持对接第三方中继，只做原样转发；若 uuid 失败可作为降级路径。
```

## 9. 迭代路线建议

1. **P0**：落地 `lander` 目录与 `Lander` trait，Jito/RPC 实现先行；引擎现有逻辑迁入 `engine::*`。
2. **P1**：重构 `strategy` 目录，拆分各策略并实现统一 `Strategy` trait。
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

  启动后，galileo 会在 `listen` 地址暴露 `/metrics` 供 Prometheus 抓取，外部聚合器仅作为 HTTP 依赖，不再需要单独采集本地进程指标。

- **指标示例（按模块拆分）**：
  - Quote：`galileo_quote_total{strategy,result}`、`galileo_quote_latency_ms{strategy}`、`galileo_opportunity_detected_total{strategy}`。
  - 装配：`galileo_assembly_pipeline_total{base_mint}`、`galileo_assembly_decorator_total{decorator,base_mint}`、`galileo_assembly_compute_units{decorator}`、`galileo_assembly_guard_lamports{decorator}`、`galileo_assembly_flashloan_total{base_mint}`。
  - Flashloan：`galileo_flashloan_applied_total{strategy,protocol}`、`galileo_flashloan_amount_lamports{strategy,protocol}`。
  - 交易 / 落地：`galileo_swap_compute_unit_limit{strategy,local_ip}`、`galileo_swap_prioritization_fee_lamports{strategy,local_ip}`、`galileo_transaction_built_total{strategy}`、`galileo_lander_submission_total{strategy,lander,variant,result}`。
  - 资源：`galileo_accounts_precheck_total{strategy,result}`、`galileo_ip_inflight{task,local_ip}`、`galileo_ip_cooldown_total{task}`。
  - Copy：`galileo_copy_queue_depth{wallet}`、`galileo_copy_queue_dropped_total{wallet}`、`galileo_copy_wallet_refresh_total{wallet}`。

- **抓取与看板**：
  1. 在 Prometheus `scrape_configs` 中新增 job 指向 Galileo 监听端口；聚合器指标可按需在外部系统收集。
  2. Grafana 可基于上述指标构建“机会发现/成功率、Quote 延迟、Lander 成功率”等看板，尤其推荐按 `local_ip` 维度拆分 `galileo_lander_submission_total`、`galileo_swap_compute_unit_limit_bucket` 观察退避效果。结合 Hotpath 报告可快速定位瓶颈。

- **性能注意事项**：仅在配置中启用 Prometheus 时才会初始化 exporter 并真正上报指标；默认关闭时，`metrics` 调用会落入空实现，不影响主套利流程。

## 12. Titan 引擎概览

- 流水会在内存中聚合 Titan 的 `SwapRoute`，自动挑选最佳正反腿路由并评估毛利、tip 与阈值；满足条件后直接将 Titan 返回的指令拼成交易序列，串联现有的 flashloan 与落地逻辑。
- Titan 指令按原始顺序下发：前置 ComputeBudget 指令会提前抽离，其余保持“正向 → 反向”执行顺序；ALT 地址会自动汇总后交给交易构建器。
