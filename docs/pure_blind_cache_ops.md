# 纯盲发缓存运维指引

> 帮助运维与开发在不同环境中管理 `PureBlindCacheManager` 输出的快照目录。

## 快速概览

- 默认目录：`monitoring/pure_blind_cache/`，可通过 `pure_blind_strategy.cache.cache_dir` 自定义。
- 启用条件：`pure_blind_strategy.cache.enable_persistence = true`。
- 写入策略：后台任务每 `snapshot_interval_secs` 刷新一次，CLI 退出前会再执行 `flush`。

## 日常操作

### 查看当前快照

```bash
ls monitoring/pure_blind_cache/
cat monitoring/pure_blind_cache/pools.json | jq '.entries | length'
cat monitoring/pure_blind_cache/routes.json | jq '.entries | length'
```

### 清理或禁用缓存

1. 临时关闭：在策略配置中将 `enable_persistence` 设为 `false`。
2. 强制清空：删除目录或备份后清空。

```bash
rm -rf monitoring/pure_blind_cache/
```

> **注意**：删除目录后下次启动会尝试重新创建并记录 `skipped` 原因。

### 手动写入

运行时可调用 CLI 的退出逻辑或在 REPL 中触发：

```rust
cache_manager.flush(&pool_catalog, &route_catalog).await?;
```

## 监控指标

| 指标 | 标签 | 含义 |
| ---- | ---- | ---- |
| `galileo_pure_blind_cache_snapshot_written_total` | `target` | 成功写入快照次数（`pool`/`route`） |
| `galileo_pure_blind_cache_snapshot_entries` | `target` | 每次快照记录的条目数 |
| `galileo_pure_blind_cache_snapshot_skipped_total` | `target`,`reason` | 跳过写入原因，例如 `missing`、`disabled` |
| `galileo_pure_blind_cache_pruned_total` | `target` | 超出容量时被淘汰的条目数量 |

启用 Prometheus 输出时记得在看板中添加上述指标，以便观察冷启动命中率与淘汰频率。

## 故障排查

- **快照导入失败**：检查日志 `pure_blind::cache`，若为 `expired` 可调整 `snapshot_ttl_secs`。
- **动态路由未生效**：确认 `galileo_pure_blind_cache_snapshot_skipped_total{reason="expired"}` 是否增加，以及动态 worker 是否在消费路由事件。
- **磁盘占用异常**：增加 `max_pools`/`max_routes` 前先评估内存开销，必要时缩短 `snapshot_ttl_secs` 或调低容量阈值。
