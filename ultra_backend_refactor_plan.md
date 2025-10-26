# Ultra Backend 重构总体方案

## 目标
- 引擎层对外仅接收统一的“可执行交易”抽象，屏蔽各聚合器的实现细节。
- 支持 Ultra 作为独立 backend，同时在 multi-leg 模式下既能做买腿也能做卖腿。
- 复用现有计算/监控能力，最小化对策略层与落地层的侵入。

## 核心设计

### 1. 统一机会与交易描述
- **ExecutionOpportunity**：保留套利必备字段（`pair`、`amount_in`、`expected_out`、`profit`、`tip`、`metadata`），去除 QuotePayload 依赖。
- **PreparedSwap**：统一封装落地所需的 `instructions`、`lookup_tables`、`compute_budget`、`prioritization_fee`、`raw_transaction` 等信息。

### 2. SwapPreparer Trait
```rust
pub trait SwapPreparer: Send + Sync {
    async fn prepare(
        &self,
        opp: &ExecutionOpportunity,
        ctx: &EngineContext,
    ) -> EngineResult<PreparedSwap>;
}
```
- **JupiterPreparer**：沿用现有 `swap_instructions` 调用链。
- **DflowPreparer**：保留限流、失败重试策略。
- **UltraPreparer**：解析 `/order` base64 交易，利用 ALT 缓存与 ComputeBudget 归并。
- 后续新增 provider（Titan 等）只需实现 `SwapPreparer`。

### 3. ALT 与交易解码下沉
- 将 `multi_leg::alt_cache`、`transaction::instructions` 等工具迁移到共享模块（如 `engine::prepared`）。
- PreparedSwap 总是带齐需要的 lookup 表；TransactionBuilder 只担补齐、回退逻辑。

### 4. ProfitEvaluator 解耦
- 仅负责收益评估，返回 `ExecutionOpportunity`；不再生成合并后的 QuotePayload。
- 引擎主流程：`OpportunityEvaluator → SwapPreparer → TransactionBuilder → Lander`。

### 5. 闪电贷与监控
- PreparedSwap 提供 `compute_unit_limit` / `prioritization_fee`；TransactionBuilder 叠加 Flashloan overhead。
- 新增监控事件：`events::swap_prepared(provider, result)` 等，覆盖 Ultra/Jupiter/DFlow。

### 6. Multi-leg 一致性
- `LegPlan` 内部复用 `PreparedSwap`；`assemble_multi_leg_instructions` 直接合并各腿的 prepared 数据。
- Ultra 可通过 `ultra.legs = ["buy", "sell"]` 同时注册两侧腿。

## 实施步骤
1. **抽象层搭建**  
   - 引入 `PreparedSwap`、`SwapPreparer`，调整 TransactionBuilder 接口。
   - 将现有 `SwapInstructionFetcher` 拆分为 Jupiter/DFlow 两个 preparer。
2. **机会结构重塑**  
   - 重写 ProfitEvaluator → `ExecutionOpportunity`；改造 `StrategyEngine` 使用新结构。
3. **Ultra 接入**  
   - 下沉 multi-leg Ultra 的 ALT/解码工具，完成 `UltraPreparer`。  
   - 支持在 multi-leg 中配置 `LegRole::Sell`。
4. **CLI & Backend**  
   - 扩展 `StrategyBackend`、`AggregatorContext`，新增 Ultra 分支与命令。
   - 更新监控、日志、文档。
5. **测试与验收**  
   - 重建单测/集成测试，覆盖 Jupiter/DFlow/Ultra。  
   - dry-run + 实网验证多腿与单腿路径。

## 后续注意
- 删除旧的 QuotePayload 合并逻辑，避免双轨代码。
- 文档同步更新：`docs/strategy_arch.md`、`docs/ultra_*` 等。
- 保留对性能的测量（hotpath 注解），防止重构带来额外延迟。

- 仍待处理：清理旧导出与未使用代码、补全文档/监控、为 Ultra/Multi-leg 路径补测试，以及完成文中提到的指标说明。