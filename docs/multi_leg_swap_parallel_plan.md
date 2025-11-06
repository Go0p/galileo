# Multi-Leg Quote/Swap Pipeline

截至 2025-11-06，Galileo 的多腿流水线已经完成拆分与并发化改造，当前实现遵循以下流程与约束。

## 核心流程
1. **Quote 阶段只拉取报价**  
   `MultiLegOrchestrator::quote_pair` 会先串行获取买腿 quote，再基于买腿的 `amount_out` 调整卖腿规模并获取卖腿 quote。两个 quote 被封装成 `LegPairQuote`，其中包含标准化的 `LegQuote` 摘要（in/out、阈值、失效时间等）。
2. **收益预筛选**  
   运行时在 quote 阶段即计算 `estimated_gross_profit = sell.amount_out - buy.amount_in`，若结果不正，则直接跳过后续 swap 指令生成，避免无意义的 `/swap-instructions` 调用。
3. **并发构建 Swap 计划**  
   只有当报价通过预筛选时，`MultiLegOrchestrator::build_pair_plan` 才会使用 `tokio::try_join!` 并发构建买/卖腿计划，并在落地前重新校准最小成交量等参数。

## Trait 与类型
- `LegProvider::summarize_quote`：每个 provider 需将原始报价映射为统一的 `LegQuote`。这允许 orchestrator 在不触发 swap 指令的情况下完成收益评估与 TTL 校验。
- `LegQuoteHandle`：保存 `LegQuote` 与 provider 自有的报价上下文，后续可再次调用以生成 `LegPlan`，实现 quote/plan 完整拆分且无动态分发开销。

## 运行时策略
- **IP 租约复用**：quote 阶段与 plan 阶段分别申请/释放 `IpLeaseHandle`，保证关键路径持有时间最小化。
- **Titan 并发控制**：保持原有的限流与节流逻辑，在新流水线上同样生效。
- **ALT 填充**：在 `build_pair_plan` 之后依旧由 `populate_pair_plan` 负责 fetch ALT 与重写指令，接口未变化。

## 观测与告警
- Quote 阶段与 Plan 阶段分别打点，以便定位延迟和失败来源。
- 对于被预筛掉的组合仅记录 debug 日志，不计入失败列表，避免噪声。
- 继续沿用既有的利润评估与排序逻辑，最终收益会在 `LegPairPlan` 上按实际 prioritization fee 扣减。

## 后续优化方向
- 在 quote 阶段补充 TTL / prioritization fee 估算的统计，以便更精确地提前过滤。
- 按需扩展 `LegPairQuote`，支持缓存可重用的报价结果或携带更多监控上下文。

该文档将作为多腿流水线的最新设计说明，旧的串行流程已经完全废弃。
