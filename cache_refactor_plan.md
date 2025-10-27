# Cache Refactor Plan

## 目标
- 抽象统一的缓存模块，给 ALT、余额等账户类数据提供高性能、可扩展的复用层。
- 默认提供内存实现，后续可无缝替换为 Redis 等外部后端。
- 支持 TTL、懒清理、观测指标，保持零拷贝或最小拷贝。
- 提供通用 `load_or_fetch` 接口，减少业务侧重复代码。
- 允许围绕缓存模块对现有落地路径（例如 ALT 处理、Marginfi 余额等）进行重构，追求更优雅、更高性能的实现；不保留向后兼容。

## 关键设计
1. **模块结构**
   - `cache::backend`：定义 `CacheBackend<K, V>` trait 以及默认 `InMemoryBackend`。
   - `cache::entry`：封装 `Arc<V>` + 过期时间，提供零拷贝访问。
   - `cache::layer`：公开 `Cache<K, V, B>`，实现 `load_or_fetch`、`invalidate`、metrics 钩子。
   - `cache::key`：标准化 key 序列化，例如 `Alt(pubkey)`、`AtaBalance(pubkey)`。

2. **后端抽象**
   - `get` 返回 `Guard`（`Arc<V>` 或 `Arc<Lease>`），尽量避免拷贝。
   - `put` 接受 TTL，可选 `None` 表示常驻。
   - 后续实现 `RedisBackend` 时只需满足相同 trait。

3. **TTL 与清理**
   - 默认懒清理：`get` 时检查过期并移除。
   - 可选开启后台 GC 任务；实现细节根据性能调优。

4. **集成路线**
   - 替换 `multi_leg::AltCache` 与 `engine::LookupTableCache` → 使用新的 `Cache<Pubkey, AddressLookupTableAccount>`。
   - Marginfi 余额缓存迁移到同一套缓存接口。
   - 文档/metrics 同步更新，确保缓存命中率、刷新失败等可观测。

5. **性能要求**
   - 采用 `Arc` 与 `DashMap` 等结构减小锁粒度、clone 次数。
   - 若需更进一步，可选 `arc-swap`、`parking_lot` 等方案提高吞吐。

## 允许的重构
- 缓存模块上线过程中，只要能获得更优雅、更高性能的实现，允许重写或删除原有实现（包括 ALT 处理、交易构建、余额查询等），无需顾虑旧接口兼容。

