• 多腿链路已集成到策略主循环：`StrategyEngine` 会在 `engine.backend = "multi-legs"` 时使用盲发配置生成
  PairPlan 请求，挑选最优收益方案后构造 `SwapInstructionsVariant::MultiLeg`，并复用现有
  TransactionBuilder/LanderStack 以 dry-run 或正式模式执行。所以下面列表聚焦后续补完事项。

  待补 observability

  - 在 monitoring::events 中新增 multi-leg 指标（quote/plan 成功率、profit/tip 直方图），同步更新文档与
    Prometheus 配置。
  - 为多腿执行路径补充 tracing：标记买/卖腿聚合器、trade size、收益与落地结果。

  测试与验证

  - 为 `process_multi_leg_task` / `assemble_multi_leg_instructions` 增加单元测试，覆盖 ALT 去重、收益过滤、
    dry-run 行为。
  - Titan WS / provider 层继续补 mock 测试，确保推流与容错策略稳定。
  - 在 CI 或手动脚本中增加 `backend=multi-legs` 的 dry-run 集成测试，验证构建出的交易可通过模拟落地。

  运行时调优

  - 针对 compute budget/price 指令做去重与合并，规避多腿组合导致的重复配置。
  - 将 multi-leg 特有的指标（按腿/组合维度）纳入告警体系，例如 Titan 推流超时、组合收益长期为负等。

  文档与配置

  - 在文档中追加 multi-leg dry-run 操作示例，以及默认指标看板说明。
  - 如果需要细粒度限制腿组合或每对交易对的最小收益阈值，可考虑在配置中加入白名单/override 描述，并在
    `MultiLegEngineContext` 中使用。
