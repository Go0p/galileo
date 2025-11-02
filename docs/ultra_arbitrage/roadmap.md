# Ultra 套利实施路线图

| 阶段 | 目标 | 关键任务 | 输出物 |
| --- | --- | --- | --- |
| Phase 0 | 奠定基础 | ✅ Ultra API 客户端（/order,/execute）；配置新增 `leg` 字段 | `src/api/ultra/*`, `galileo.yaml` |
| Phase 1 | multi_leg 基础 | ✅ 定义 `LegSide/QuoteIntent/LegPlan`；提供交易解码 & 指令清理工具；创建模块骨架；引入 `engine.backend=none` 模式 | `src/engine/multi_leg/{types,leg,transaction}`, `src/config/types.rs` |
| Phase 2 | Provider 接入 | ✅ DFlow / Ultra / Titan provider；完成 Titan 报价抽象与 Ultra 交易解包 | `src/engine/multi_leg/providers` |
| Phase 3 | 组合 orchestrator | - ✅ 初版 orchestrator：支持腿注册、配对枚举、统一 quote→plan 调用<br>- ✅ `MultiLegRuntime` + ALT 缓存批量填充<br>- ✅ 自动化配对 & 收益排序（基于 `LegPlan.quote`），并对 Titan 推流加并发/防抖保护<br>- ◻ 与 `TransactionBuilder` / Jito bundle 对接 | `src/engine/multi_leg/{orchestrator,runtime,alt_cache}` |
| Phase 4 | 观测与容错 | - 新增多腿指标（Quote/Plan/Bundle）<br>- tracing 贯穿腿 ID → bundle ID<br>- 落盘/回放工具便于调试 | `monitoring/events`, 文档更新 |
| Phase 5 | 上线准备 | - Dry-run & replay 测试<br>- 性能基准（延迟、成功率）<br>- 仪表盘 & 告警同步 | PR 汇总，仪表盘链接 |

## 近期迭代顺序
1. **Engine 对接**：在 `engine.backend = none` 路径实例化 `MultiLegRuntime`，串联 `TransactionBuilder` / Jito bundle，并复用现有收益阈值与监控。  
2. **收益/风控融合**：将 `PairPlanBatchResult` 接入既有 `ProfitEvaluator` / 风控开关，补齐失败重试策略与最小收益过滤。  
3. **观测与测试**：补充多腿 metrics / tracing、Titan 推流专用观测，以及 ALT 缓存 & runtime 的单元/集成测试。  

## 里程碑验收标准
- **M1**：给定固定代币对，能稳定完成一次买卖腿并在本地模拟通过。  
- **M2**：Jito bundle 在测试环境成功落地，失败路径可追踪。  
- **M3**：Prometheus 指标齐备，告警策略评审通过。  
- **M4**：Dry-run 连续运行 >=48 小时且无未解决故障，准备上线。  
