# Galileo 配置映射参考

Galileo 已移除历史 Jupiter 依赖，所有聚合器配置均通过 `galileo.yaml` 的结构化字段管理。本文按照模块梳理常用项目，便于从旧版样例或第三方脚本迁移配置。

## 1. 全局与运行环境
- `rpc_url`(s) → `[global].rpc_urls`：按顺序轮询的主链 RPC 列表。
- `yellowstone_grpc_url` / `yellowstone_grpc_token` → `[bot].yellowstone_grpc_url` / `[bot].yellowstone_grpc_token`。
- `proxy` → `[global].proxy`：支持定义命名的代理 `profiles`，并在 `enable.<module>` 中引用（例如 `quote`、`lander`）；`per_request: true` 可强制每次请求重建连接以配合旋转代理。旧的单字符串写法仍视作 `default` 兜底，且各引擎的 `engine.<backend>.api_proxy` 依旧可以局部覆盖。
- 加密钱包 → `[global.wallet.wallet_keys]`：列表项格式 `- "<备注>": "<base64 密文>"`；当列表为空时，启动 Galileo 会提示录入三段私钥并写回配置。
- `cpu_affinity` 相关参数 → `[bot.cpu_affinity]`：绑定 Tokio runtime 到指定 CPU，减少与 RPC 节点抢占。
- `quote_ms` / `swap_ms` / `landing_ms` → `[engine.time_out]`：统一的报价、指令和落地超时配置（毫秒）。
- `strategies.enabled` → `[bot.strategies.enabled]`：集中声明启用的策略标签（`blind_strategy` / `pure_blind_strategy` / `copy_strategy` / `back_run_strategy` 等）。
- `engines.pairs` → `[bot.engines.pairs]`：配置 multi-leg 组合允许的买/卖腿来源。
- `flashloan.products` → `[bot.flashloan.products]`：启用的闪电贷协议列表，同时在此处标注 `prefer_wallet_balance` 等偏好。
- `[bot.dry_run]`：包含 `enable` 与 `rpc_url`。启用后，所有策略的 RPC 调用与落地请求会改用该节点，并强制落地器退化为 RPC，适合在本地 devnet/sandbox 回放交易。

## 2. 聚合器引擎
`[engine.backend]` 支持 `jupiter` / `dflow` / `ultra` / `kamino` / `multi-legs` / `none`，各自子表涵盖 API 端点、代理以及并发参数：

- `[engine.console_summary]`：`enable` 控制是否开启控制台机会摘要面板；启用后，每轮 trade size 批次结束会输出机会数、延迟与落地统计（当前仅适用于单引擎 Quote 流程），默认关闭以保持日志精简。
- `[engine.dflow]`
  - `api_quote_base` / `api_swap_base`：DFlow Quote 与 Swap API 基址。
  - `api_proxy`：可选，覆盖全局代理。
  - `quote_config.cadence.default.group_parallelism`：`"auto"`（按 IP 资源动态并发）或正整数；`intra_group_spacing_ms` 控制同批次 quote 组的启动间隔，`wave_cooldown_ms` 定义下一轮批次的最小冷却时间。
  - `quote_config.cadence.per_base_mint`：可选，针对特定 base mint 覆盖默认节奏（同字段含义）；未配置时沿用 `default`。
  - `swap_config.cu_limit_multiplier`：对 `/swap-instructions` 返回的 compute unit limit 乘以系数重写。

- `[engine.ultra]`
  - `api_quote_base` / `api_swap_base`、`api_proxy` 同上。
  - `quote_config.use_wsol`、`taker`、`include_routers` / `exclude_routers`：构造 `/order` 请求所需的额外参数。
  - `swap_config.cu_limit_multiplier`：Ultra 交易落地前的 compute budget 调整。

- `[engine.kamino]`
  - `quote_config.cu_limit_multiplier`：按倍数缩放 `/kswap/all-routes` 的 compute unit limit。
  - `quote_config.resolve_lookup_tables_via_rpc`：`true` 时只返回 lookup table key，装配阶段改由我们主动拉取账户列表。

- `[engine.multi_leg]`
  - `backend = "multi-legs"` 时启用多腿组合。具体腿角色由 `engine.jupiter`（多实例场景可写成 `jupiter_buy:` / `jupiter_sell:` 等子项）/ `engine.dflow.leg` / `engine.ultra.leg` / `engine.titan.leg` 指定，是否注册则由 `bot.engines.pairs` 控制；同时需在 `bot.strategies.enabled` 中包含 `blind_strategy` 以提供交易对。
  - 当需要为 Jupiter 声明多条腿时，可在 `engine.jupiter` 下改写为：
    ```yaml
    engine:
      jupiter:
        jupiter_buy:
          leg: "buy"
          api_quote_base: "..."
          api_swap_base: "..."
        jupiter_sell:
          leg: "sell"
          api_quote_base: "..."
          api_swap_base: "..."
    ```
    旧版单实例写法仍然兼容（直接在 `engine.jupiter` 中配置字段）。
  - 各聚合器的 `swap_config.wrap_and_unwrap_sol`、`dynamic_compute_unit_limit` 等选项会注入 `MultiLegEngineContext`，直接影响组合装配。

## 3. Titan / Flashloan / 上链器
- Titan 参数集中在 `[engine.titan]`：包含 WS 端点、JWT、推送节奏以及允许的路由列表。
- 闪电贷配置 → `[flashloan.marginfi]`：保留 Marginfi 账户、CU 开销等细节；启停及是否优先使用钱包余额由 `bot.flashloan` 统一驱动。
- 各类上链器（Jito、Staked、Temporal、Astralane）已拆分到 `lander.yaml`，Galileo 会在与主配置相同的目录下自动加载。

## 4. 策略层
- 盲发 → `[blind_strategy]`：`base_mints`/`lanes`、`min_quote_profit`、`enable_dexs` 等字段与旧版策略一一对应（启停由 `bot.strategies.enabled` 控制）。
- Copy → `[copy_strategy]`：`pull_interval_minutes`、`queue_capacity`、`queue_worker_count` 控制拉取频率与排队行为。
- Back-run → `[back_run_strategy]`：`base_mints[*].trade_configs`、`min_quote_profit` 等字段沿用原始命名。

## 5. 监控与日志
- Prometheus → `[bot.prometheus]`：`enable` 与 `listen` 控制 `/metrics` 暴露地址。
- 日志 → `[global.logging]`：`level`、`json`、`profile`、`slow_quote_warn_ms`、`timezone_offset_hours` 分别映射旧版的运行日志选项。

## 6. 迁移建议
1. 逐步补全 `rpc_urls`、代理与 YellowStone 等敏感信息，确认网络访问路径可达。
2. 将旧版 `intermediate_tokens` 文件转换为 `blind_strategy.base_mints[*].intermediate_mints` 或按需落入 DFlow/Kamino 白名单；若仍使用脚本生成 `token-cache.json`，请同步更新配置引用。
3. 若需要分环境部署，建议在 `config/` 目录维护多份 `galileo.<env>.yaml`，通过 CLI `--config` 指定。

截至当前版本，Galileo 默认不再启动本地 Jupiter 进程，所有聚合器调用均直接走远端 API。若外部流程仍依赖旧的 Jupiter 启动脚本，请更新文档或脚本指向新的配置路径。
