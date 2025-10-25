• 目前的多腿链路尚未接入引擎；要把 MultiLegRuntime 真正跑起来，需要对配置、策略调度、引擎执行、落地与观测做成
  套改造。梳理完现状后，我整理出一份分阶段落地的具体方案，建议按顺序推进：

  总入口与配置

  - galileo.galileo.engine.backend 需固定为 none；新增专门的 multi_leg_strategy 配置块，用来描述 base/quote
    mint、trade sizes、最小收益阈值、允许的腿组合等信息。
  - 在 run_strategy 时，当 multi-leg 启用时禁止盲发/纯盲发并行，切换到新的 run_multi_leg_engine。

  Provider & Runtime

  1. 初始化 RPC、身份（EngineIdentity）和 TransactionBuilder、LanderFactory，沿用现有初始化流程。
  2. 实例化 MultiLegOrchestrator 并按配置注册：
      - Ultra：构造 UltraApiClient，以 /order 结果驱动 LegProvider（仅做 quote+plan，暂不走 /execute）。
      - DFlow：复用现有 DflowApiClient，针对 LegRole 决定 buy/sell。
      - Titan：实现 TitanWsQuoteSource，缓存（input, output, amount, leg）→ stream，按 SwapMode 取最优
        route（ExactIn 取最大 out、ExactOut 取最小 in），用 notify_waiters+超时控制并发。
  3. AltCache + Arc<RpcClient> + orchestrator 封装成 Arc<MultiLegRuntime>，并暴露 orchestrator 的 buy_legs()/
     sell_legs() 方便后续枚举。

  策略调度

  - 新增 MultiLegStrategy：沿用 StrategyContext 的 tick/轮询逻辑，把 ready 的 trade size 打包成
    MultiLegTask（包含 pair、amounts、min_profit、tag、允许的 leg 组合）。
  - StrategyEngine 保持泛型，通过 Action::MultiLeg 分支调用 process_multi_leg_tasks。

  Engine 执行流程

  1. 在 StrategyEngine 中持有 MultiLegEngineContext：
      - runtime: Arc<MultiLegRuntime>
      - leg_defaults: HashMap<AggregatorKind, LegContextDefaults>（wrap_and_unwrap/dynamic CU limit 等）
      - allowed_pairs: HashMap<(Pubkey, Pubkey), Vec<LegPairDescriptor>>
      - slippage_bps、fee_account
  2. process_multi_leg_tasks 逻辑：
      - 根据 runtime 的腿集合构造 PairPlanRequest（买腿 QuoteIntent 用输入 token，卖腿用输出 token，slippage
        取配置；LegBuildContext 里写入 payer、compute_unit_price、wrap/dynamic flag）。
      - 通过 plan_pair_batch_with_profit 获得 PairPlanBatchResult，对 successes 逐个计算 gross_profit（已扣
        prioritization fee）。
      - 复用 ProfitEvaluator 增加 evaluate_multi_leg(gross_profit, min_profit_override)，得到 (tip, profit)，
        筛出净收益最高的候选。
      - build_multi_leg_bundle：合并 compute budget 指令、主指令、ALT 列表（去重）。
      - 将 bundle 封装成新的 SwapInstructionsVariant::MultiLeg（需要在 aggregator 模块新增变体并实现
        compute_unit_limit/flatten_instructions/address_lookup_table_addresses 等接口）。
      - 调 TransactionBuilder::build_with_sequence（override 为 compute_budget + main instructions），沿用现有
        variant_planner + LanderStack::submit_plan 完成落地；dry-run 模式直接跳过提交。
  3. 失败路径：对 PairPlanFailure 记录 warn + metrics，runtime cache 失败原因便于 debug。

  监控与 tracing

  - 在 monitoring::events 新增多腿指标：
      - galileo_multi_leg_quote_total{result=success/failure,leg}、
        galileo_multi_leg_plan_total{result=success/failure}。
      - galileo_multi_leg_profit_lamports、galileo_multi_leg_tip_lamports 直方图。
  - tracing：multi_leg::runtime 已有基础日志，再补 info!(target="engine::multi_leg", ...) 标识 pair/amount/选
    中的腿。
  - 文档：更新 docs/ultra_arbitrage/overview.md、roadmap.md，解释新的配置块、调度策略、Prometheus 指标。

  测试建议

  1. 单元测试：
      - TitanWsQuoteSource 使用 mock stream 验证 ExactIn/ExactOut 选择逻辑、超时。
      - build_bundle 确保指令/ALT 去重正确。
      - ProfitEvaluator::evaluate_multi_leg 阈值与 tip 逻辑。
  2. 集成测试（可放 feature flag 下）：
      - fake providers：构造固定 quote，验证 process_multi_leg_tasks 能生成交易、tip、排序。
      - dry-run 架起 runtime + builder（使用 solana_sdk::signature::Keypair::new()）验证整链路不 panic。
  3. 文档里的 dry-run 步骤（cargo run --features=hotpath -- strategy --dry-run）要覆盖 multi_leg_strategy，确
  ———

  下一步建议

  1. 优先落地 Provider & Runtime 与引擎联通（步骤 1~3），跑通 dry-run 流程。
  2. 再补 ProfitEvaluator/Tip + SwapInstructionsVariant::MultiLeg 及落地逻辑。
  3. 最后完善监控与测试。

  目前代码库还未改动上述核心逻辑；若认可这套拆分方案，可以按阶段执行，或先确认 multi_leg_strategy 的配置结构
  （例如：是否需要对白名单腿组合、不同 trade size 的专属标签等）再动手。若需我继续实现，麻烦明确：

  - Multi-leg 的配置 schema（字段、默认值、是否需要 per-pair leg 约束）。
  - Titan WS 的默认超时时间/并发策略是否有特殊要求。
  - 是否要首批就串联 Jito bundle，还是先以 RPC lander 为主。

  确认后我会接着从 Provider 接入开始实现。