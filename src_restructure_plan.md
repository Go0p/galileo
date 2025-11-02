> 进展：
> - 运行时分层继续收敛：`engine::multi_leg::transaction` 仅保留 `decoder`/`instructions`，原先用于接线的 `sanitizer` 已移除，ComputeBudget 去重逻辑落入腿 provider 与装配链路。
> - `InstructionBundle` 支持惰性重建 compute budget 指令，新增 `into_flattened` 消费式拼接，三条策略路径不再克隆指令序列。
> - ATA 推导改为 `cache::cached_associated_token_address` 全局缓存，预检查/盲发/多腿 provider 等依赖统一复用，消除重复 PDA 计算。
> - 指令层完成重构：移除了过渡期的 `instructions/jupiter/{accounts,route}.rs`，`lighthouse` re-export 改为只暴露实际消费的符号；Marginfi 闪电贷统一通过 `engine::plugins::flashloan` 顶层导出对外，避免在各处手写 `#[allow]`。
> - 引擎 helper 清理：`aggregator` 删除未使用的 Quote 访问器与 Swap variant kind，`planner` 抛弃 `TipOverride` 及相关接线逻辑，Jito 路径直接复用策略配置；`UltraLegProvider` 不再携带闲置 `swap_config` 字段。
> - 文档同步：Ultra 交易流程改为描述内置拆分逻辑，移除了对 `transaction::sanitizer` 的依赖叙述。
> - WSOL 原语缓存已抽象成 `instructions::wsol`，用于后续内部构造指令；第三方返回的指令保持原样消费。
> - 装饰器链热区标注：`DecoratorChain::apply_all` 使用函数级 `hotpath::measure`，守护链路延迟纳入性能观测。
> - Multi-leg orchestrator 与 runtime 避免指令克隆：`assemble_multi_leg_instructions` 直接搬移腿计划内的指令/ALT 数据，执行阶段以所有权传递拼装 bundle。

> 后续安排：
> - 对 `instructions/jupiter/types` 中的备用构造器进行梳理，确认哪些需要转入测试或高层 API，避免未来再次出现 `#[allow(dead_code)]`。
> - 多腿与装饰器链路需要新增集成测试，覆盖 ALT 缓存共用与 Jito tip 行为，作为下一阶段性能验收的基线。
> - 为 WSOL 缓存补充并发命中与回退测试，记录内部使用场景的性能收益。
