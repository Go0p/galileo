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
   - 默认策略：
     - `AllAtOnce` → 单一 `TxVariant`。
     - `OneByOne` → 依据 endpoint 数量或 tip 钱包数量生成多个变体，差异体现在 tip 钱包/金额等轻量字段。
   - `PreparedTransaction` 继续携带 signer、tip 等信息，供派生变体。

3. **策略执行**
   - `AllAtOnce`：并发向所有 endpoint 推送同一交易，成本最低。
   - `OneByOne`：按 `(variant, endpoint)` 队列快速发送，期待利用限流同步延迟捕捉机会。
   - 成功即终止；失败则继续下一对，最终全部失败返回最后错误。

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
             ├── AllAtOnce → 并发同一交易
             └── OneByOne → variant × endpoint 轮询发送
```

---

## 性能与可观测性

- **性能**
  - 变体生成在 engine 层完成，避免落地器重复计算。
  - 序列化缓存并复用，减少分配和编码开销。
  - `OneByOne` 默认同步轮询（可引入有限并发），控制调度成本。

- **指标建议**
  - `lander.dispatch.attempts{strategy, variant_id, lander}`
  - `lander.dispatch.success{strategy, variant_id, lander}`
  - `lander.dispatch.tip_spent{strategy}`

---

## 测试计划

1. **单元测试**
   - `TxVariantPlanner`：验证变体数量与配置。
  - `LanderStack`：AllAtOnce、OneByOne 行为与失败回退。
   - `JitoLander`：tip 覆盖、uuid query 拼接。

2. **集成测试**
   - 搭建伪 endpoint，验证 OneByOne 在多失败场景下的回退逻辑。
   - 确认 `process_delay`、`sending_cooldown` 等节奏控制仍有效。

---

## 迭代步骤

1. 引入 `DispatchPlan/DispatchStrategy`，保持 AllAtOnce 行为。
2. 实现 `TxVariantPlanner` 与 `LanderStack::submit_plan`。
3. 改造各落地器以支持 `submit_variant`。
4. 实现 OneByOne 队列轮询逻辑以及 uuid query。
5. 增补指标、日志与对应测试。
6. 更新文档与配置示例（包含 `lander.yaml`）。

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
