# Jupiter 自托管常用启动参数参考

从 `third_party` 目录中的成熟套利脚本（`run-jup.sh`、`config.yaml.example`、`mints-query.sh`）提炼出以下 Jupiter Swap API 启动要点，便于在 `galileo` 中生成或配置。

## 基础网络与监听
- `--rpc-url <URL>`：必填，指向主力 Solana RPC 节点。
- `--yellowstone-grpc-endpoint <URL>` / `--yellowstone-grpc-x-token <TOKEN>`：使用 Yellowstone Geyser 插件时必填，token 可选。
- `--host 0.0.0.0`、`--port 18080`：HTTP 监听地址与端口，第三方默认 18080。
- `--metrics-port 18081`：公开 Prometheus 指标端口，方便统一监控。

## 路由与市场配置
- `--market-cache https://cache.jup.ag/markets?v=4`：加载官方市场快照。
- `--market-mode remote`：与官方缓存实时同步（避免本地缓存积压）。
- `--enable-markets --enable-tokens`：启动启动阶段健康检查，确保市场与代币数据准备完成。
- `--allow-circular-arbitrage`：允许环形套利路径。
- `--enable-new-dexes`：在 Jupiter 侧新增交易所后自动拾取。
- `--filter-markets-with-mints <mintA,mintB,...>`：限制市场集合，第三方脚本通过 `token-cache.json` 自动生成。
- `--exclude-dex-program-ids <dexA,dexB>`：排除不想参与的 DEX。

## 性能调优
- `--total-thread-count 64`：总线程数量，第三方示例根据机器资源设置。
- `--webserver-thread-count 24`：HTTP/请求处理线程。
- `--update-thread-count 5`：市场/缓存更新线程。
- `RUST_LOG=info`：默认日志级别，可按需提高（`debug`/`trace`）排查问题。
- 系统层面配合 `ulimit -n 100000` 增大文件描述符限制。

## 代币白名单与来源
- `token-cache.json`：最终的代币列表，脚本按顺序整合自：
  1. `intermediate_tokens`：配置文件静态列表。
  2. `load_mints_from_url`：远程服务返回的代币数组。
  3. `intermediate_tokens_file`：本地 JSON 文件。
  4. `load_mints_from_birdeye_api_max_mints` + `birdeye_api_key`：从 Birdeye 拉取高流动性代币。
- `max_tokens_limit`：上限控制，超过后随机截断（始终保留 WSOL）。
- `jup_exclude_dex_program_ids`、`not_support_tokens`：过滤不希望触达的 DEX 或代币。

## 自动化与守护
- 脚本使用 `.jupiter_running` 控制监控循环，并在崩溃后重启。
- `auto_restart`：按分钟定期重启以刷新市场缓存或释放资源。
- `kill-process.sh`、`run.sh`：提供统一的进程管理入口，可借鉴其守护逻辑融入 `galileo`。

## 在 `galileo` 中的落地建议
1. 在 `JupiterConfig` 中新增（或自动填充）常见字段：`host`、`port`、`metrics_port`、线程数字段、`allow_circular_arbitrage` 等，避免手动拼接 `args`。
2. 结合 `mints-query.sh` 思路，为 `galileo` 设计代币白名单生成器或导入接口，确保 `--filter-markets-with-mints` 能自动维护。
3. 在生命周期管理阶段实现“崩溃检测 + 自动重启”，并保留 `.log` 输出以便追溯（可通过 `auto_restart_minutes`、`max_restart_attempts` 调控）。
4. 本地调试时将 `[jupiter.launch].disable_local_binary = true` 并设置 `[bot].jupiter_api_url` 指向线上 Jupiter API，可避免本地进程占用资源。
5. 针对 `metrics_port` 与健康检查路由，规划监控系统对接方式（Prometheus/OpenTelemetry）。

更多字段与第三方脚本的映射请查阅 `docs/galileo_config_reference.md`。后续阶段落地时可直接引用本列表生成默认配置或 CLI 选项提示。
