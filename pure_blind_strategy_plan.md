## 纯盲发策略配置拆分计划

### 背景
- 当前 `blind_strategy` 同时承载传统盲发与纯盲发配置，造成语义混乱。
- 需引入独立的 `pure_blind_strategy` 配置块，字段如用户提供的结构。
- 两套策略配置互不复用（包括 `base_mints`），并保留手工 `overrides` 能力。

### 当前进度
- ✅ `galileo.yaml` 已新增独立的 `pure_blind_strategy` 段，字段与注释对齐最新模板；原 `blind_strategy` 已剥离纯盲相关配置。
- ✅ 配置解析层更新：`GalileoConfig` 增加纯盲专用结构，落地默认值与序列化；`pure_mode`/`pure_routes` 等遗留字段全部移除。
- ✅ 策略启动逻辑拆分完毕：`run_strategy` 强制互斥两种模式；普通盲发继续走报价链路，纯盲发进入新的 `run_pure_blind_engine`。
- ✅ 纯盲落地流程目前复用现有 `PureBlindRouteBuilder`，并在运行启动时引入 `pure_blind::market_cache` 模块，按配置下载/缓存 markets.json（已支持定时刷新、程序 ID 白名单过滤）。
- ✅ 文档 (`docs/blind_strategy.md`、`closed_loop_blind_design.md`) 已更新为新配置结构；新增 `pure_blind_strategy_plan.md` 记录方案。
- ✅ `cargo fmt` / `cargo check` 通过。
- ✅ `MarketCacheHandle` 已串联到路由构建：缓存解析 `routing_group`/`token_mints` 元数据，并按程序 ID 做基础过滤。
- ✅ 自动 2-hop 路线落地：基于 `assets.base_mints` 与 `intermediates` 构建闭环候选，支持去重、lookup table 自动解析，并在监控中区分 `route_source={manual|auto}`。
- ✅ 自动 3-hop 路线落地：针对 `route_type=3hop` 组合两段中间资产，自动匹配三腿市场并生成闭环，提供 ALT 校验与失败日志。
- ✅ 监控打点覆盖纯盲关键路径：新增 `galileo_pure_blind_routes_total`、`galileo_pure_blind_route_legs`、`galileo_pure_blind_orders_total` 指标，并带路由/来源标签。
- ✅ 市场缓存阶段暂不执行二次流动性过滤，完全复用 Jupiter 已筛选结果，便于专注盲发链路。

### 待完成事项
1. **ALT 可用性与账户覆盖**
   - 对自动路线涉及的 Address Lookup Table 执行账户覆盖校验，补充缺失池的告警与降级策略。
2. **失败回退与优先级策略**
   - 对自动生成失败或池元数据拉取异常的场景，提供可配置的手工兜底优先级（manual 优先生效 / auto-only）。
   - 记录自动生成失败原因（缺少池子/ALT/流动性）并打指标，方便后续调参。
3. **监控与运行文档收尾**
   - 扩充纯盲运行指南，补充自动路由、Prometheus 指标释义及看板建议。
   - 编写离线缓存验证脚本（见新增 Task）使用说明，并在 README/Docs 补充引用。

### 里程碑
- **M1：完成路由自动生成 PoC** —— 仅支持 `route_type=2hop`，验证缓存筛选 + 闭环组装。
- **M2：扩展至多跳 & ALT 校验** —— 已覆盖 `route_type=3hop`，下一步补充 ALT 覆盖度检查。
- **M3：生产化验证** —— 跟踪自动路由的成功率、落地耗时、监控指标齐备，准备上线评审。
