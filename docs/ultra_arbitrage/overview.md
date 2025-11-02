# 多腿组合方案概览

> 目标：在现有 Galileo 引擎“一聚合器双腿”能力的基础上，构建可复用的多聚合器腿组合层（`src/engine/multi_leg/`），最终通过 Jito bundle 实现原子落地。

## 当前进度与基础能力
- `engine::QuoteExecutor` / `engine::swap_preparer::SwapPreparer` 已封装单聚合器双腿流程，可继续为各腿 provider 所复用。
- `TransactionBuilder`、`ProfitEvaluator`、`Scheduler` 等引擎组件已经稳定运行，无需为多腿方案重写。
- 新增 `engine.backend = none` 配置，允许策略在无单聚合器后端的前提下运行纯盲发模式，为多腿 orchestrator 接管报价/落地链路铺路。（当前纯盲发仍是唯一策略入口）
- `engine::multi_leg::providers::dflow`、`engine::multi_leg::providers::ultra`、`engine::multi_leg::providers::titan` 均已落地：
  - Ultra provider 会解码 `/order` 返回的 base64 交易，拆分 ComputeBudget 指令并保留原始 `VersionedTransaction`，暂仅支持无 ALT 的 v0/legacy 消息。
  - Titan provider 通过抽象 `TitanQuoteSource` 将 WS 报价映射为标准 `LegPlan`，默认仅承担买腿。
- `LegPlan` 新增 `quote` 元数据（实际 in/out、保底 out、slippage、request_id 等），统一提供收益评估所需的上下文。
- `engine::multi_leg::orchestrator` 在腿注册后自动完成配对、规模对齐与收益计算：买腿 `min_out_amount` 会传递到卖腿的 `QuoteIntent`，再结合 `plan_pair_batch_with_profit` 的收益降序排序筛出最优组合。
- 引入 `engine::multi_leg::alt_cache` 与 `engine::multi_leg::runtime`：支持在构建腿计划后批量获取 ALT（`getMultipleAccounts` + 缓存），并在 orchestrator 内部并发规划买/卖腿，降低 Quote→Plan 延迟。
- `MultiLegRuntime` 已对关键流程加固：
  - `plan_pair_with_alts` 自动以买腿保底产出调整卖腿规模，确保两腿严格对接。
  - `plan_pair_batch_with_profit` 支持多 trade size 并发规划并返回收益降序列表，可直接输送给收益评估。
  - Titan 推流入口集成内置限流 / 防抖（默认 2 并发、200ms debounce），避免 WS 过载。
- `StrategyEngine` 已在 `engine.backend = multi-legs` 模式下直接调用 `MultiLegRuntime`，复用盲发调度生成
  PairPlan 请求，并在选择最优收益后通过 `TransactionBuilder` + `LanderStack` 完成落地（dry-run 下仍保留
  全量构建链路）。

## multi_leg 模块新增内容
- **类型抽象**：`LegSide`、`QuoteIntent`、`LegBuildContext`、`LegPlan`、`LegDescriptor` 描述腿角色、报价意图与输出计划。
- **交易工具**：`transaction::decoder` 负责 base64 ↔ `VersionedTransaction` 转换，ComputeBudget 拆分/指令去重逻辑现已集成到各腿 provider 与装配阶段，避免重复维护辅助模块。
- **提供方接口**：`LegProvider` trait 统一定义腿的报价 + 指令构建流程，现已提供 DFlow/Ultra/Titan 三个实现，便于 orchestrator 直接组合。
- `LegPlan` 新增 `raw_transaction` 字段，Ultra 提供方会将未签名交易完整保留，后续 orchestrator 可在准备阶段处理 ALT 或重新拼装。
- **运行时封装**：`MultiLegRuntime` 将 orchestrator、AltCache、RPC 结合，提供 `plan_side_with_alts/plan_pair_with_alts` 等接口，为后续策略接入（engine.backend=none）铺路。

## 聚合器角色说明
- **DFlow**：仍支持独立的双腿套利；当在 `galileo.yaml` 中配置 `leg: buy/sell` 时，可按角色暴露给 multi-leg 组合层。`DflowLegProvider` 已落地。
- **Ultra**：需要与其他腿组合，`/order` 返回的未签名交易已由 `UltraLegProvider` 解码并清理 ComputeBudget 指令；若响应包含 ALT，会记录 lookup 表地址并由 runtime 通过 ALT 缓存自动拉取与还原指令账户。Ultra 自带的 CU limit/price、tip 会被剥离，由合并阶段统一计算总 CU 限制与优先费。
- **Titan**：只能承担买腿，`TitanLegProvider` 基于可注入的报价源（WS/MPC）生成标准指令集合；Titan 报价不会提供 `/swap-instructions`，需与其他卖腿（DFlow 等）拼装。

## 配置映射
- `galileo.yaml` 中每个聚合器新增 `enable` + `leg` 字段；未配置 `leg` 的聚合器保持原有行为，不参与多腿组合。
- 当存在至少一条 `buy` 与一条 `sell` 腿且均启用时，multi_leg orchestrator 将尝试组合；仍可保留 DFlow 独立套利作为兜底。
- Titan 固定 `leg: buy`，Ultra/DFlow 可配置 `buy` 或 `sell`。

## Multi-Legs 引擎入口
- 新增 `galileo.engine.backend = "multi-legs"`，由盲发策略照常提供 base/quote mint 与 trade size，engine 层负责 orchestrator 落地。
- backend=multi-legs 时：
  - 读取 `engine.{ultra,dflow,titan}`，实例化至多两个 provider（Titan 固定 buy，Ultra/DFlow 按 `leg` 字段决定方向）。
  - 提前初始化 `MultiLegRuntime`（ALT 缓存 + Titan 流量限制），记录可用腿信息，后续迭代将把 runtime 接入策略主循环。
  - 继续复用盲发配置生成的 trade pair/size，零成本抽象原则保持不变。

目前 multi-legs 已完成策略主循环与落地链路串联，后续工作重点转向指标补齐、回归测试与多腿指令的
性能调优。

## 下一步流程（高层）
1. **腿发现**：解析配置 → 实例化各聚合器的 `LegProvider`（已完成 DFlow/Ultra/Titan）。  
2. **腿报价**：使用 `QuoteIntent` 调用 provider 的 `quote`，沿用原有超时/指标逻辑。（待接入策略）  
3. **腿计划构建**：根据 `LegBuildContext` 获取 `LegPlan`（指令 + ALT + prioritization）并补齐 `LegQuote`。  
4. **组合与落地**（进行中）：`MultiLegRuntime` 已能枚举腿组合并给出收益排序，下一步需要把成功计划交给 `TransactionBuilder`/Jito 完成落地。  
5. **观测与文档**：延续现有 metrics/tracing 体系，补充多腿维度指标与 Titan 推流观测。

## 设计原则
- **复用优先**：尽可能复用 engine 层现有实现，multi_leg 只负责腿抽象与组合协调。
- **配置驱动**：所有角色与特殊参数由配置文件控制，避免硬编码。
- **透明观测**：继续使用 `hotpath::measure` 与统一指标前缀，新增多腿相关 metrics。
- **原子安全**：组合腿后默认采用 Jito bundle，无法保证原子性时禁止落地产线。  

## 待完成事项
- 将 `MultiLegRuntime` 纳入 `engine.backend = none` 策略路径，依据配置注册 buy/sell provider，并把 `plan_pair_batch_with_profit` 结果送入现有调度器。
- 复用 `ProfitEvaluator`、tip 计算、失败重试与风险阈值，确认 multi-leg 输出满足最小收益 / 账户检查。
- 串联 `TransactionBuilder` 与 Jito Lander，完成 bundle 构建、落地与收益打点，补齐 Prometheus 指标 / tracing。
- Titan 推流源头尚未接入实际 WS 订阅管理，需要结合 runtime 防抖策略完成来源实现、回收机制与可观测性。
