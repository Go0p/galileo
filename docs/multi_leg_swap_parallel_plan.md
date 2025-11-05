# Multi-Leg Quote/Swap Pipeline Refactor Plan

## 背景
- 当前 multi-leg 组合在 `MultiLegOrchestrator::plan_pair` 中，对每条腿执行一次 `quote -> swap_instructions` 串行流程。
- `LegProviderAdapter::plan` 会立即执行 `quote` 与 `build_plan`，导致在收益未确认时就请求 `/swap/v1/swap-instructions`。
- Jupiter / DFlow 的 swap 指令生成平均需要 700–1200ms，一轮套利需要两次调用，串行加总放大尾延迟，引发超时告警。
- 最近观测到 DFlow 报价偶发超时，进一步暴露当前流程中缺乏并发与节流策略的问题。

## 现状流程
1. `MultiLegOrchestrator::plan_pair` 先以买腿 `QuoteIntent` 调用 provider：
   - `quote_with_ip` 请求 `/quote`。
   - 紧接着 `swap_instructions` 请求 `/swap/v1/swap-instructions`。
2. 取买腿 `amount_out` 作为卖腿输入，重复执行一次 `quote -> swap_instructions`。
3. 只有在拿到双腿 `LegPlan` 后，才进入 profit 评估与排序。

### 主要问题
- **指令请求冗余**：即便后续发现利润不足，也已经向两侧 aggregator 拉取了一遍 swap 指令。
- **延迟串行**：总耗时 `t_buy_swap + t_sell_swap`，高于实际需求的 `max(t_buy_swap, t_sell_swap)`。
- **资源占用**：IP 租约在第一次 swap 期间被占用，导致下一条腿排队；连接池负载增大。
- **监控噪声**：超时 WARN 难以区分是 quote 阶段还是 swap 阶段引起。

## 目标
1. 在确认利润前只执行报价，避免不必要的 swap 指令请求。
2. 在需要构建执行计划时，并发拉取买/卖腿 swap 指令，缩短总耗时。
3. 保持现有 `LegPlan` 数据结构和调用方接口兼容，降低迁移风险。
4. 为后续引入重试、节流、缓存等机制打基础。

## 拟议架构调整
### 接口改造
- 将 `LegProvider` 拆分为两个阶段：
  ```rust
  async fn quote(&self, intent, lease) -> Result<QuoteResponse>;
  async fn build_plan(&self, quote, context, lease) -> Result<LegPlan>;
  ```
  - 现有 `build_plan` 逻辑迁移至第二阶段。
  - `LegPlan` 保留 prioritization fee、compute limit 等 swap 数据。
- 更新 `DynLegProvider` 适配层，分别暴露 `quote_only` 与 `build_plan`.

### Orchestrator 调整
1. `plan_pair` 改为：
   - 先串行获取买腿 quote，再根据 amount_out 请求卖腿 quote。
   - 在 quote 阶段产出 `LegPairQuote { buy_quote, sell_quote }`。
2. 在确定利润需落地时（例如批量评估 best routes）：
   - 同时对买卖腿调用 `build_plan`，使用 `try_join!` 并发执行。
   - 需要时支持超时/取消，从而释放 IP 租约。
3. 新增快速利润估算：只基于 quote 数据计算预期收益，将 prioritization fee 估算或延迟扣除。

### IP 与租约
- 拆分阶段后，quote 阶段结束即可释放或复用租约。
- build 阶段申请 Lease 时可改为 `IpLeaseMode::Scoped { guard }`，确保并发 swap 时租约维持到指令返回。

## 实施步骤
1. **接口设计与适配**
   - 定义新的 `LegProvider` trait 方法；提供默认实现以兼容现有 provider。
   - Jupiter / DFlow provider 先实现新接口，功能等价于旧版。
2. **Orchestrator 重构**
   - 引入 `LegPairQuote`、`LegPlanRequest` 等中间结构。
   - `plan_pair` 改为只负责 quote；新增 `build_pair_plan` 用于并发 swap。
3. **Runtime 调整**
   - `plan_pair_batch_with_profit` 先收集所有 `LegPairQuote`，筛选 Top N，再并发调用 `build_pair_plan`。
   - 为 quote 阶段和 swap 阶段分别记录监控指标。
4. **回归与验证**
   - 单元测试：对 orchestrator/provider 添加新流程测试。
   - 集成测试： dry-run 跑多条腿，验证收益排序、指令完整性。
   - 基准测试：记录重构前后 quote→swap 总耗时、成功率、WARN 数。
5. **逐步上线**
   - 默认通过 feature flag 控制：先在 dry-run 环境启用新流水线。
   - 监控稳定后再在生产配置开启。

## 度量与监控
- **延迟指标**：拆分 quote/swap 指标，监控平均与尾延迟。
- **请求量**：比较 swap-instruction 调用量是否下降。
- **利润命中率**：确认调整后真实成交量及利润率未下降。
- **错误分布**：检查 timeout / rate limit 告警是否收敛。

## 风险与缓解
- **Quote 过期**：并发 swap 前 quote 可能失效 → 在 `LegPairQuote` 中记录 `expires_at_ms`，提前校验窗口。
- **Prioritization fee 偏差**：估算不准会导致误判 → 在 pipeline 后期重新扣除实际 fee，如果收益跌破阈值则放弃。
- **兼容性**：调用方期望同时拿到完整 `LegPlan` → 保持最终输出结构不变，对外 API 仅改变内部调度。
- **复杂度提升**：引入更多状态和异步组合 → 通过单元测试、日志标注（quote阶段/plan阶段）降低排障难度。

## 开放问题
- 是否需要在 quote 阶段就为 swap 预申请 IP 资源？（防止 build 阶段拿不到租约）
- 对 Titan / Ultra 等其它 aggregator 是否沿用同样流程，还是逐个改造？
- 当并发 swap 有一侧失败时，是否要回退另一次请求结果还是缓存以备后用？

---
负责人：待指定  
目标上线：待评估（建议两周内完成接口重构 + 并发原型验证）
