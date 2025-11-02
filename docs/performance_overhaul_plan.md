# Galileo 性能冲刺方案

## 背景
- 运行时重构已完成分层拆解，但指令装配与缓存策略仍沿用迁移期实现，存在不必要的 Vec 克隆与内存分配。
- 参考 `/home/go0p/code/rust/sol-trade-sdk` 的流水线设计：以中间件链 + SmallVec/缓存组合大幅降低热路径开销，具备直接套用价值。
- 当前阶段已获准放弃兼容旧实现，可对现有模块进行破坏性重构，目标是在保持抽象优雅的前提下压缩延迟与内存占用。

## 目标
1. **热路径零拷贝**：装配链路中所有指令拼接/拆分过程消除 `Vec::clone`，统一改为 SmallVec 或一次性 `Vec::with_capacity`。
2. **缓存命中优先**：对 ComputeBudget、WSOL、ATA 派生等常用原语引入 DashMap + SmallVec 缓存，CPU 热点不再重复构造。
3. **装配流水线再设计**：将 `InstructionBundle` 拆成轻量结构，指令段落改为 SmallVec/InlineArray，并提供“消费型” API。
4. **Context 连续内联**：`AssemblyContext`、多腿 orchestrator、Flashloan 装饰器全部使用 `#[inline(always)]` 与小型结构，减少栈拷贝。
5. **指标 & 热区分析**：增加 `hotpath::measure_block!` 包裹关键函数，配合 flamegraph 回归基线，确保优化收益可观测。

## 关键改造项
### 1. 指令缓存层
- 新增 `engine/assembly/cache::{compute_budget, wsol, ata}` 模块。
- 使用 `DashMap<CacheKey, SmallVec<[Instruction; 3]>>` 缓存 ComputeBudget / WSOL 指令组合。
- `AssemblyContext` 与装饰器仅引用缓存副本，不再内部构造指令。

### 2. `InstructionBundle` 全面改写
- 采用 `SmallVec<[Instruction; 4]>` 存储 compute/pre/post 段；主指令段在从外部迁入时直接“夺取”所有权。
- 引入 `into_vec()` / `append_into()` 等消费型 API，取代 `flatten()`/`replace_instructions()` 的拷贝实现。
- 统一 ALT 元数据搬移接口，保持零拷贝。

### 3. 中间件链性能优化
- `Decorator::apply` 统一返回 `AssemblyOutcome`，内部通过 `#[inline(always)]` 提示编译器展开。
- 对 Flashloan、Lighthouse、GuardBudget 等装饰器引入 SmallVec 缓存，改写成“读缓存 → 填充参数 → append”的流程。
- `ComputeBudgetDecorator` 直接调用缓存层；`GuardBudgetDecorator` 仅计算数值，不再修改指令 Vec。

### 4. 多腿与聚合引擎内存策略
- `SwapInstructionsVariant` 已压缩为 `enum { Dflow(..), Ultra(..), MultiLeg(..), Kamino(..) }`，统一暴露 `into_segments()` 供 bundle 零拷贝消费。
- Multi-leg orchestration 合并阶段使用 `Vec::with_capacity(total_len)` + `extend_from_slice`，杜绝多次 reallocation。
- Ultra/Kamino 解码模块针对 base64/bincode 解码复用缓冲区，避免重复分配。

### 5. 观测体系
- 在 builder、装配链、multi_leg runtime 的关键步骤包装 `hotpath::measure_block!`，预置 `hotpath` feature。
- 新增 `docs/perf_baseline.md` 记录优化前后基线（延迟、指令数、内存占用）。

## 工作分解
1. **缓存框架搭建**：实现 compute budget/WSOL 缓存接口，接入装饰器。
2. **InstructionBundle 重写**：调整内部结构与消费 API，迁移三条策略路径（swap/blind/multi-leg）。
3. **Variant & Orchestrator 调整**：为 `SwapInstructionsVariant`、Multi-leg orchestrator 提供所有权传递接口。
4. **辅助工具优化**：ATA/WSOL 相关辅助方法同步改为 SmallVec + 缓存。
5. **指标与文档**：补充 hotpath 标注、记录性能基线及测试指引。

## 破坏性变更声明
- 迁移过程中可能删除/改名现有公共函数或结构体；不保留旧接口的兼容层。
- 装饰器链、bundle 接口将全面更新，旧调用点必须同步调整。
- 若现有测试依赖原 API，需一并改写；目标是最终形态的实现而非阶段性兼容。

## 验收标准
- `cargo fmt && cargo check --features hotpath` 通过。
- 覆盖 swap/blind/multi-leg 流程的性能基线：延迟、CU 预算命中率与内存分配次数低于当前版本。
- 指令装配过程中无 `Vec::clone`/`extend` 复制整段指令的热点。
- 文档中明确记录性能测试方法及观测指标。

## 进度同步
- ComputeBudget 与 ATA 缓存已合入主线，装配链双向消费 `InstructionBundle::into_flattened()`。
- `instructions::wsol` 模块已准备好内部复用的 wrap/unwrap 缓存，但目前仅在自建路径中待接入。
- `DecoratorChain::apply_all` 改为函数级 `hotpath::measure` 标注，装配流水线延迟纳入性能观测体系。
- Multi-leg orchestration 改为所有权传递：腿计划的指令与 ALT 直接搬入 bundle，避免 assemble 阶段的 Vec 克隆。
