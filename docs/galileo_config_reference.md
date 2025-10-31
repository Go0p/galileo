# Galileo 配置映射参考

本页整理了 `third_party/config.yaml.example` 中常用的 Jupiter 参数，并映射到 `galileo` 的结构化配置，以便快速完成自托管环境的迁移。示例配置可直接参考根目录下的 `galileo.yaml`。

## 网络与监听
- `rpc_url` → `[global].rpc_urls`（策略运行时访问主链 RPC，支持配置多端点自动轮询）
- `yellowstone_grpc_url`/`yellowstone_grpc_token` → `[bot].yellowstone_grpc_url` / `[bot].yellowstone_grpc_token`
- Titan WS 相关配置 → `[galileo.engine.titan]`（含 `enable` / `ws_url` / `default_pubkey` / `jwt` / `providers` / `reverse_slippage_bps` / `interval_ms` / `num_quotes`）
  - `interval_ms` / `num_quotes`：控制 Titan WS 推送频率与单次返回的报价数量，未指定时使用服务端默认值。
- `jupiter_api_url` → `[bot].jupiter_api_url`（禁用本地 Jupiter 时指向在线 API）
- `jupiter_local_port` → `[jupiter.launch].port`
- `jup_bind_local_host` → `[jupiter.launch].host`
- `jupiter_disable_local` → `[jupiter.launch].disable_local_binary`
- `auto_restart` → `[jupiter.process].auto_restart_minutes`
- `max_retries`（如有）→ `[jupiter.process].max_restart_attempts`

## 钱包安全
- 首次启动时可在 `global.wallet.private_key` 写入明文密钥，程序会提示设置钱包密码并使用 Argon2id + AES-256-GCM 生成同级目录下的 `wallet.enc`，随后自动清空配置中的明文。
- 若检测到 `wallet.enc`，预检前会提示输入密码解锁；当前版本仅支持交互式输入，不再提供环境变量绕过。
- 忘记密码时只能删除 `wallet.enc` 并重新在配置中填入私钥创建新的加密文件，请妥善保管备份。

## 市场与代币
- `jupiter_market_mode` → `[jupiter.launch].market_mode`
- `jup_exclude_dex_program_ids` → `[jupiter.tokens].exclude_dex_program_ids`
- `intermediate_tokens`/`intermediate_tokens_file`/`load_mints_from_url` → 结合脚本生成 `filter_markets_with_mints`，填入 `[jupiter.tokens].filter_markets_with_mints`
- `not_support_tokens` → 可并入 `exclude_dex_program_ids` 或下游白名单管理
- 运行时动态添加市场 → `[jupiter.launch].enable_add_market`

生成 `token-cache.json` 的流程可继续沿用 `third_party/mints-query.sh`，执行后把输出写入 `galileo` 的白名单列表。

## 性能相关
- `total_thread_count` → `[jupiter.performance].total_thread_count`
- `jupiter_webserver` → `[jupiter.performance].webserver_thread_count`
- `jupiter_update` → `[jupiter.performance].update_thread_count`
- `jupiter_skip_user_accounts_rpc_calls` → `[jupiter.launch].skip_user_accounts_rpc`
- Bot/节点共机隔离 → `[galileo.bot.cpu_affinity]`（`enable`、`worker_cores`、`max_blocking_threads`、`strict` 分别控制是否启用绑定、Tokio worker 对应的核心列表、阻塞线程池上限以及拓扑校验策略）
- 报价 / 指令 / 落地超时 → `[galileo.bot.quote_ms]`、`[galileo.bot.swap_ms]`、`[galileo.bot.landing_ms]`（单位：毫秒；`swap_ms` 省略时继承 `quote_ms`）
- `random_engine`、`max_concurrent` 等策略参数仍由套利逻辑控制，`galileo` 侧保留 `jupiter.extra_args` 以便继续传入实验性开关。
- 钱包余额刷新 → `[galileo.bot].auto_refresh_wallet_minute`（`0` 表示禁用；启用后定期刷新身份余额与 ATA 缓存，对应指标 `galileo_copy_wallet_refresh_total`、`galileo_copy_wallet_accounts`）
- Copy 队列并行度 → `[galileo.copy.copy_dispatch].queue_worker_count`（默认 `1`，大于 1 时同时消费排队任务），配合 `queue_capacity` / `queue_send_interval_ms` 调整节奏；对应指标 `galileo_copy_queue_depth`、`galileo_copy_queue_dropped_total`、`galileo_copy_queue_workers` 观测排队情况。
- CPU 密集型任务并行度 → 环境变量 `GALILEO_RAYON_THREADS`（可选），用于覆盖多腿收益评估等 rayon 线程池的线程数；默认为物理核心数。

## 守护与监控
- `.jupiter_running`/`kill-process.sh` 中的重启机制 → `[jupiter.process]`，后续由 `JupiterBinaryManager` 统一调度；可通过 `auto_restart_minutes` + `max_restart_attempts` 控制重启频率与次数
- `jupiter-api.log` 的日志级别 → `[jupiter.environment].RUST_LOG`
- `--metrics-port`/`--enable-markets --enable-tokens` 已纳入 `effective_args`，默认开启 Prometheus 指标与市集加载检查。
- 日志输出策略 → `[galileo.global.logging]`：
  - `profile`：`lean`（默认）只保留关键信息，`verbose` 打开调试细节。
  - `slow_quote_warn_ms` / `slow_swap_warn_ms`：配置慢请求阈值，超限时会额外落 Warn 日志并计入指标。
  - `timezone_offset_hours`：日志时间的 UTC 偏移量（单位：小时），默认 `0`；例如填写 `8` 即输出为北京时间。

## 上链器与小费
上链器（Jito、Staked、Temporal、Astralane 等）以及 compute unit price/tip 策略已拆分到独立的 `lander.yaml`，程序会在 `galileo.yaml` 所在目录或 `config/` 目录中自动加载该文件，可直接复制模板并按需扩展字段。

## 高性能默认值
`galileo` 会自动生成以下核心启动参数：

```text
--market-cache https://cache.jup.ag/markets?v=4
--market-mode remote
--allow-circular-arbitrage
--enable-new-dexes
--enable-add-market（当 `[jupiter.launch].enable_add_market = true` 时添加）
--expose-quote-and-simulate
--enable-markets --enable-tokens
--skip-user-accounts-rpc-calls
--total-thread-count {>=1}
--webserver-thread-count {<= total_thread_count}
--update-thread-count {>=1}
```

如需附加实验标志，可在 `galileo.yaml` 中设置：

```yaml
jupiter:
  extra_args:
    - "--flag-a"
    - "--flag-b=value"
```

## 套利策略配置
- `BLIND_QUOTE_STRATEGY.base_mints` → `blind_strategy.base_mints`
- `BLIND_QUOTE_STRATEGY.*.trade_size_range` → `blind_strategy.base_mints[*].lanes`
- `BLIND_QUOTE_STRATEGY.*.trade_range_strategy` → `blind_strategy.base_mints[*].lanes[*].strategy`
- `BLIND_QUOTE_STRATEGY.*.min_quote_profit` → `blind_strategy.base_mints[*].min_quote_profit`
- `BACKRUN_STRATEGY.base_mints` → `back_run_strategy.base_mints`
- `BACKRUN_STRATEGY.*.trade_configs` → `back_run_strategy.base_mints[*].trade_configs`

## 闪电贷
- `flashloan.enable`：全局开关，置为 `true` 后会自动确保 Marginfi account 并在需要时注入闪电贷指令。
- `flashloan.prefer_wallet_balance`：启用后先检查钱包余额，如果交易规模在余额范围内就直接走自有资金，超出部分才触发闪电贷。
- `flashloan.marginfi.compute_unit_overhead`：闪电贷指令额外消耗的 compute unit（默认为 `110000`），交易落地时会在 `/swap-instructions` 返回值基础上累加该开销。

Marginfi 的账户创建、Bank 映射、借款规模等均内置处理：首次启动会根据钱包自动创建 Marginfi account，后续复用；借款金额默认等于策略的交易规模。

## DFlow 引擎参数
- `[galileo.engine.dflow.swap_config].cu_limit_multiplier`：对 `/swap-instructions` 返回的 compute unit limit 乘以系数后再落地，默认 `1.0`，可用于收敛 DFlow 过于保守的估算；若填入 `0 < value < 1` 表示收缩，`value > 1` 则放大开销。

## Kamino 引擎参数
- `[galileo.engine.kamino.quote_config].cu_limit_multiplier`：对 `/kswap/all-routes` 返回的 compute budget 指令中的 compute unit limit 乘以系数并写回，默认 `1.0`；用于在保存吞吐的前提下压缩或放大 Kamino 给出的上限估计。
- `[galileo.engine.kamino.quote_config].parallelism`：Kamino 报价请求并发度（同 Jupiter/DFlow 语义），支持 `"auto"` 或正整数。
- `[galileo.engine.kamino.quote_config].batch_interval_ms`：Kamino 报价批次之间的最小间隔，毫秒；`0` 表示不额外延迟。

## 后续建议
1. 根据环境补全 RPC、Yellowstone、Jito 等敏感信息。
2. 将 `token-cache.json` 的生成流程移植为 Rust 子命令或定时任务，保持和 `filter_markets_with_mints` 同步。
3. 若需要多环境差分，可把 `galileo.yaml` 拆分为 `config/galileo.{prod,dev}.yaml` 并通过启动参数覆盖。
