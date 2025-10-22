# 闭环盲发套利重构设计（草案）

> 目标：以“闭环路线”为核心抽象，重写盲发套利管线，使其既兼容现有两腿路线，也能扩展到任意长度的多跳路径；同时清理历史遗留代码，确保高性能、高可观测性。

---

## 1. 背景与动机

- 现有纯盲发实现基于 `buy_market` / `sell_market` 的两腿假设，无法表达 HumidiFi → Whirlpool → ZeroFi 等多跳路径。
- `route_v2` 指令的真实语义是“任意闭环”——只要资金最终回到起点槽位即可。因此我们应围绕“闭环”来设计，而非预设跳数。
- 仓库尚未上线，可放手进行破坏性重构，移除旧逻辑，避免历史包袱。

---

## 2. 设计目标

1. **闭环优先**：统一以闭环路线为基础抽象，兼容两腿、三腿乃至 N 腿。
2. **配置表达力**：支持显式声明任意腿序列；旧配置可自动升级。
3. **高性能/零成本抽象**：沿用 trait + 泛型结构，保持热路径无多余动态派发。
4. **可观测性**：新增/调整 metrics、tracing，满足守则要求。
5. **简化代码**：移除旧的两腿特化逻辑，引入更清晰的模块划分。

---

## 3. 配置与数据模型

### 3.1 配置 Schema

- 统一结构：
  ```yaml
  pure_routes:
    - name: "humid_whirlpool_zero"
      legs:
        - market: "<HumidiFi 市场>"
        - market: "<Whirlpool 市场>"
        - market: "<ZeroFi 市场>"
      # 可选：direction / program override / mid token override 等
  ```
- 校验要求：
  - 至少 2 腿。
  - 每条腿的输入 mint 必须等于上一条腿的输出 mint。
  - 最后一条腿输出 mint 必须回到第一条腿的输入 mint。

### 3.2 数据结构

- 引入 `BlindLeg`：
  ```rust
  struct BlindLeg {
      dex: BlindDex,
      market: Pubkey,
      input: AssetSlot,   // mint + token program
      output: AssetSlot,
      payload: BlindDexPayload, // 预编码/解码需要的数据
  }
  ```
- `BlindRoutePlan` 改为：
  ```rust
  struct BlindRoutePlan {
      forward: Vec<BlindLeg>,
      reverse: Vec<BlindLeg>,
      cycle_assets: Vec<AssetSlot>, // 记录每个槽位对应的资产
  }
  ```
- 反向路线通过逆序 + 方向翻转构造。

### 3.3 DEX 适配器扩展

- 在闭环模型落地过程中，同步完善 Raydium CLMM、Meteora DLMM、Orca Whirlpool 三类池子的解析与账户组装。
- 依托 yellowstone vixen 系列 parser，解码池子元数据、推导所需 PDA（tick array / bin array / oracle 等），并确定 token program、vault、vault mint 的账户顺序。
- 所有新适配器保持零成本抽象：`DexMetaProvider` 负责异步解码，`SwapAccountAssembler` 输出符合 Jupiter `RoutePlanStepV2` 规范的 `remaining_accounts`，同时回填 `BlindAsset` 的 mint / token program。

---

## 4. 路由构建流程

1. **市场元数据解析**  
   - 批量拉取 `legs` 列表的账户数据。
   - 根据 owner（Idls：`idls/jup6.json`、现有 adapter）解析出 base/quote mint、token program、DEX 专属信息。
2. **资产流校验**  
   - 初始化 `current_asset = first_leg.input`.
   - 对每条腿：
     - 判断 `current_asset` 是否匹配该腿的输入资产；若不匹配则报错。
     - 依据 DEX 特性决定 swap 方向（可自动推断，必要时读取配置或 `payload` 指示）。
     - 更新 `current_asset = 输出资产`。
   - 最终 `current_asset` 必须等于起始资产。
3. **槽位分配**  
   - 槽位 0：起始资产。
   - 对第 `i` 条腿，`input_index = slot(current_asset_in)`，`output_index = next_free_slot`；若输出回到已有槽位，则重用索引（闭环最后一腿写回槽位 0）。
   - 同时构建 `cycle_assets`，记录每个槽位绑定的资产信息，供后续 `RouteV2Accounts` 与 `AtaResolver` 使用。
4. **Payload 生成**  
   - 调用各 DEX adapter 编码 swap 数据（`HumidiFiSwap`, `WhirlpoolSwap`, `ZeroFiSwap` 等）。
   - `remaining_accounts` 顺序由 adapter 给出。

---

## 5. 引擎落地改造

### 5.1 `PureBlindRouteBuilder`

- 重写为闭环逻辑：输入配置 → 输出 `BlindRoutePlan`（包含 forward / reverse / cycle_assets）。
- 支持多腿校验、槽位分配、payload 生成。
- 加入 `cfg_attr(feature="hotpath", hotpath::measure)`，观察构建耗时。

### 5.2 `Engine::build_route_plan`

- 移除 base/quote 恒等假设，改用 `BlindLeg` 提供的 `input` / `output`。
- 根据 `cycle_assets` 调用 `AtaResolver::get`，确保预先推导所有中间资产的 ATA。
- 生成 `RoutePlanStepV2` 和 `remaining_accounts`。
- 更新 `resolve_step_input/output` → `resolve_leg_input/output`，支持任意资产。

### 5.3 其他调整

- `BlindSwapDirection` 可扩展为 `LegFlow { input: AssetRole, output: AssetRole }`，或保留枚举但与资产槽位绑定。
- 清理所有已弃用的两腿特化分支。

---

## 6. 监控与性能

1. **指标**  
   - `blind.route_legs_total{route, legs}`：记录各路线腿数。
   - `blind.tx_prepared_total{route, direction}`：原有指标可复用，但增加 route 标签。
   - `blind.slot_mapping_total{slot}`：用于调试槽位分配异常。
2. **日志**  
   - 在构建与执行阶段打出 `route`, `legs`, `slot_plan`, `assets` 等字段。
3. **Hotpath**  
   - 对构建、落地关键函数添加 `hotpath::measure` 或 `hotpath::measure_block!`。

---

## 7. 测试计划

1. **单元测试**
   - 配置解析：两腿/三腿/四腿闭环成功，非法配置（断腿、不闭环、重复资产）报错。
   - 槽位分配：断言 `input_index`/`output_index` 序列与预期一致。
   - Payload 编码：针对 HumidiFi / Whirlpool / ZeroFi / SolFiV2 / TesseraV / ObricV2 生成样例。
2. **集成测试**
   - 构造伪 DEX adapter → 校验 `RouteV2Instruction` 与 `remaining_accounts`。
   - 闭环多跳 + flashloan 组合的 end-to-end dry-run。
3. **回归测试**
   - 现有两腿路线应输出与旧版本一致的 `RoutePlanStepV2`（除文档要求的清理外）。

---

## 8. 迁移策略

1. **配置迁移脚本**（可选）：把旧 `buy/sell` 转换成新 `legs` 格式，输出 diff 供用户确认。
2. **文档更新**：同步 `docs/blind_strategy.md`、`docs/strategy_arch.md`、`galileo.yaml` 示例。
3. **启用步骤**：
   - 切至新分支 → 实施代码重构。
   - 跑 `cargo fmt`、`cargo test`, 特别是 `pure_blind` 相关测试。
   - Dry-run 验证闭环路线。

---

## 9. 风险与缓解

| 风险 | 说明 | 缓解措施 |
| --- | --- | --- |
| 槽位分配错误 | `input_index` / `output_index` 出错将导致交易失败 | 构建阶段加入断言 & 单测覆盖；日志打印槽位映射 |
| DEX payload 不兼容 | 多腿路线需要更多协议组合 | 使用 IDL + adapter 统一编码；必要时引入模拟交易验证 |
| 性能开销增加 | 多腿构建需要更多 RPC 解码、账户组装 | 复用现有 adapter 缓存（单 tick 内）、加 `hotpath` 分析并优化 |
| Flashloan 顺序影响 | 闭环路径可能对 flashloan 依赖不同 | 在 `flashloan` 管线里抽象成对 `route_plan` 的包装，必要时扩展配置 |

---

## 10. 下一步

1. 落地配置解析与 `PureBlindRouteBuilder` 重写。
2. 重构 `Engine::build_route_plan`，接入新 `BlindLeg` 抽象。
3. 更新监控与测试，完成回归验证。
4. 整理文档、配套示例，准备破坏性合并。

---

> 维护者：TODO（合入时填写）  
> 初稿：2025-XX-XX
