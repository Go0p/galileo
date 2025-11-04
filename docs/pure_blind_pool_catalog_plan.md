# 纯盲发池子画像方案

> 目标：基于实时监听的 Jupiter 成功交易，维护可用于纯盲发的池子画像，并在满足策略阈值时自动触发盲发。在保证零成本抽象和性能优先的前提下，将新能力融入现有 `pure_blind` 架构。

## 1. 现状梳理
- 纯盲发策略目前由 `src/strategy/pure_blind/runner.rs:35` 的 `PureBlindRouteBuilder` 在启动时读取市场缓存、生成固定 `BlindRoutePlan`，`PureBlindStrategy` 在 `on_market_event` 中按照固定路由推送 `BlindOrder`（`src/strategy/pure_blind/runner.rs:1380`）。
- Copy 策略通过 Yellowstone gRPC 监听钱包交易，但目前仅在 `src/strategy/copy/transaction.rs` 内做轻量指令解析，未复用 `carbon-jupiter-swap-decoder`，也没有沉淀池子统计。
- Jupiter 指令 ABI 已在 `idls/jup6.json`、`src/instructions/jupiter/types.rs` 中建模，我们可以直接引入 `carbon-jupiter-swap-decoder` 获得开箱即用的 `RoutePlanStep`/`Swap` 解析。

## 2. 新增模块分层
```
src/strategy/pure_blind/
├── observer/
│   ├── mod.rs
│   ├── listener.rs           # gRPC 订阅 + 交易过滤
│   ├── decoder.rs            # 调用 carbon-jupiter-swap-decoder 解析 RoutePlanStep
│   ├── profile.rs            # PoolKey / PoolProfile / stats 累计
│   ├── scorer.rs             # 权重/阈值判定
│   └── bridge.rs             # 将画像转成 BlindStep/BlindOrder
└── pool_catalog.rs           # Arc<PoolCatalog> 对外接口（供 PureBlindStrategy 使用）
```
- `observer` 子模块负责实时监听与画像维护；`pool_catalog.rs` 提供线程安全的查询/订阅接口，策略层仅依赖该抽象，符合守则“策略与引擎解耦”。
- `bridge.rs` 将 `PoolProfile` 与现有 `ResolvedMarketMeta`（`src/strategy/pure_blind/runner.rs:1367`）打通，重用已有的 `BlindStep` 构造逻辑。

## 3. 数据流设计
1. **监听**：`listener.rs` 复用 Yellowstone gRPC 客户端（参考 `src/strategy/copy/grpc.rs`），配置支持多钱包/全网过滤。监听到 `SubscribeUpdateTransactionInfo` 后，先按 Jupiter Program + 指令长度做快速过滤。
2. **解码**：`decoder.rs` 将交易中的 `RouteV2`/`SharedRouteV2` 数据喂给 `carbon_jupiter_swap_decoder::JupiterSwapDecoder`，生成 `RoutePlanStep` 列表。借助 `Swap` 枚举内的方向字段（如 `Whirlpool { a_to_b }`）和 `input_index/output_index`，锁定池子账号与方向。
3. **画像归一**：`profile.rs` 构建 `PoolKey = (dex_program_id, pool_state, input_mint, output_mint, swap_variant)`，同时记录：
   - `pool_accounts: SmallVec<Pubkey>`（保持 accounts 顺序）
   - `user_input/output_ata`、`source/destination_mint`
   - 成功统计：`hit_count`、`last_slot`、`last_latency`、`net_profit`（通过 `pre/post token balances` 估算）
   - 失败统计：来自盲发执行反馈
4. **权重判定**：`scorer.rs` 提供 `ScoringPolicy` trait，根据配置 `min_hits`、`max_slot_gap`、`min_profit_lamports`、`decay_half_life` 等计算动态权重。满足阈值的画像会进入 `PoolCatalog::activate`.
5. **执行桥接**：`bridge.rs` 在激活时调用现有 `resolve_market_meta`（`src/strategy/pure_blind/runner.rs:1120` 起）获取 `BlindMarketMeta`，将画像转成单步或多步的 `BlindRoutePlan`：
   - 单步池（常见于纯 copy swap）直接构造一腿路线。
   - 若 `RoutePlanStep` 中存在多段 CPI（例如 Meteora + Whirlpool），按 step 顺序生成多腿路由。
6. **策略消费**：`PureBlindStrategy` 改为持有 `StaticRoutes + Arc<PoolCatalog>`，`on_market_event` 时从 catalog 读取当前活跃画像生成 `BlindOrder`，与既有静态路由一起打乱推送。

## 4. 配置与文档
- 在 `src/config/types.rs` 新增：
  - `PureBlindObserverConfig`（gRPC endpoint/token、监听钱包、过滤 DEX Program、并发度）
  - `PureBlindActivationPolicy`（命中阈值、利润阈值、衰减参数、最大并发盲发数、是否跟踪失败回退时间）
  - `PureBlindPoolRetention`（最大画像数、淘汰策略）
- `PureBlindStrategyConfig` 增加 `observer: Option<PureBlindObserverConfig>` 与 `activation: PureBlindActivationPolicy`，保持默认关闭。
- 更新 `docs/strategy_arch.md` 与 `docs/blind_strategy.md`，新增本文档引用，介绍如何启用监听式纯盲发、如何刷新池子画像。

## 5. 监控与性能
- 新增 metrics（`src/monitoring/events.rs`）：
  - `galileo_pure_blind_pool_observed_total{dex,pool}`：监听到的新池子
  - `galileo_pure_blind_pool_active_total{dex}`：当前激活画像
  - `galileo_pure_blind_blindfire_success_total/failed_total{dex,reason}`
  - `galileo_pure_blind_pool_decay_total{cause}`：淘汰原因
- 热路径（解析/权重计算/下发）用 `hotpath::measure_block!` 包裹，确保守则要求的性能可观测性。
- `PoolCatalog` 内部数据结构采用 `dashmap::DashMap` 或 `parking_lot::RwLock<HashMap<...>>`；权重计算使用 `SmallVec`/`VecDeque` 避免堆分配。

## 6. 与落地器集成
- `bridge.rs` 在生成 `BlindOrder` 时复用现有 `BlindOrder.lookup_tables` 字段：从监听的交易里提取 `AddressLookupTable`（若存在）并缓存。
- 盲发执行失败（`EngineResult::Err`）由 `StrategyEngine` 回调时写回 `PoolCatalog::record_failure`，支持指数退火。
- `StrategyEngine` 维持现有 Quote → Dispatch → Lander 流程，无需修改 `engine::context`，但需要在 `Action::DispatchBlind` 分支中区分 “静态” 与 “动态”来源以便监控。

## 7. 交付节奏建议
1. **解析 & 画像落地**（Stage 1）：实现 `observer::*`，提供 CLI 子命令离线验证（输入交易 JSON → 输出 PoolProfile）。
2. **Catalog + 策略联动**（Stage 2）：改造 `PureBlindStrategy` 接受动态画像，完成与现有执行链路的整合，补测试。
3. **监控 & 文档**（Stage 3）：补 metrics、Prometheus 配置说明，更新 `docs/` 与示例配置，准备 `galileo.yaml` 样例。
4. **扩展评估**（Stage 4，可选）：引入收益回放、模拟回测工具，或与 copy 策略共享监听管道。

> 该方案利用现有纯盲发基础设施，只在策略层新增可插拔的池子索引器；未来如需支持更多 DEX 变体，只需扩展 `bridge.rs` 中的 `SwapVariant → BlindDex` 映射即可。

