# Galileo 源码目录重构计划（2024 Q4 更新）

## 总体目标
- 在保持套利链路低延迟与零成本抽象的前提下，重构目录与装配流程，使策略、引擎、交易指令三层职责清晰。
- 用流水线式的指令装配代替散落的 Vec 操作，便于挂载闪电贷、利润守护、Jito Tip 等扩展能力。
- 构建统一的指令原语层，供装配流水线与聚合器特定实现共享。
- 为后续新增策略/落地器/监控指标提供可复用的模板，同时保证配置驱动、观测完备。

## 设计约束
- `trait + 泛型` 优先，避免在关键路径引入 `Box<dyn …>`。
- 全部可调参数来自配置，禁止写死魔法数字。
- 装配流水线应尽可能零拷贝、少锁；compute budget 与 lookup table 处理保持可预估的分支。
- 监控/日志/热路径标注与代码同步更新。
- `instructions/` 层只提供协议无关的指令原语，`txs/` 保留聚合器/DEX 专属拼装，避免互相污染。

## 阶段 0：现状盘点与依赖映射
1. 统计策略、引擎、交易装配的现有模块访问关系：
- `strategy::*` → `engine::{StrategyContext, ProfitEvaluator, SwapPreparer}`。
- `engine::mod` 中 `execute_blind_order` / `execute_plan` / `process_multi_leg_execution` / `process_swap_opportunity` 共有的指令插入逻辑。
- `flashloan::marginfi`、`lighthouse::*` 对装配结果的依赖和回写行为。
- 对比参考仓库 `/home/go0p/code/rust/sol-trade-sdk` 的结构，记录其 `instruction/` + `trading/common` + `trading/middleware` 的职责划分，为后续引入中间件链和指令缓存提供素材。
2. 梳理 CLI 初始化流程：确认配置解析、flashloan/lighthouse/lander 构建点，标注需要拆分的段落。
3. 收集 dry-run、集成测试、benchmarks，以及运行策略所需的 mock 资源，建立重构期间的验证清单。

## 阶段 1：目录重组与策略归档
1. 建立新的策略层级：
   ```
   src/strategy/
     mod.rs            // 仅 re-export
     common/
       mod.rs
       builder.rs      // trade pair 构建、公用工具
     blind/
       mod.rs
       runner.rs
     pure_blind/
       mod.rs
       route_builder.rs
     multi_leg/
       mod.rs
       orchestrator.rs // 现有 orchestrator/runtime 迁移后重命名
     copy/
       mod.rs
       runner.rs
   ```
2. 将 `src/multi_leg/*`、`src/copy_strategy/*`、`src/pure_blind/*` 迁入对应目录，旧路径保留临时 re-export（限定一到两个版本内移除）。
3. 清理策略模块对 `engine` 的直接耦合，公共构建逻辑挪到 `strategy/common`。

## 阶段 2：CLI 装配拆分
1. 拆出 `cli/commands`, `cli/runtime`, `cli/engine_builder`, `cli/resources` 四个子模块：
   - `cli/commands/strategy.rs` 仅处理 clap/命令行。
   - `cli/runtime/mod.rs` 驱动 run/dry-run 入口。
   - `cli/engine_builder.rs` 负责注入 EngineSettings、LanderStack、QuoteExecutor 等。
   - `cli/resources.rs` 针对 flashloan、lighthouse、monitoring、ip allocator 等提供构建器。
2. 约束：CLI 不直接访问策略内部实现，统一通过 `strategy::factory` （可在 `strategy/common` 增设）暴露接口。
3. 重构完成后补充文档，说明 CLI 如何装配不同策略以及如何启用 flashloan / lighthouse。

## 阶段 3：指令装配流水线化
1. 在 `src/engine/assembly/` 引入统一的交易装配模块：
   ```
   src/engine/assembly/
     mod.rs                    // Pipeline trait + 入口
     bundle.rs                 // InstructionBundle/InstructionParts
     decorators/
       mod.rs
       compute_budget.rs       // CU limit / price 装饰器
       flashloan.rs            // marginfi -> Adapter<F>
       lighthouse.rs           // TokenAmountGuard 装饰器
       jito_tip.rs             // prioritization 计算
       custom.rs               // 针对未来扩展
   ```
2. 将现有 `SwapInstructionsVariant::flatten_instructions` 扩展为提供结构化的 `InstructionBundle`（前缀/主体/后缀/lookup tables/prioritization），并为 Jupiter / MultiLeg / Ultra / Kamino 后端实现转换。
3. `StrategyEngine` 内部只负责构造基础 bundle，然后按顺序执行装饰器链（可用零拷贝切片、`SmallVec` 优化），避免在多个函数重复操作 Vec。
4. 闪电贷与 Lighthouse 插件化：`MarginfiFlashloanManager` 改为实现 `InstructionDecorator`，Lighthouse 守护亦然；相关指令原语（借贷流程、MemoryWrite/AccountDelta、ComputeBudget）从 `instructions/` 引入，compute-unit overhead 回写通过统一接口完成。
5. 引入 `hotpath` 标注，确保 pipeline 中关键函数具备性能探针；必要时补充 microbench（criterion 或自研 benchmark）验证无额外拷贝。

## 阶段 4：配置与监控同步
1. 将 pipeline 装饰器可配置项写入 `galileo.yaml` / `galileo.bot.*`，说明如何启用/禁用 flashloan、Lighthouse、Jito Tip。
2. `monitoring::events` 补充装配阶段事件（例如 `instruction_pipeline_start/end`、`decorator_applied`），Prometheus 指标注明标签与预期值。
3. 更新 `docs/strategy_arch.md`、`docs/monitoring.md`、`docs/deploy.md`，同步描述新的目录与流水线。

## 阶段 5：长文件拆分与收尾
1. 将 `engine/mod.rs` 拆分成 `engine/runtime.rs`（事件循环/调度）、`engine/pipeline.rs`（装配）、`engine/context.rs`（保留调度上下文）等，减少巨型文件。
2. 删除阶段 1 的临时 re-export，确保所有模块引用新路径。
3. 全量 `cargo fmt`, `cargo clippy`, `cargo test`；必要时补 dry-run 或仿真测试（Jupiter、Kamino、Ultra）确认落地流程正确。

## 验证与回滚策略
- 每阶段结束执行 `cargo check` + 对应 dry-run；对 Pipeline 变更补充单元测试（pipeline 组合 smoke test、decorator 顺序测试）。
- 回滚策略：保持每阶段独立分支，必要时 revert 到上一阶段；关键配置变更前记录默认值，避免影响现有 dry-run。

## 目录规划（候选树）
```
src/
  api/
  cache/
  cli/
    commands/
    engine_builder.rs
    resources.rs
    runtime/
  config/
  concurrency/
  engine/
    assembly/
      bundle.rs
      decorators/
        compute_budget.rs
        flashloan.rs
        lighthouse.rs
        jito_tip.rs
        mod.rs
    context.rs
    identity.rs
    mod.rs
    pipeline.rs
    profit.rs
    quote/
    runtime.rs
    types.rs
  instructions/            // 协议无关的指令原语层
    mod.rs
    compute_budget.rs
    flashloan/
      marginfi.rs
      mod.rs
    guards/
      lighthouse.rs
      mod.rs
    token.rs
  strategy/
    common/
    blind/
    pure_blind/
    multi_leg/
    copy/
  flashloan/
  lighthouse/
  monitoring/
  txs/
  wallet/
```
- `instructions/` 目录沉淀 ComputeBudget、Flashloan、Lighthouse 等通用指令原语；`txs/` 保持 Jupiter/Kamino 等聚合器专属的路线与账户拼装，分层后便于 pipeline 复用且避免相互污染。
- 以上结构兼顾高性能（关键路径集中在 `engine/runtime + assembly`）、可观测（monitoring 同步）、可扩展（策略/落地器/装饰器分离）。
- 参考仓库 `/home/go0p/code/rust/sol-trade-sdk` 的性能亮点（中间件链、指令缓存、零分配构建、SIMD/预取/核心亲和等），在装配与构建阶段结合使用：例如 `SmallVec<Instruction>` 缓存 compute budget、`DashMap` 缓存 ATA/PDA、对象池化 `VersionedMessage` 构造等。

## 后续工作建议
1. 先实现阶段 0–2，确保目录布局和 CLI 装配独立化，再切入 Pipeline 抽象（阶段 3）。
2. 阶段 3 完成后对闪电贷、Lighthouse、新指令扩展进行性能评估（延迟、CU 使用、指令长度）。
3. 根据 Pipeline 经验证果，决定是否将 `instructions/` 目录对外暴露为 crate（方便未来拆分成工作区成员）。

> 重构期间保持与监控/落地配置的同步，确保 hotpath 标注、metrics、配置示例与代码一致，从而实现性能优先且优雅的套利机器人架构。
