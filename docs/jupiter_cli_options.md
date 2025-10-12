# Jupiter Swap API 启动参数速查

本文基于 `./jupiter-swap-api --help` 输出整理常用启动参数，结合自托管场景给出中文说明。所有参数均可通过环境变量传入（示例见括号中的 `env:` 标注），便于容器或系统服务配置。

## 核心要求

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `--rpc-url <RPC_URL>` | **必填** `RPC_URL` | 主 RPC，用于轮询账户、抓取报价依赖数据。建议使用低延迟、高吞吐的专线或自建节点。 |
| `--market-cache <MARKET_CACHE>` | 自动匹配 `market-mode`；`MARKET_CACHE` | 市场快照来源，可指定远程 URL、本地文件或 europa 接口。未配置时会使用 `market-mode` 的默认值（例如 `https://cache.jup.ag/markets?v=4`）。 |
| `--market-mode <europa|remote|file>` | `europa`；`MARKET_MODE` | 市场更新策略：`europa` 实时推送、`remote` 使用固定快照、`file` 读取本地文件。自托管/离线环境推荐 `remote` 或 `file`。 |
| `--secondary-rpc-urls ...` | —；`SECONDARY_RPC_URLS` | 辅助 RPC 列表，部分耗时操作可分摊到副本节点。 |

## Yellowstone gRPC 相关

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `-e, --yellowstone-grpc-endpoint <URL>` | —；`YELLOWSTONE_GRPC_ENDPOINT` | Yellowstone Geyser 入口（如 `https://jupiter.rpcpool.com`）。启用后会通过 Geyser 实时获取 AMM 变更。 |
| `-x, --yellowstone-grpc-x-token <TOKEN>` | —；`YELLOWSTONE_GRPC_X_TOKEN` | Yellowstone 的认证 Token。 |
| `--yellowstone-grpc-enable-ping` | 关闭；`YELLOWSTONE_GRPC_ENABLE_PING` | 为负载均衡节点启用 ping，避免长连被踢。 |
| `--yellowstone-grpc-compression-encoding <none|gzip|zstd>` | `gzip`；`YELLOWSTONE_GRPC_COMPRESSION_ENCODING` | gRPC 压缩方式。网络环境良好可维持默认 gzip。 |
| `--snapshot-poll-interval-ms <MS>` | Geyser 模式默认 `30000`；`SNAPSHOT_POLL_INTERVAL_MS` | 定期快照 AMM 状态的轮询间隔，数值越小越实时，负载越高。 |
| 连接参数（`--yellowstone-grpc-setting-*` 系列） | 详见帮助 | 控制 gRPC 连接的超时、窗口、keepalive 等细粒度行为。默认配置已满足大部分使用场景，仅在超时频繁、需要精细调优时修改。 |

## 市场 / DEX 过滤

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `--dex-program-ids ...` | —；`DEX_PROGRAM_IDS` | 指定仅加载的 DEX Program 列表。 |
| `--exclude-dex-program-ids ...` | —；`EXCLUDE_DEX_PROGRAM_IDS` | 排除不需要的 DEX。 |
| `--filter-markets-with-mints ...` | —；`FILTER_MARKETS_WITH_MINTS` | 仅保留涉及给定 mint 集合的市场，常与套利白名单结合使用。 |
| `--enable-new-dexes` | 关闭；`ENABLE_NEW_DEXES` | 启用最新集成但可能尚未充分验证的 DEX。 |
| `--allow-circular-arbitrage` | 关闭；`ALLOW_CIRCULAR_ARBITRAGE` | 允许输入/输出 mint 相同的循环套利路径。配合策略使用。 |

## HTTP 服务与线程

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `-H, --host <HOST>` | `0.0.0.0`；`HOST` | HTTP 监听地址。 |
| `-p, --port <PORT>` | `8080`；`PORT` | HTTP 服务端口。 |
| `--metrics-port <PORT>` | 未启用；`METRICS_PORT` | Prometheus `/metrics` 暴露端口。 |
| `--total-thread-count <N>` | `8`；`TOTAL_THREAD_COUNT` | 进程可用线程总数。需要根据机器核数配置。 |
| `--webserver-thread-count <N>` | `2`；`WEBSERVER_THREAD_COUNT` | HTTP 线程数量。请求量大时适当增大。 |
| `--update-thread-count <N>` | `4`；`UPDATE_THREAD_COUNT` | 负责更新市场与账户的线程数量。 |

## 可选 REST 接口开关

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `-s, --expose-quote-and-simulate` | 关闭 | 开启 `/quote-and-simulate` 快速调试接口。 |
| `--enable-deprecated-indexed-route-maps` | 关闭 | 生成旧版路由映射，开销较大，不建议在生产启用。 |
| `--enable-diagnostic` | 关闭 | 开放 `/diagnostic` 调试端点。 |
| `--enable-add-market` | 关闭 | 支持运行时通过 API 添加新市场（实验特性）。 |
| `--enable-tokens` / `--enable-markets` | 关闭 | 延续旧版 `/tokens`、`/markets` 接口。 |

## 监控与日志

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `--environment <ENV>` | `production`；`ENVIRONMENT` | 仅用于标记环境名称。 |
| `--sentry-dsn <DSN>` | —；`SENTRY_DSN` | 将错误上报到 Sentry。 |
| `--loki-url` / `--loki-username` / `--loki-password` | — | Loki 日志聚合配置。 |
| `--loki-custom-labels KEY=VALUE ...` | — | 附加 Loki 标签，如 `APP_NAME=jupiter-swap-api`。 |

## 运行时控制

| 参数 | 默认值 / 环境变量 | 说明 |
| --- | --- | --- |
| `--enable-external-amm-loading` | 关闭 | 允许通过 keyedUiAccounts 载入外部 AMM。 |
| `--disable-swap-cache-loading` | 关闭 | 禁用 swap 所需缓存，可用于纯 quote API。 |
| `--geyser-streaming-chunk-count <N>` | `12`；`GEYSER_STREAMING_CHUNK_COUNT` | Geyser 推流时的分块数量。 |
| `--rtse-url <URL>` | —；`RTSE_URL` | 预留的外部实时数据服务入口。 |

## 其他

| 参数 | 说明 |
| --- | --- |
| `-h, --help` | 显示完整帮助。 |
| `-V, --version` | 输出二进制版本（例如 `jupiter-swap-api 6.0.62 <commit>`）。 |

---

### 推荐的最简自托管参数集

```bash
./jupiter-swap-api \
  --rpc-url https://your-rpc.example \
  --market-mode remote \
  --market-cache https://cache.jup.ag/markets?v=4 \
  --allow-circular-arbitrage \
  --enable-new-dexes \
  --enable-markets \
  --enable-tokens
```

根据机器资源与策略特性，再增减线程数、过滤的 DEX、Loki/Sentry 等附加参数。更多配置映射可参见 `docs/jupiter_launch_params.md` 与 `galileo.yaml` 模板。***
