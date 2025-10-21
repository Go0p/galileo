# Galileo Dex 模块重构路线图

> 目标：打造高性能、易维护的盲发套利核心，解耦各 DEX 适配层，实现统一的 Jupiter 指令构建流程。

## 阶段一（当前）

- [x] **Trait 框架落地**
  - 定义 `DexMetaProvider`：负责市场配置解析、swap_id/动态参数生成，具备单 tick 范围内的零拷贝缓存能力。
  - 定义 `SwapAccountAssembler`：根据用户身份拼装 DEX `remaining_accounts`，并提供 metrics hook。
- [x] **Adapter 注入**
  - 为 HumidiFi / SolFiV2 / TesseraV 实现 Adapter，并在纯盲策略中统一通过适配器解析市场。
  - 为策略引擎增加 `hotpath::measure` 埋点，记录元数据解析与账户生成耗时。

## 阶段二

- [ ] **Jupiter 子系统重构**
  - 拆分 `txs/jupiter`：`swaps`（payload 编码）、`accounts`（标准账户生成）、`route_v2`（数据布局）。
  - 新增 `SwapInstructionBuilder`，统一拼装 `Instruction` 与监控附件。
  - 对接 `DexMetaProvider` 输出，确保 `route_plan` 与 `remaining_accounts` 均由新 builder 管理。
- [ ] **策略/引擎适配**
  - `PureBlindRouteBuilder` 与 `StrategyEngine` 仅依赖 trait/registry，不再引用具体 DEX 模块。
  - 清理遗留的 payload 结构体，统一通过 Jupiter builder 生成 `EncodedSwap`。

## 阶段三

- [ ] **观测与性能**
  - 引入 `monitoring::dex` 模块，打点 `dex_swap_accounts_built_total`、`dex_meta_latency_ms` 等指标。
  - 补充 `docs/architecture/blind_dex.md`，描述分层、扩展流程、性能调优手册。
  - 视需要添加 `criterion` 基准，对比重构前后延迟。

> 本路线图会随实现进展及时更新；完成各阶段后会同步新增回归测试与文档说明。
