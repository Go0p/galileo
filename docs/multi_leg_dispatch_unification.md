# Multi-Leg Dispatch Unification Plan

## 背景
当前多腿模式在调度层仍走 `process_quote_batch_legacy`，导致：
- Quote 批次串行执行，`max_concurrent_slots` / `inter_batch_delay_ms` / `cycle_cooldown_ms` 等节奏参数无法生效。
- 只能同时持有 1 买 + 1 卖 IP，无法充分利用多 IP 并发。
- 监控与日志路径和单腿截然不同，维护成本高。

目标：让多腿任务在“整体调度与节奏控制”上与单腿引擎一致，只在内部执行阶段体现“买腿 + 卖腿”组合。

## 设计要点

### 1. 统一批次调度入口
- 重新实现 `run_quote_batches` 的 multi-leg 分支，使其同样调用 `QuoteDispatcher::dispatch`。
- Multi-leg 批次被视为“特殊 quote 任务”，调度器负责：
  - 依据 `max_concurrent_slots` 管理活跃槽位数量。
  - 按槽位顺序发射批次（遵守 `inter_batch_delay_ms` / `cycle_cooldown_ms`）。
  - 申请 / 释放 quote 用 IP 租约。
  - 收集执行结果并回传 outcome。
- 调度器无需理解多腿细节，具体落地由新 `MultiLegBatchExecutor` 负责。

### 2. MultiLegBatchExecutor
- 输入：`QuoteBatchPlan`（pair、amount、batch_id）。
- 步骤：
  1. 根据配置组装单个“买腿 + 卖腿”组合（固定 1:1）。
  2. 调用 `MultiLegRuntime::plan_single_pair(request)`，执行：
     - 买/卖腿 quote（并发）。
     - Quote-level 预筛选。
     - 并发构建 swap 指令。
     - ALT 填充。
     - 单一净收益计算（不再需要 rayon 排序）。
  3. 返回 `QuoteDispatchOutcome` 等价结构（带上利润、指令、监控信息）。
- 失败、超时在这里统一处理，调度器只接收成功/失败标识。

### 3. 收益评估精简
- 多腿仅支持一个组合，所以：
  - Quote 阶段计算 `gross = sell_out - buy_in`；若 <= 0 直接返回失败。
  - Build 阶段成功后再扣 prioritization fee，生成最终 `net_profit`。
  - 不再使用 rayon 线程池排序，直接返回单个 `SwapOpportunity`。

### 4. 监控与日志
- 复用单腿的 console summary / metrics：调度器记录 quote 用时、成功率。
- Multi-leg executor 内保留细粒度日志（quote 阶段、plan 阶段）。
- 失败路径需额外打点区分 quote 段 vs plan 段，方便回溯。

### 5. 配置与兼容
- Multi-leg 仍由配置决定是否启用；一旦开启：
  - Quote 批次可以像单腿一样设置 `max_concurrent_slots`、`inter_batch_delay_ms`、`cycle_cooldown_ms`。
  - IP allocator 自动满足多并发租约。
- 保持 `multi_leg` runtime 初始化流程不变，只调整调度消费方式。

## 实施步骤
1. **调度改造**
   - 让 multi-leg 分支使用 `QuoteDispatcher::dispatch`。
   - 增加 `MultiLegBatchExecutor` 适配器，接管实际执行。
2. **Runtime 调整**
   - 新增 `plan_single_pair`，封装 quote → plan → populate → profit。
   - 精简 `PairPlanBatchResult` / `evaluate_pair_plans` 逻辑。
3. **策略层结果处理**
   - 修改 outcome 解析，接收 multi-leg executor 返回的机会。
   - 保持策略后续处理（profit evaluator、落地）接口不变。
4. **文档 & 回归**
   - 更新 `docs/multi_leg_swap_parallel_plan.md`，说明调度统一化。
   - Dry-run 验证多 IP 并发、节奏参数生效。

## 预期收益
- 多 IP 同时运行成为默认能力，链路性能随 IP 数线性提升。
- 调度与监控只维护一套逻辑，降低维护成本。
- Quote 阶段就能享受现有节奏/节流机制，避免爆发式请求。

## 实现简述
- 新增 `QuoteDispatcher::dispatch_custom`，在保持 spacing / 并发控制的前提下允许自定义批量执行器。
- `MultiLegBatchHandler` 基于 `MultiLegEngineContext` 生成 `PairPlanRequest`，复用 `plan_pair_batch_with_profit` 产出计划与失败信息。
- `StrategyEngine::handle_multi_leg_batch` 负责利润评估、监控打点与最终落地，单腿/多腿共用同一执行路径。
