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
- ⚙️ `cargo check` 通过；`cargo run -- jupiter start` 会在前台输出 Jupiter 日志并复用本地二进制。

## 未决问题 / 约束
1. **策略身份环境变量**：运行套利前需要设置 `GALILEO_USER_PUBKEY`（可选：`GALILEO_FEE_ACCOUNT` 等）；缺失时引擎会报错。
2. **高性能与可观测性**：尚未覆盖缓存、压测指标、监控埋点等增强项。
3. **运维流程**：缺少部署/灰度/告警脚本与文档。
4. **今日进度**：完成配置精简与启动流程调试，暂定今日收工，下一次会话继续推进阶段 A 的环境验证。

## 下一阶段计划

### 阶段 A（短期）— 完成基础运行闭环
1. **运行配置验收**  
   - [ ] 在预期环境执行 `cargo run -- jupiter start`，验证 `.jupiter-version` 与版本化副本写入。  
   - [ ] 通过 `cargo run -- jupiter status` 和 `ps` 双重确认后台进程存在。  
   - [ ] 根据需要将 `global.logging.json` 设为 `false` 以启用文本日志。  
2. **策略最小自检**  
   - [ ] 准备必需环境变量（`GALILEO_USER_PUBKEY` 等）  
   - [ ] 运行 `cargo run -- strategy` 检查初始化与身份解析是否顺利。  
   - [ ] 确认 Jupiter API base URL 正常解析（本地或远端）。

### 阶段 B（中期）— 高性能与可观测性
1. **HTTP/Quote 性能优化**  
   - [ ] 评估 `JupiterApiClient` 连接池、超时和重试策略。  
   - [ ] 考虑缓存热门 quote 结果，减少重复 API 调用。
2. **监控与日志**  
   - [ ] 扩充 `metrics` 模块，记录 quote / swap latency、下载耗时等。  
   - [ ] 建立统一的日志字段规范，规划是否接入 Prometheus / OpenTelemetry。

### 阶段 C（长期）— 策略执行与运维
1. **套利执行闭环**  
   - [ ] 接入实时机会过滤、落地交易（Jito 或自建 relayer）。  
   - [ ] 补充滑点、资金阈值、失败回退等风控模块。
2. **部署与告警**  
   - [ ] 编写 systemd/docker 部署脚本与巡检脚本。  
   - [ ] 定义 SLO / 告警策略（进程存活、API 失败率等）。

## 里程碑
- **M1**：Jupiter 二进制管理 + 基本策略循环稳定运行（当前大部分内容已完成，待环境验证 ✅）。
- **M2**：性能指标达标并具备监控体系（阶段 B 完成）。
- **M3**：套利执行闭环 + 运维体系落地（阶段 C 完成）。

## 后续协作建议
- 若需新增配置字段，优先更新 `galileo.yaml` / `jupiter.toml` 模板并同步 `types.rs`；保持“配置先行，代码跟随”的约定。
- 策略模块依赖的环境变量请在 README 或单独文档中列出，方便部署人员配置。
- 关键变更（性能、监控、策略执行）建议先方案评审，再进入开发阶段。
- 若暂时不继续开发，可按当前状态保存；下次会话从阶段 A 的验证任务开始。
