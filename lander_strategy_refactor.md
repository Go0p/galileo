# Lander 发送策略重构设计

> 目标：在不考虑历史兼容的前提下，重构落地层，支持多种发送策略并清理老旧逻辑。强调高性能、优雅架构与易扩展性。

---

## 核心变化概览

1. **统一落地入口**
   - 引入 `DispatchPlan`，封装发送策略 (`DispatchStrategy`) 和交易变体 (`TxVariant`)。
   - 策略枚举：
     ```rust
     pub enum DispatchStrategy {
         AllAtOnce,
         OneByOne,
     }
     ```
   - `LanderStack::submit_plan(plan, deadline, strategy_name)` 成为唯一入口，替换旧 `submit(prepared, …)`。

2. **交易变体生成（TxVariantPlanner）**
   - 在 `engine` 层完成，确保落地器只负责发送。  
   - 生成原则：
     - `AllAtOnce` → 为每个 lander 实例只衍生一份带签名的 `TxVariant`，供该实例下的全部 endpoint 共用；Jito 如需追加 tip 会在这份变体基础上局部加工，但不会重新签名。
     - `OneByOne` → 针对「lander × endpoint」数量派生多份互相独立的 `TxVariant`，每份都拥有独立签名，以便针对不同 endpoint 调整 tip、优先费等参数并并发投递。
   - `PreparedTransaction` 继续携带 signer、tip 等信息，为变体派生提供上下文。

3. **策略执行（核心语义）**
   - `AllAtOnce`
     - 对同一个 lander，仅签名并构造**一笔交易**；若该 lander 需额外加工（例如 Jito 加 tip、Staked 直接使用原交易），会在生成后再做轻量转换，但签名保持唯一。
     - 这一笔交易被同时广播到该 lander 管理的全部 endpoint，实现“同一签名、并发发送、先到先得”，最大化 gas 共享效率。
     - 多个 lander 并行工作，各自广播自己的那一笔交易版本。
   - `OneByOne`
     - `TxVariantPlanner` 为同一个 lander 的每个 endpoint 生成**独立签名/独立交易**（N 个 endpoint 就有 N 笔 tx），可以在 tip、优先费、lookup-table 等细节上做差异化探索。
     - 这些交易在同一时刻并发发送到各自的 endpoint，利用“多路不同参数”提高命中/收益率，即便单点失败也不会影响其它 endpoint 的尝试。
     - 同样遵循“任一交易落地成功即整体成功”，若全部失败也会串联错误向上抛出。

4. **Jito 落地器重构**
   - 接收 `TxVariant`，根据变体参数调整 tip 钱包/金额，构造 bundle。
   - UUID 池逻辑沿用，并在 OneByOne 时为每次请求附带 `uuid` query 参数。

5. **配置简化**
   ```yaml
   lander:
     sending_strategy: "AllAtOnce"  # 或 "OneByOne"
   ```
   - 其它节奏控制（`process_delay`、`sending_cooldown`）继续在策略层使用。
   - 不再引入额外复杂配置保持策略轻量。

6. **保留并梳理 tip 配置**
   - 现有 `LanderJitoConfig` 中的固定/区间/floor tip 配置继续沿用；Planner 负责读取并生成基准交易。
   - 交易变体可以在 _已有配置基础上_ 调整 tip 或其他维度（如 CU limit、指令顺序），而非硬编码随机值。
   - Jito 落地器由 Planner 明确告知是否覆盖 tip，落地器自身仅负责执行，不再分散 tip 逻辑。

---

## 模块结构调整

```text
engine/
  ├── builder.rs            (PreparedTransaction)
  ├── planner.rs            (TxVariantPlanner, DispatchPlan)      ← 新增
  └── mod.rs                (StrategyEngine 调用 submit_plan)

lander/
  ├── stack.rs              (LanderStack::submit_plan)
  ├── jito.rs               (JitoLander::submit_variant)
  ├── rpc.rs / staked.rs    (与 TxVariant 兼容)
  └── mod.rs                (暴露 Dispatch 使用的类型)
```

---

## 时序流程

```
套利机会 → PreparedTransaction
             ↓
         TxVariantPlanner
             ↓
  DispatchPlan { strategy, variants }
             ↓
  LanderStack::submit_plan
             ├── AllAtOnce → 每个 lander 拿到单一签名的交易版本，广播至其全部 endpoint
             └── OneByOne → 为每个 endpoint 准备独立签名的交易版本，并发发送
```

---

## 性能与可观测性

- **性能**
  - 变体生成在 engine 层完成，避免落地器重复计算。
  - 序列化缓存并复用，减少分配和编码开销。
  - `AllAtOnce` 广播路径只复制必要的几份交易，`OneByOne` 则按 endpoint 预先烘焙多份变体，通过并发发送提升首包命中率，同时保留失败后的精细化重试空间。

- **指标建议**
  - `lander.dispatch.attempts{strategy, variant_id, lander}`
  - `lander.dispatch.success{strategy, variant_id, lander}`
  - `lander.dispatch.tip_spent{strategy}`

---

## 测试计划

1. **单元测试**
   - `TxVariantPlanner`：验证在 AllAtOnce/OneByOne 下生成的变体数量、参数差异和 mint 绑定关系。
   - `LanderStack`：确认 AllAtOnce 会对每个 lander 启动一次并发广播，而 OneByOne 会针对 endpoint 派发专属变体并在失败后继续投递。
   - `JitoLander`：tip 覆盖、uuid query 拼接，以及在不同策略下的 bundle 组装差异。

2. **集成测试**
   - 搭建伪 endpoint，验证 AllAtOnce 同时命中多个 endpoint 的行为，以及 OneByOne 在多失败场景下的回退逻辑。
   - 确认 `process_delay`、`sending_cooldown` 等节奏控制仍有效。

---

## 迭代步骤

1. 引入 `DispatchPlan/DispatchStrategy`，初版即对 lander/endpoint 做并发发送。
2. 实现 `TxVariantPlanner`：支持按 lander/endpoint 生成专属变体，并在 AllAtOnce/OneByOne 下使用不同粒度的复制策略。
3. 改造 `LanderStack::submit_plan`，AllAtOnce 并发广播变体、OneByOne 并发下发 endpoint 专属交易，同时统一成功/失败回调。
4. 更新 Jito/Staked 等落地器实现：
   - Jito 在 AllAtOnce 模式下只构造一份含 tip 的 bundle，并并发灌入全部 endpoint。
   - 在 OneByOne 模式下，根据 Planner 提供的多份 bundle 逐一并发交付（tip、优先费等参数可因变体而异）。
5. 扩展指标/日志覆盖，验证不同 endpoint 下的落地情况。
6. 补充配置示例与运行手册（含 `lander.yaml`）以明确新语义。

---

## 清理列表

- `LanderJitoConfig.tip_strategies` 等旧版配置字段与解析逻辑。
- `LanderStack::submit` 旧接口及调用链。
- Jito 落地器中过时的 tip 处理占位代码、重复序列化路径。
- 分散在 engine/lander 中的 tip 选择逻辑，统一迁移至 `TxVariantPlanner`。

---

通过上述重构：

- Engine 专注“生成什么交易”；Lander 专注“如何发送”。
- 同一配置下，默认行为等价 `AllAtOnce`；开启 `OneByOne` 时成本与成功率可控。
- 移除历史包袱，确保热路径更简洁、高效，便于后续扩展（如动态策略、端点优先级等）。
