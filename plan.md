# 项目推进计划（galileo） – 2025-10-12

## 当前进展摘要
- ✅ **配置体系精简完成**  
  仅保留 `galileo.yaml` 与 `jupiter.toml` 中实际存在的字段；移除历史遗留开关（`bot.identity`、`jupiter.tokens`、`disable_update` 等）。二进制启动参数通过 `LaunchOverrides` 动态生成。
- ✅ **Jupiter 管理流程完善**  
  启动前会校验版本文件并跳过重复下载，写入 `.jupiter-version` 与版本化副本；`galileo jupiter start` 现以前台模式运行，日志可实时查看，`Ctrl+C` 触发优雅停止。
- ✅ **策略引擎重构**  
  身份信息由环境变量与 `global.wallet` 提供，去掉强制的配置段；保留套利循环框架，可接入后续交易执行逻辑。
- ✅ **日志与 CLI 行为对齐**  
  支持 `JUPITER_URL`/`JUPITER_URL` 环境变量覆盖 base URL；无效的 CLI flag 已移除。
- ✅ **Quote 接口联调通过**  
  手动执行 `curl` 至 `http://172.22.166.244:18080/quote` 返回预期 JSON，`restrictIntermediateTokens=true` 等全局配置已生效。
- ✅ **Rust Demo 完整示例**  
  `docs/demo.md` 提供 Rust 版 quote/swap/bundle 样例，对齐 blind_strategy / back_run_strategy 配置。
- ✅ **策略执行链路初版**  
  `ArbitrageEngine` 接入交易构建与 Jito Bundle 发送，现阶段以 blind_strategy 基础配置（DEX 白名单、优先费、落地通道）为主。
- ⚙️ `cargo check` 通过；`cargo run -- jupiter start` 会在前台输出 Jupiter 日志并复用本地二进制。

## 未决问题 / 约束
1. **策略身份环境变量**：运行套利前需要设置 `GALILEO_USER_PUBKEY`（可选：`GALILEO_FEE_ACCOUNT` 等）；缺失时引擎会报错。
2. **生产级套利引擎**：尚需补齐行情采集、实时调度、执行落地、风控等核心模块。
3. **高性能与可观测性**：需按 README 中的并发/监控方案实现指标、限流、压测工具（报价场景以低延迟为主，暂不引入 quote 缓存）。
4. **运维流程**：缺少部署/灰度/告警脚本与文档。
5. **今日进度**：完成 Quote 接口远端验证与 Rust Demo 编写，即将进入生产套利阶段。

## 下一阶段计划

### 阶段 A（短期）— 完成基础运行闭环
1. **运行配置验收**  
   - [x] 在预期环境执行 `cargo run -- jupiter start`，验证 `.jupiter-version` 与版本化副本写入（用户实测通过）。  
   - [x] 通过 `cargo run -- jupiter status` 和 `ps` 双重确认后台进程存在。  
   - [x] 根据需要将 `global.logging.json` 设为 `false` 以启用文本日志。  
2. **策略最小自检**  
   - [x] 准备必需环境变量（`GALILEO_USER_PUBKEY` 等）。  
   - [x] 运行 `cargo run -- strategy` 检查初始化与身份解析是否顺利。  
   - [x] 确认 Jupiter API base URL 正常解析（本地或远端）。
3. **文档对齐**  
   - [x] 将 `docs/demo.md` 更新为涵盖 blind_strategy / back_run_strategy 调用流程要点。  
   - [x] 补充 `/quote` 与 `/swap-instructions` 请求参数与 `request_params` 段配置之间的映射说明。

### 阶段 B（中期）— 生产级套利引擎
1. **架构定稿与设计输出**  
   - [x] 结合 README 高性能方案拆分模块（行情采集、报价调度、执行落地、风控监控）。  
   - [ ] 明确策略 trait/泛型接口与缓存职责，形成 `docs/strategy_arch.md` 草案。
2. **行情采集与状态服务**  
   - [ ] 接入 Yellowstone gRPC / RPC，维护最新 slot / blockhash / orderbook 等热数据（quote 请求直接走实时通道）。  
   - [ ] 为落地与风控提供只读状态服务，并补充一致性 / 延迟基准测试脚本。
3. **套利调度执行链路**  
   - [ ] 搭建异步通道 + Rayon 组合的多策略报价流水线（blind/back_run）。  
   - [x] 抽象 bundle 发送器，支持 Jito / Staked 等落地通道，并实现优先费/小费策略（当前实现 Jito，预留多通道扩展）。
4. **风险控制与资金管理**  
   - [ ] 实现滑点、持仓阈值、冷却时间等风控模块。  
   - [ ] 增加失败回退、重试与黑名单逻辑。

### 阶段 C（长期）— 可观测性与性能运营
1. **监控与日志**  
   - [ ] 扩充 metrics（quote/swap latency、bundle 命中率、缓存命中率）。  
   - [ ] 统一日志字段，评估接入 Prometheus / OpenTelemetry。
2. **压测与调优**  
   - [ ] 构建压测场景（模拟高并发报价与 bundle 提交），评估 CPU/内存/延迟。  
   - [ ] 按压测结果优化内存布局、锁粒度与异步调度策略。

### 阶段 D（运维）— 部署与告警
1. **部署流程**  
   - [ ] 编写 systemd/docker 部署脚本与滚动升级策略。  
   - [ ] 整合配置下发、版本管理与回滚流程。
2. **告警与 SLO**  
   - [ ] 定义核心告警项（进程心跳、API 错误率、套利利润波动等）。  
   - [ ] 建立日常巡检与故障演练清单。

## 里程碑
- **M1**：Jupiter 二进制管理 + 基本策略循环稳定运行（当前大部分内容已完成，待环境验证 ✅）。
- **M2**：生产套利引擎上线（阶段 B 完成）。
- **M3**：可观测性完善 + 运维体系落地（阶段 C、D 完成）。

## 后续协作建议
- 若需新增配置字段，优先更新 `galileo.yaml` / `jupiter.toml` 模板并同步 `types.rs`；保持“配置先行，代码跟随”的约定。
- 策略模块依赖的环境变量请在 README 或单独文档中列出，方便部署人员配置。
- 关键变更（性能、监控、策略执行）建议先方案评审，再进入开发阶段。
- 运维与性能相关的新增文档建议归档于 `docs/` 子目录，便于后续查阅。
