# 纯盲发池子与路线持久化设计

> 目标：让纯盲策略在冷启动时无需静态路由即可运行，并确保池子/路线画像在长期运行中保持有序淘汰与持久化。

## 1. 持久化总览

- 新增运行期缓存目录，例如 `monitoring/pure_blind_cache/`（可配置）。
- 维护两份快照文件：
  - `pools.json`：记录活跃池子画像（池子地址、币对、最近命中等）。
  - `routes.json`：记录闭环路线（步骤列表、使用的 ALT、公用指标等）。
- 提供 `load_snapshot` / `export_snapshot` API，使 `PoolCatalog` 与 `RouteCatalog` 能在启动时预热、运行时落盘。
- 快照格式含 `version`、`generated_at` 与 `entries`，便于兼容升级；写文件采用 “临时文件 + 原子替换” 防止半写，CLI 退出前会执行一次最终 flush。

## 2. 启动流程

1. 从配置读取缓存目录与开关；若持久化关闭，直接跳过。
2. 依次读取 `pools.json` 与 `routes.json`：
   - 校验版本与 `generated_at` 是否在 TTL 内（默认 3600 秒）。
   - 解析为 `SnapshotPool` / `SnapshotRoute` 结构体，逐条调用 `ingest_snapshot` 进入 catalog。
3. 若启用了 observer，继续并行监听实时交易，快照只作为冷启动加速手段，导入会立即触发激活事件，动态 worker 随即消费。
4. `PureBlindRouteBuilder::build()` 在没有静态路由时允许返回空集合，此时策略将依赖快照或后续动态路由。

## 3. 运行期落盘

- 订阅 `PoolCatalogEvent` 与 `RouteCatalogEvent`，维护一份内存态快照 buffer。
- 新增任务调度器，每 `snapshot_interval_secs`（默认 30 秒）或累计 N 条变更时触发落盘，并在 CLI 退出时调用 `PureBlindCacheManager::flush` 做最终写入：
  1. 从 catalog 收集当前活跃条目。
  2. 过滤掉超过 `snapshot_ttl_secs` 未命中的记录。
  3. 写入临时文件，再原子 `rename` 到正式文件；若持久化关闭会记录 skipped 原因。
- 换班或关闭时可主动调用 `export_snapshot`，或依赖 CLI 自动 flush，确保最新数据持久化。

## 4. 容量与淘汰策略

- `PureBlindStrategyConfig` 新增 `cache` 配置：
  ```toml
  [pure_blind_strategy.cache]
  enable_persistence = true
  cache_dir = "monitoring/pure_blind_cache"
  max_pools = 50
  max_routes = 20
  snapshot_interval_secs = 30
  snapshot_ttl_secs = 3600
  ```
- 在 `PoolCatalog::ingest` 与 `RouteCatalog::ingest` 尾部检查总量，超过阈值时按以下优先级淘汰：
  1. `stats.last_seen_slot`（最久未命中先淘汰）。
  2. 若相同，再比较 `stats.observations`（命中次数较少者先淘汰）。
  3. 需要时加入 `ActivationState.last_observed_at` 辅助。
- 可选：按 base mint 分桶维护 Top-N，防止热门 base 占满配额。

## 5. 数据结构调整

- 为 `PoolCatalog` / `RouteCatalog` 增加：
  - `load_snapshot(entries: Vec<SnapshotPool/Route>)`
  - `export_snapshot(now: Instant) -> Snapshot`
  - `enforce_capacity()` 控制总量。
- `SnapshotPool`/`SnapshotRoute` 含最小必要字段：
  - 池子：池地址、DEX、输入/输出 mint、最新 slot、命中次数、关联 ALT。
  - 路线：依次列出步骤的池地址+mint、合并后的 ALT 列表、累计命中、最近 slot。
- 允许记录来源（snapshot/observation），以便调试与后续统计，导入时会立即触发激活事件供动态路由使用。

## 6. 监控与验证

- 新增指标：
  - `galileo_pure_blind_cache_snapshot_written_total`
  - `galileo_pure_blind_cache_snapshot_entries`
  - `galileo_pure_blind_cache_snapshot_skipped_total{reason}`
  - `galileo_pure_blind_cache_pruned_total{target="pool|route"}`
- 文档更新：
  - 说明快照目录、配置项含义、如何清空缓存；可直接删除 `monitoring/pure_blind_cache/` 或运行 `scripts/pure_blind_cache.sh` 重新拉取基线文件。
  - 给出冷启动流程与调试命令（例如 `cat monitoring/pure_blind_cache/routes.json` 查看当前快照）。

> 按此设计，纯盲策略就算在完全空白节点上重启，也能依靠上次观测到的高频池子/路线快速进入盲发状态，并且不会因为池子持续增长导致资源失控。随后可在实现阶段逐步落地配置、快照 API、容量控制及监控。  
