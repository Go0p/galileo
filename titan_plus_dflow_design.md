# Titan + DFlow 组合套利设计草案

> 目标：在 Titan WS 订阅的一条腿基础上，引入 DFlow HTTP 报价 / 指令生成作为另一条腿，实时拼装闭环交易，实现稳定、低延迟的双腿套利。

## 1. 架构概览

```
┌─────────────────┐      TitanQuoteUpdate      ┌──────────────────┐
│ TitanWsClient   │  ───────────────────────▶  │ TitanLegHandler  │
│  (WS 单流)       │                            │  (买腿估值)        │
└─────────────────┘                            └──────────────────┘
                                                         │
                                                         │ DFlowQuoteRequest
                                                         ▼
                                               ┌──────────────────┐
                                               │ DflowApiClient   │
                                               │  (卖腿报价/指令)    │
                                               └──────────────────┘
                                                         │
                                                         │ 完整报价响应 + Titan 数据
                                                         ▼
                                               ┌──────────────────┐
                                               │ ArbitragePlanner │
                                               │  windfall / risk │
                                               └──────────────────┘
                                                         │
                                                         │ SwapInstructions + TitanRoute
                                                         ▼
                                               ┌──────────────────┐
                                               │ TransactionBuilder│
                                               │  拼装/提交         │
                                               └──────────────────┘
```

关键点：

- **TitanWsClient**：只订阅固定 `input → output` 的单条流。推送频率由 `interval_ms` 控制，需要保证在 Titan 推送间隔内我们能完成 DFlow 报价和后续落地。
- **TitanLegHandler**：解析 Titan 的 `SwapQuotes`，根据 `providers`、`reverse_slippage_bps` 过滤、排序，得出买腿实际可用的 `{in_amount, out_amount, provider}`。
- **DflowApiClient**：现已支持 `api_quote_base` / `api_swap_base` 两个端点。卖腿报价需兼容 Titan 推送的数额，保证闭环一致。
- **ArbitragePlanner**：把 Titan leg 的 out 作为 DFlow leg 的 in，扣除 gas/Jito 等成本后计算净收益。若满足阈值 → 请求 `/swap-instructions`，进入交易构建。

## 2. 配置 & Backend

- `galileo.engine.backend` 新增枚举值 `titan_dflow`。
- `galileo.engine.titan` 新增结构（已在 `TitanEngineConfig` 中定义）：
  - `enable`: 是否启用；`ws_url`、`jwt`、`default_pubkey` 必须设定。
  - `providers`: Titan 允许的报价来源列表（默认为空=全部）。
  - `reverse_slippage_bps`: Titan 反向腿使用的滑点。
  - `interval_ms` / `num_quotes`: 控制 Titan 服务端推送节奏。
- `galileo.engine.dflow` 已支持 `api_quote_base` / `api_swap_base` 两个端点，适配 `/quote` 与 `/swap-instructions` 分别走 Proxy 与主域。
- `galileo.engine.dflow.max_consecutive_failures` 控制 DFlow 报价/指令阶段的连续失败容忍度，达到上限后机器人退出；设为 `0` 表示无限容忍。
- `galileo.engine.dflow.wait_on_429_ms` 指定命中 429 限流后等待的毫秒数，避免继续冲击 DFlow 限流。

示例（根目录 `galileo.yaml`）：

```yaml
galileo:
  engine:
    backend: "titan_dflow"
    titan:
      enable: true
      ws_url: "wss://api.titan.exchange/api/v1/ws"
      default_pubkey: "Titan11111111111111111111111111111111111111"
      jwt: "<JWT>"
      providers:
        - "Titan"
        # - "Metis"
      reverse_slippage_bps: 0
      interval_ms: 800
      num_quotes: 2
    dflow:
      enable: true
      api_proxy: "http://192.168.124.4:9999"
      api_quote_base: "https://aggregator-api-proxy.dflow.workers.dev"
      api_swap_base: "https://quote-api.dflow.net"
      max_consecutive_failures: 0
      wait_on_429_ms: 0
      quote_config:
        use_auto_slippage: false
      swap_config:
        dynamic_compute_unit_limit: true
```

## 3. 运行流程

1. **引擎启动 (`backend = titan_dflow`)**
   - 构建 TitanWsClient：`ws_url?auth=<jwt>`。
   - 调用 `subscribe_quote_stream`，传入 `TradePair`、`TitanLeg`、`amount`。若需多腿，可多次调用。
   - 并行初始化 DFlow HTTP 客户端。

2. **接收 Titan 推送**
   - `TitanQuoteStream::recv` 返回 `TitanQuoteUpdate { seq, quotes }`。
   - 根据最新 `SwapQuotes`：
     - 选取 top provider（或多 provider 并行对比），记录 `provider_id` 与 `in/out amount`。
     - 若 Titan leg 为 `ExactIn`：使用 Titan out 作为 DFlow in。
     - 若 Titan leg 为 `ExactOut`：Titan in 作为 DFlow 目标 out。

3. **触发 DFlow 报价**
   - 构造 `/quote` 请求：`input_mint`、`output_mint`、`amount` 与 Titan leg 对应。
   - 成功则返回 `QuoteResponse`；若失败 → 标记 Titan 报价为不可用（冷却或降级）。

4. **收益计算**
   - Titan leg：`amount_titan_in/out`（根据 leg 方向调整）。
   - DFlow leg：`amount_dflow_in/out`。
   - 计算闭环收益：`out_total - in_total - fee`。需考虑：
     - Titan 返还或手续费；
     - DFlow 平台费；
     - 交易落地的 gas / Jito tip。

5. **生成落地指令**
   - 当净收益 > 阈值 → 调用 DFlow `/swap-instructions`，获得落地指令。
   - Titan leg 需根据 provider 类型决定落地方式（目前 Titan 推送多为 RFQ / 聚合器，需要确认是否提供原生落地指令；若没有，则该方案仅需 Titan 报价协助判断，不直接落 Titan 交易）。

6. **交易提交**
   - 组合 DFlow 提供的 `swap_instruction` + 本地账户准备指令构成 Solana 交易。
   - 若 Titan leg 需要对冲（如 Titan 仅提供 off-chain 成交），应转交各自落地器。

## 4. 组合策略细节

- **腿方向约束**：Titan leg 设置 `ExactIn`（买进 base），DFlow leg 设置 `ExactOut`（卖出 base），形成 `base → quote → base` 回路；或根据策略切换方向。
- **订阅数量**：初版先支持单 Pair；后续可扩展为多条 Titan stream + 多 DFlow 请求并发。
- **滑点与限额**：
  - Titan `reverse_slippage_bps`：用于反向腿 (ExactOut) 时的容忍度；
  - DFlow `slippage_bps`：根据 Titan leg 的滑点和策略配置设定。
- **节流与防抖**：
  - Titan 推送频率较高，需对同 seq / 相似 out 的重复更新做去重；
  - DFlow 报价有速率限制，必要时加冷却或使用批量节流器。

## 5. 错误处理与重连

| 场景                       | 处理策略                                                                 |
|----------------------------|--------------------------------------------------------------------------|
| Titan WebSocket 断开       | 指数退避重连；保持 `stream_id` 失效后从头订阅；日志/metrics 告警。        |
| Titan 推送解析失败         | 跳过当前更新，采集 `galileo_titan_parse_error_total`。                    |
| DFlow `/quote` 超时/失败   | 标记 Titan 报价不可用，统计失败原因，过 1-2 个推送周期恢复。              |
| DFlow `/swap-instructions` | 失败时记录失败原因，并避免重复触发（防止无效落地）。                      |
| Titan + DFlow 结果不一致   | 建立 sanity check（例如 Titan out ≈ DFlow in ± ε），超出范围则丢弃。     |

## 6. 观测指标

建议新增以下 metrics / tracing：

- `galileo_titan_quote_seq`（Gauge）：最新 Titan seq；
- `galileo_titan_best_out_amount`、`galileo_titan_leg_profit`（Histogram）；
- `galileo_titan_dflow_quote_latency_ms`（Histogram）：Titan 推送到 DFlow 报价耗时；
- `galileo_titan_dflow_profit_total{status=success/failure}`（Counter）；
- Tracing：`titan.stream.update`、`titan.dflow.quote`, `titan.dflow.swap`.

## 7. 测试计划

1. **单元测试**
   - Titan 请求构造 (`build_subscription_request`)；
   - Titan update → DFlow 请求参数映射；
   - 收益计算逻辑（模拟 Titan out + DFlow in/out）。

2. **集成测试**
   - 本地 mock Titan WS（固定推送），配合 DFlow sandbox；
   - 确认 `backend = titan_dflow` 时引擎启动、Titan 重连、DFlow 双路径成功。

3. **回归验证**
   - 仍需确认 `backend = dflow` / `jupiter` 不受影响。

## 8. 实施顺序

1. 拆分 Titan manager（已完成单流订阅能力）。
2. 扩展 `EngineBackend::TitanDflow`，写协调 Orchestrator。
3. 集成 DFlow `/quote` + `/swap-instructions`，串联收益评估。
4. 加入观测指标与错误处理。
5. 编写测试、文档与示例配置。

## 9. 后续扩展

- Titan + 其它 HTTP legs（例如 Jupiter）；
- 支持多条 Titan 流同时运行（不同流动性场景）；
- 更智能的收益预测（考虑链上 price impact、MEV 风险）。

---

如有需求变更，可在设计执行前补充讨论：例如 Titan leg 是否需要真实落地、是否支持多 provider 对冲、是否需要更多风控（黑名单、限仓等）。此文档后续会随着实现情况持续更新。***
