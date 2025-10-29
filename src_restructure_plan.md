# Galileo 源码目录重构计划

目标：将策略相关代码统一归档，削减巨型文件，同时保持引擎/策略分层清晰，方便后续维护与扩展。

## 阶段 0：盘点与约束
- 梳理所有引用 `src/multi_leg`, `src/copy_strategy`, `src/cli/strategy.rs` 的模块。
- 记录当前 `strategy` 模块对外暴露的 trait / 类型，确保迁移后 API 兼容。
- 收集单元 / 集成测试、dry-run 脚本，规划验证手段。

## 阶段 1：目录重组
- 创建新的 `src/strategy/{blind,pure_blind,copy,multi_leg}` 目录结构。
- 将 `src/multi_leg/*` 迁入 `src/strategy/multi_leg/`，同步更新 `mod` 与 `use` 路径。
- 将 `src/copy_strategy/*` 迁入 `src/strategy/copy/`，保留旧路径的临时 re-export（避免一次性大面积修改）。
- `src/strategy/mod.rs` 补充 re-export，提供稳定入口。

## 阶段 2：拆分 CLI 逻辑
- 将 `src/cli/strategy.rs` 拆为：
  1. `cli/commands/strategy.rs`：解析 CLI 参数、调用策略运行入口。
  2. `cli/strategy/init.rs`：负责加载配置、构建 `StrategyEngine`。
  3. `strategy/{blind,pure_blind,copy}/runner.rs`：各自实现 `run_*` 接口。
- 调整 `cli` 入口，仅持有 orchestrator / builder 的装配逻辑。

## 阶段 3：策略配置解耦
- 将 blind / pure blind / copy 的配置解析与校验迁入各自模块，CLI 侧只负责传递 `AppConfig`。
- 明确各策略对 engine 依赖：将公共构建器抽到 `strategy/common`（如 `build_trade_pairs`）。
- 更新文档 `docs/` 与 `README` 中的路径示例。

## 阶段 4：长文件拆分与清理
- 针对残留巨型文件（>500 行）逐个拆分，例如：
  - `strategy/blind/runner.rs`
  - `strategy/multi_leg/runtime.rs`
- 删除阶段 1 的临时 re-export，完成路径切换。
- 全量 `cargo fmt / cargo check / cargo clippy`，dry-run 验证。

## 验证与回滚策略
- 每阶段结束执行 `cargo check` 与关键 dry-run；必要时补充单元测试覆盖迁移函数。
- 若出现生产路径风险，可通过 git revert 回滚到阶段 0 分支。

## 文档同步
- 每阶段更新对应设计文档（例：`docs/strategy_arch.md`）说明结构变化。
- 在最终 PR 写明目录调整、接口兼容性、验证结果。
