# 交易解包与拼装细节

## Ultra `/order`
1. **解码**：`UltraLegProvider` 内部调用 `engine::multi_leg::transaction::decoder::decode_base64_transaction` 将 base64 交易解析成 `VersionedTransaction`，同时将原始交易写入 `LegPlan.raw_transaction`，便于后续对 ALT / 优先费二次处理。  
2. **指令清理**：Provider 内部遍历 `VersionedMessage`，按 Program ID 拆分 ComputeBudget 指令并置入 `compute_budget_instructions`，其余 swap/setup/cleanup 指令保留在 `LegPlan.instructions`。若 Ultra 返回带 ALT 的 v0 交易，会将 lookup 表地址写入 `LegPlan.address_lookup_table_addresses`，由 runtime 利用 ALT 缓存自动拉取并还原指令账户；Ultra 默认附带的 `SetComputeUnitLimit/Price` 和 1000 lamports tip transfer 在这里会被剥离，仅保留纯粹的路由指令。  
3. **附加元数据**：保留 `prioritization_fee_lamports`、`routePlan`、`requestId` 等字段，为收益评估与监控打点提供上下文。

## DFlow 提供方（已实现）
1. **报价**：`DflowLegProvider` 根据 `QuoteIntent` 调用 `/quote`，使用 `DflowQuoteConfig` 自动处理 `slippage_bps`、`only_direct_routes`、`max_route_length`。  
2. **指令获取**：结合 `LegBuildContext`（payer、CU price、fee account 等）构造 `/swap-instructions` 请求，复用现有 `DflowApiClient`。  
3. **计划输出**：生成 `LegPlan`，包含：
   - `compute_budget_instructions`（单独存放，便于去重）
   - 主体交易指令（setup → swap → cleanup → other）
   - ALT 地址列表（若 DFlow 提供）
   - `prioritization_fee_lamports`、`blockhash`

## Titan 提供方
1. **报价转换**：`TitanLegProvider` 通过注入的 `TitanQuoteSource`（WS 推流或其他来源）获取 `SwapRoute`，并将 Titan 的指令/ALT 列表转为标准 `LegPlan`。  
2. **指令生成**：Titan 指令集直接转换为 `Instruction`，ComputeBudget 指令被拆分到 `compute_budget_instructions`；Titan 默认为买腿，需要 orchestrator 为其匹配卖腿（如 DFlow）。

## Orchestrator 骨架（新增）
- `engine::multi_leg::orchestrator` 当前支持注册任意腿提供方、按 `buy/sell` 分类，并在同一接口下触发 quote→plan。  
- `available_pairs()` 返回所有潜在买卖腿组合，便于在初始化阶段观测覆盖范围；组合执行时直接由 runtime 统一调度。
- `plan_pair()` 将先获取买腿计划，再以买腿“保底产出”(`min_out_amount`) 自动调整卖腿报价规模，保证两腿规模精准匹配。
- `MultiLegRuntime::plan_pair_batch_with_profit` 会为每个组合填充 ALT、重建指令账户，并计算收益；结果按净收益排序并带上批次 tag，同时保留失败列表进行监控。
- Titan 腿在 runtime 层具备并发上限 + debounce 控制，防止 WS 推流被突发请求压垮。

## 指令合并策略
### 单交易模式
- 收集每条腿的 `LegPlan`：  
  1. 合并并去重 ComputeBudget；  
  2. 按腿顺序插入主体指令；  
  3. 汇总 ALT 地址，交由 `TransactionBuilder` 拉取具体账户；  
  4. 设置优先费/赞助费。
- 利用 `LegPlan.quote` 中的 in/out、保底 out、slippage 等数据，在合并阶段即可完成收益预估与最小收益校验。
- 最终调用现有 `TransactionBuilder` 生成 `VersionedTransaction`，并复用 Engine 的利润/调度/监控逻辑。

### Jito bundle 模式
- 将各腿计划拆分为独立交易（若不易合并），并附加 tip 交易。  
- 使用 LanderStack 内的 Jito 落地器提交 bundle，沿用既有状态跟踪与告警。

## 风险控制
- **账户冲突**：组合腿时检查 ATA 读写权限，必要时在合并阶段清理重复指令。  
- **超时**：尊重腿内 `expiresAtMs`/`blockhash`，过期立即丢弃并重新拉取报价。  
- **滑点**：沿用现有 `ProfitEvaluator`，在 `LegPlan` 中保留 quote 数据评估风险。  
- **回滚**：Jito bundle 或交易失败时落盘 `LegPlan` 与 quote 元数据，触发监控告警。
- **Titan 推流**：借助 runtime 内建的限流 + debounce，保障 Titan WS 连接在多规模并发扫描场景下仍然稳定。
