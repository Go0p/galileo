# Galileo 配置映射参考

本页整理了 `third_party/config.yaml.example` 中常用的 Jupiter 参数，并映射到 `galileo` 的结构化配置，以便快速完成自托管环境的迁移。示例配置可直接参考根目录下的 `galileo.toml`。

## 网络与监听
- `rpc_url` → `[jupiter.launch].rpc_url`
- `jupiter_local_port` → `[jupiter.launch].port`
- `jup_bind_local_host` → `[jupiter.launch].host`
- `jupiter_disable_local` → `[jupiter.launch].disable_local_binary`
- `yellowstone_grpc_url`/`yellowstone_grpc_token` → `[jupiter.launch.yellowstone]`
- `jupiter_api_url` → `[http].base_url`（用于客户端引用）
- `auto_restart` → `[jupiter.process].auto_restart_minutes`
- `max_retries`（如有）→ `[jupiter.process].max_restart_attempts`

## 市场与代币
- `jupiter_market_mode` → `[jupiter.launch].market_mode`
- `jup_exclude_dex_program_ids` → `[jupiter.tokens].exclude_dex_program_ids`
- `intermediate_tokens`/`intermediate_tokens_file`/`load_mints_from_url` → 结合脚本生成 `filter_markets_with_mints`，填入 `[jupiter.tokens].filter_markets_with_mints`
- `not_support_tokens` → 可并入 `exclude_dex_program_ids` 或下游白名单管理

生成 `token-cache.json` 的流程可继续沿用 `third_party/mints-query.sh`，执行后把输出写入 `galileo` 的白名单列表。

## 性能相关
- `total_thread_count` → `[jupiter.performance].total_thread_count`
- `jupiter_webserver` → `[jupiter.performance].webserver_thread_count`
- `jupiter_update` → `[jupiter.performance].update_thread_count`
- `jupiter_skip_user_accounts_rpc_calls` → `[jupiter.launch].skip_user_accounts_rpc`
- `random_engine`、`max_concurrent` 等策略参数仍由套利逻辑控制，`galileo` 侧保留 `jupiter.extra_args` 以便继续传入实验性开关。

## 守护与监控
- `.jupiter_running`/`kill-process.sh` 中的重启机制 → `[jupiter.process]`，后续由 `JupiterBinaryManager` 统一调度；可通过 `auto_restart_minutes` + `max_restart_attempts` 控制重启频率与次数
- `jupiter-api.log` 的日志级别 → `[jupiter.environment].RUST_LOG`
- `--metrics-port`/`--enable-markets --enable-tokens` 已纳入 `effective_args`，默认开启 Prometheus 指标与市集加载检查。

## 高性能默认值
`galileo` 会自动生成以下核心启动参数：

```text
--market-cache https://cache.jup.ag/markets?v=4
--market-mode remote
--allow-circular-arbitrage
--enable-new-dexes
--expose-quote-and-simulate
--enable-markets --enable-tokens
--skip-user-accounts-rpc-calls
--total-thread-count {>=1}
--webserver-thread-count {<= total_thread_count}
--update-thread-count {>=1}
```

如需附加实验标志，可在 `galileo.toml` 中设置 `jupiter.extra_args = ["--flag-a", "--flag-b=value"]`。

## 套利策略配置
- `base_mint`/`intermediate_tokens` → `[strategy].base_mint` 与 `[strategy].quote_mints`
- `trade_range` / `trade_range_strategy` → `[strategy].trade_range` / `[strategy].trade_range_strategy`
- `min_profit_threshold` → `[strategy].min_profit_threshold_lamports`
- `max_tip_lamports` → `[strategy].max_tip_lamports`
- `bot_config` → `[strategy.bot]`（`enable_reverse_trade`、`over_trade_process_delay_ms`、`static_tip_config` 等）
- `blind_config` → `[strategy.blind]`
- `spam_config` → `[strategy.spam]`
- `jito_engine`/`random_engine` → `[strategy.jito].engine_urls`、`random_engine`
- `private_key`/`jito tip account` → `[strategy.identity].user_pubkey`、`tip_account`

## 后续建议
1. 根据环境补全 RPC、Yellowstone、Jito 等敏感信息。
2. 将 `token-cache.json` 的生成流程移植为 Rust 子命令或定时任务，保持和 `filter_markets_with_mints` 同步。
3. 若需要多环境差分，可把 `galileo.toml` 拆分为 `config/galileo.{prod,dev}.toml` 并通过启动参数覆盖。
