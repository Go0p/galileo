# Landing Pipeline Refactor Plan

## 背景 & 目标

Galileo 当前的落地流水线在「指令装配 → 变体复制 → 落地器发送」过程中存在强耦合：

- 指令一旦在装配阶段确定（含 tip / CU price / guard），后续所有落地器只能拿到完全相同的交易；
- 为满足 Jito 无 CU price、Staked 需要优先费等场景，只能在落地器内部“回退”或“剥离”指令，守护逻辑无法按落地器区分；
- 指令、落地配置、落地器之间的依赖线混乱，难以维护，也不利于扩展新落地器。

为解决守护阈值与落地器策略难以解耦的问题，我们计划重构落地流水线，使其具备：

1. **与落地器解耦的执行计划**：策略层只产出交易意图，不包含落地器特定细节；
2. **落地配置( Profile ) 驱动的装配器**：所有指令补全（tip、guard、CU price 等）都在落地器专属的装配环节完成；
3. **落地器只负责发送**：落地器无需再调整指令，只负责将已有的 `PreparedTransaction` 落地，保持高性能；
4. **高性能 & 扩展性**：确保现有性能不下降，同时为引入更多落地器（Temporal/Astralane 等）留下空间。

我们参考了 `sol-trade-sdk` 的三段式架构（交易参数 → 中间件链 → 并行落地），重点借鉴其「工厂 + 中间件」的组合方式，将落地流水线拆分为更细粒度的阶段，并允许针对不同落地器应用差异化的中间件链。

## 设计原则

1. **零成本抽象**：落地器的判断与转换在编译期决定，运行期仅做必要的数据搬运；
2. **管线分层**：策略→执行计划→落地装配→落地器发送，各层边界清晰；
3. **配置驱动**：更细的落地配置 `LandingProfile` 统一描述 tip / CU / guard 等策略；
4. **幂等 / 无副作用**：`ExecutionPlan` 不包含落地器副作用，`LandingAssembler` 只依据本地输入决定输出；
5. **高可测性**：每一层都可单测；守护阈值、tip 策略按落地器类型验证；
6. **可扩展**：便于新增落地器，或让现有落地器添加中间件、fallback 策略；
7. **与现有逻辑行为保持一致**：重构后 Jito、Staked、RPC 等流程的行为与当前一致（除了 bug 修复）。

## 核心概念

| 名称 | 说明 |
| ---- | ---- |
| `ExecutionPlan` | 策略层生成的纯交易意图，内含 swap 指令、ALT、元数据，不含 tip / guard / CU price。 |
| `LandingProfile` | “对某个落地器如何落地”的配置。由 lander + 全局配置/策略生成，描述 tip、guard、compute budget 等策略及中间件链。 |
| `LandingAssembler` | 根据 `ExecutionPlan + LandingProfile` 产出 `PreparedTransaction`；可针对不同落地器定制。 |
| `LanderEndpoint` | 发送交易的落地器实现（Jito / Staked / RPC…），只负责任务调度和发送。 |
| `LandingPlan` | `PreparedTransaction` 与落地器元数据的组合体，是提交给落地器的最终成果。 |

### 数据流概览

```
Strategy Engine
    └─ generate ExecutionPlan (pure swap intent, no tip/guard)
        └─ LandingPlanner
             ├─ resolve LandingProfiles (per lander)
             ├─ for each profile: LandingAssembler::build(plan, profile)
             └─ produce LandingPlan set
                 └─ TxVariantPlanner (replay bumping / dispatch layout)
                     └─ LanderStack::submit (no mutation, pure send)
```

## 详细设计

### 1. `ExecutionPlan` (策略层产物)

- 新结构 `ExecutionPlan` 替代当前 `PreparedTransaction` 的部分职责：只包含
  - 原始 swap 指令（flattened 或 bundle）
  - ALT 信息
  - Memo / 自定义标签
  - Quote 元数据（用于 tip/guard 计算，例如盈利、基础手续费预估）
  - 任何后续落地需要的上下文（base mint, amount, profit 等）
- 不包含 tip / guard / compute budget；策略模块不关心落地细节。

### 2. `LandingProfile`

对每个启用的落地器（按配置顺序）构造 Profile：

- 通用字段：
  - `lander_kind`: 枚举 (Jito, Rpc, Staked, Temporal …)
  - `cu_strategy`: Fixed / Random / DerivedFromQuote / Disabled
  - `tip_strategy`: Fixed / Range / Stream / None
  - `guard_strategy`: `Base` / `BasePlusTip` / `BasePlusPrioritizationFee` / 自定义
  - `decorators`: 指令中间件链（ComputeBudget / Tip / Guard / Flashloan / ConsoleSummary …）
  - `extra_constraints`: 如 Jito bundle 限制、Staked RPC endpoint、是否需要 skip preflight 等
  - `metrics_labels`, `logging_tags`
- Profile 由新的 `LandingProfileBuilder` 统一生成，输入是 lander 配置 + 策略上下文（是否 dry-run、是否 multi-leg）。

### 3. `LandingAssembler`

定义 trait：

```rust
trait LandingAssembler {
    fn assemble(
        &self,
        plan: &ExecutionPlan,
        profile: &LandingProfile,
    ) -> Result<PreparedTransaction, LandingError>;
}
```

实现：

- `JitoAssembler`：
  - `cu_price = 0`
  - 根据 profile 的 tip 策略生成 tip 指令（支持 stream/fixed/range）
  - 守护策略：`base_fee + tip`
  - 生成 bundle JSON / payload 需要的信息（保留 variant）
- `RpcAssembler` / `StakedAssembler`：
  - 根据 profile 采样 `cu_price`
  - tip 默认为 profile 中给定值（通常为 0）
  - 守护策略：`base_fee + prioritization_fee + tip`
- `TemporalAssembler` 等其它落地器可按需扩展

Assembler 内部可以复用现有 `DecoratorChain`，具体步骤：

1. 从 `ExecutionPlan` 构造 `InstructionBundle`；
2. 应用 Profile 的中间件链（ComputeBudgetDecorator / TipDecorator / GuardDecorator…）；
3. Flatten 成最终指令，签名交易，返回 `PreparedTransaction`。

### 4. `LandingPlanner`

新模块，负责 orchestrate：

1. 遍历启用的 lander 配置，生成 `LandingProfile`；
2. 根据 lander 类型选择对应 `LandingAssembler`（可通过工厂/枚举 match）；
3. 对每个 profile 调用 assembler 生成 `PreparedTransaction`；
4. 打包为 `LandingPlan { prepared_tx, lander_kind, metadata }`;
5. 调用现有 `TxVariantPlanner` 按 dispatch strategy 生成变体；
6. 最终交给 `LanderStack::submit`；
7. `LanderStack` 将只负责流量控制 / 多落地器提交，不再调整结构。

### 5. Lander 层调整

- `JitoLander` / `RpcLander` / `StakedLander` 不再改写指令，只发送；
- 仍然保留对网络调度、tip planning、metrics、IP 租赁等能力；
- 提交失败 / retry / deadline 等逻辑保持现状。

### 6. Tip & Guard 计算方式

- `base_fee`：全局常量（现为 5_000 lamports），仍由装配器/guard decorator 获取；
- `prioritization_fee`：由 Assembler 根据 `ExecutionPlan` 提供的 `compute_unit_limit` + profile `cu_price` 计算；
- `tip`：Assembler 根据 profile 决定；
- `guard_value`：由 `GuardStrategy` 确定组合方式；
- 依据 lander 类型自动选择策略：
  - Jito：`GuardStrategy::BasePlusTip`
  - Rpc/Staked：`GuardStrategy::BasePlusPrioritizationFee`（+ tip if any）
  - Temporal 等：可配置

### 7. Metrics & Logging

- `LandingAssembler` 在输出 `PreparedTransaction` 时记录 metrics（tip、guard、cu price），替代在 `lander::submit` 中计算；
- `LandingPlan` 包含这些指标，`LanderStack` 的日志输出直接引用；
- 监控模块读取新的结构填充事件数据，形成统一指标维度（strategy, lander_kind, cu_price, tip_source, guard_strategy…）。

### 8. 与 `sol-trade-sdk` 的对齐点

参考 `sol-trade-sdk` 的设计经验：

- **分层**：`TradeConfig → Middleware → Executor`，对应我们 `ExecutionPlan → LandingProfile → LandingAssembler`;
- **中间件机制**：利用装饰器链在组装阶段注入自定义逻辑；
- **多落地器并发**：SDK 中的 SWQOS 提供“多供应商同时发送”的能力，我们的 `LandingPlanner` + `LanderStack` 亦可支持；
- **可扩展配置**：SDK 通过枚举 + trait 对不同 DEX/落地器实现统一接口，我们也采用枚举 + profile + assembler，便于扩展；
- **性能约束**：SDK 强调 async executor + 并行提交，我们保持 `TxVariantPlanner + LanderStack` 的无锁设计，确保性能不回退。

## 实施步骤

1. **引入基础类型**  
   - `ExecutionPlan`、`LandingProfile`、`GuardStrategy`、`TipStrategy` 等；
   - Profile builder & assembler trait 雏形；
   - 将策略层产出的结构替换为 `ExecutionPlan`（保持现有字段兼容）。

2. **实现 LandingAssemblers**  
   - `JitoAssembler`：tip 处理、guard 策略、bundle 构建；  
   - `RpcAssembler`：cu price 采样、guard 计算；  
   - 复用 decorator 链逻辑（compute budget / tip / guard 等）。

3. **打通 LandingPlanner**  
   - 根据 lander 配置生成 profile；  
   - 对每个 lander 调用 assembler，生成 `LandingPlan`；  
   - 接入 `TxVariantPlanner` 及 `LanderStack`。

4. **迁移旧代码**  
   - 清理 `JitoLander` / `LanderStack` 内的剥离 logic；  
   - 移除 `PreparedTransaction` 中与 lander 相关的字段（tip plan 等）；  
   - 调整监控、日志、metrics；  
   - 更新配置加载逻辑，支持 profile 生成所需的字段。

5. **验证 & 回归**  
   - 单元测试：tip 采样、guard 计算、CU price 生成、profile builder；  
   - 集成测试：Jito 独占、Jito+Staked 混用、dry-run；  
   - 性能测试：保持现有吞吐；  
   - 文档更新：说明新的落地架构、配置方式。

6. **后续可选拓展**  
   - 支持根据 Quote 返回的 `compute_unit_price` 动态覆盖 profile；  
   - Profile 级中间件配置（例如可配置的日志 decorator）；  
   - LandingPlan 的失败 fallback（如 Jito 失败后退到 Staked）。

## 风险与对策

| 风险 | 应对措施 |
| ---- | -------- |
| 落地器组合爆炸 | profile builder 保持组合有限（仅按 lander 类型），必要时加入 priority / fallback 策略配置 |
| 性能退化 | LandingAssembler 尽量复用现有 builder，保持零拷贝；落地器只发送；上线前压测 |
| 指令兼容性 | 在 unit test 中比较新旧流水线产物（Jito / staked / rpc），确保行为一致 |
| 配置复杂化 | `LandingProfile` 结构与 `lander.yaml` 对齐，默认值合理；提供文档与示例 |
| Debug 难度 | 在 LandingPlan 和日志中输出 profile 核心参数（guard/tip/cu），方便排查 |

## 交付物

1. 新的落地流水线实现（代码 + 测试）；
2. 迁移后的配置 / 文档（说明 profile 与 lander 行为）；
3. 对应的监控指标变更说明；
4. 回归报告，覆盖守护值正确性、Jito/Staked 流程、性能数据。

---

通过上述重构，我们可以彻底摆脱“落地器临时改指令”的旧模型，实现真正的落地器解耦：策略层只关心交易意图，落地器通过 profile 和 assembler 自定义行为，高性能地完成各自的落地需求。这样不仅能解决当前守护阈值错位的问题，也为扩展更多落地器、更多策略留出了充分的空间。*** End Patch
