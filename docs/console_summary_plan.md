# 控制台机会摘要面板设计方案

> 目标：在策略运行过程中，实时/准实时地在控制台底部展示每波 `trade size` 批次的关键统计信息，类似“悬浮状态栏”，以便快速了解整体表现。

---

## 1. 需求拆解

### 1.1 展示内容
- **机会概览**：按 base mint 汇总的机会数量、平均利润。
- **Quote 延迟**：forward / reverse 平均延迟（ms）。
- **Quote 成功率**：成功的 quote 组（正反双向都成功）数量 / 总 quote 组数。
- **落地结果**：成功发送的交易 / 尝试发送的交易。
- 文案示例：  
  ```
  机会数: 3/WSOL,5/USDC | 平均延迟: 100ms/123ms | 平均利润: 12345/WSOL,1234/USDC | Quote组: 8/10 | 成功发送数: 3/8
  ```

### 1.2 展示时机
- 以 **单次 `StrategyEvent::Tick` 调度的完整批次** 为单位：当 `run_quote_batches` 完成，且所有 quote 与落地流程在当前批次内走完后，输出一行 summary。
- 对于 legacy/multi-leg 模式，沿用相同策略：批次结束后汇总输出。

### 1.3 性能约束
- 统计逻辑必须在内存中完成，避免额外 IO/锁。
- 日志输出频率受批次触发频率约束，可考虑多次 tick 间隔较短时进行节流。
- 不影响 quote 的并发与调度逻辑，只在汇总阶段读取已存在的数据结构。

### 1.4 控制台呈现
- 默认使用 `tracing::info!` 打印单行 summary。
- “悬浮 / 悬停”效果可通过以下手段之一：
  1. 借助 `indicatif::MultiProgress` 或类似库，在日志输出前设置一个独立的 progress bar 持续刷新最新 summary。
  2. 自行实现简单的行刷新（`\r` / ANSI 序列）在最后一行覆盖更新。
- 需要可配置开关，例如 `console.summary.enable`，默认为关闭，避免影响无人值守运行时的日志格式。

---

## 2. 数据采集方案

### 2.1 统计数据结构
新增 `BatchStats`（拟放置在 `src/engine/runtime/strategy/quote.rs`）：
```rust
struct BatchStats {
    total_groups: u64,
    successful_groups: u64,
    forward_latency_total: Duration,
    forward_latency_count: u64,
    reverse_latency_total: Duration,
    reverse_latency_count: u64,
    opportunities: HashMap<String, OpportunityStats>,
    executed_trades: u64,
    attempted_trades: u64,
}

struct OpportunityStats {
    total_profit: i128,
    count: u64,
}
```
- `total_groups`：本批次 quote 组数量，即 `QuoteBatchPlan` 数量。
- `successful_groups`：`QuoteDispatchOutcome` 中拿到 forward+reverse quote 的组数。
- `forward_latency_*` / `reverse_latency_*`：只统计成功 quote 的延迟。
- `opportunities`：key 为 base mint（字符串），value 包含总利润和机会数，便于输出平均值。
- `executed_trades`：落地成功的交易数。
- `attempted_trades`：触发落地（机会）尝试数。

### 2.2 采集入口
- 在 `QuoteDispatcher::dispatch` 结果循环处理时，调用 `stats.record_quote_outcome(...)`。
- 在 `process_quote_outcome` 成功获取 `SwapOpportunity` 后，调用 `stats.record_opportunity(...)`；在 `execute_plan` 返回 `Ok` 时，调用 `stats.record_execution(true/false)`。
- legacy 路径（multi-leg）同样复用 `BatchStats`。

### 2.3 清理与复用
- 每次 `run_quote_batches` 调用新建 `BatchStats` 实例，函数结束后输出 summary 并丢弃。
- 未来若需要持续滚动统计，可在 `StrategyEngine` 内部维护一个滑动窗口或全局累计器。

---

## 3. 控制台展示实现

### 3.1 输出格式化
- 构建 `String`（或 `fmt::Display` 实现）拼装：
  - `机会数`: 遍历 `opportunities`，格式 `count/base_mint`，使用逗号分隔。
  - `平均延迟`: `forward_avg` / `reverse_avg`。无数据则以 `-` 占位。
  - `平均利润`: `total_profit / count` 按 base mint 输出。
  - `Quote组`: `successful_groups/total_groups`。
  - `成功发送数`: `executed_trades/attempted_trades`。

### 3.2 控制台刷新策略
**方案 A：简单日志**  
  - 使用 `info!(target = "engine::summary", summary = %line)`，让日志系统自然输出。实现快速、稳定，但无法保持“悬浮”。

**方案 B：刷新最后一行**  
  - 使用 `print!("\r{}", line);` 并在末尾添加空格清空上次输出；每次 summary 更新时覆盖同一行。
  - 需在程序退出、收尾时打印换行 `println!()`，保持控制台状态正常。
  - 注意当其他日志输出打断时，该行会被冲掉（可接受）。

**方案 C：集成 `indicatif`**  
  - 引入第三方库，在 CLI 模式下创建一个 `MultiProgress` 实例，把 summary 作为常驻 `ProgressBar`，其他日志通过 `tracing` 输出。
  - 需评估依赖引入和与现有日志系统的兼容性。

**推荐**：初期先实现方案 A（单行日志），结合配置开关。如果确认对控制台刷新有强需求，可迭代至方案 B/C。

### 3.3 配置开关
- 在 `config/types.rs` 新增：
  ```rust
  #[derive(Debug, Clone, Deserialize, Default)]
  pub struct ConsoleSummaryConfig {
      #[serde(default)]
      pub enable: bool,
  }
  ```
- `AppConfig.galileo.engine.console_summary.enable` 控制是否启用。
- `StrategyEngine::new` 根据配置决定是否创建 `BatchStats` 并输出日志。

---

## 4. 代码改动计划

1. **配置层**
   - `src/config/types.rs` 添加 `ConsoleSummaryConfig`，在 `EngineConfig` 中引用。
   - 示例配置（`galileo.yaml`）增加注释，默认 `enable: false`。

2. **统计结构**
   - 新建 `BatchStats`（建议放在 `src/engine/runtime/strategy/quote.rs` 内部或独立模块）。
   - 提供 `record_quote_outcome`、`record_opportunity`、`record_execution`、`summary_line()` 等方法。

3. **策略引擎集成**
   - `run_quote_batches` 创建 `BatchStats`，传递到 `process_quote_outcome` 等函数。
   - 调用 `execute_plan` 后根据结果更新统计。
   - 批次结束时，如果 `console_summary.enable` 为真，打印 summary 行。

4. **控制台输出**
   - 首版采用方案 A（单行日志）。实现简单，后续如需悬浮效果再扩展。
   - 如果选择方案 B/C，需要抽象 `SummaryRenderer` trait，默认实现为简单日志，未来可替换。

5. **文档更新**
   - 在 `docs/blind_strategy.md` / `docs/strategy_arch.md` 添加 “控制台摘要” 说明和配置字段。

6. **测试**
   - `cargo fmt` / `cargo check`。
   - 可添加一个单元测试，验证 `BatchStats::summary_line()` 格式化输出。

---

## 5. 风险与注意事项
- **日志抢占**：在高并发下，其他模块可能频繁输出日志，summary 行容易被冲掉；需要在文档中说明这一点。
- **统计精度**：延迟统计只包含成功 quote 的组；失败的组需要明确定义（当前可忽略或记录单独的计数）。
- **配置兼容**：默认关闭，不影响现有部署；新增字段需保证旧配置能正常反序列化。
- **性能**：统计开销主要是简单累加，对整体性能影响可以忽略。

---

## 6. 未来扩展
- 支持基于 `per_base_mint` 的单独统计面板。
- 将 summary 数据同步到 Prometheus（例如 `galileo_summary_last_opportunities_total`），供仪表盘使用。
- 提供 HTTP/WS 接口实时查询 summary。

