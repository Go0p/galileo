# Galileo 并发重构计划

> 目标：围绕报价、多腿组合、复制策略三条关键链路，替换掉高冲突的锁与串行计算逻辑，在保持业务语义和监控观测的前提下提升吞吐与响应速度。

## 里程碑概览

1. **调度与评估并行化**
   - 将 `engine::multi_leg::runtime::plan_pair_batch_with_profit` 换成流式收敛（`FuturesUnordered` / `JoinSet`），边出结果边评估。
   - 收集的腿计划交给常驻 `rayon::ThreadPool` 并行计算收益与排序，回写结果结构保持不变。
   - 为新线程池添加热路径打点（metrics/tracing），方便量化收益。

2. **复制策略队列与缓存优化**
   - `strategy::copy::wallet` 的任务队列改用 `flume::Receiver`/`Sender`，替代 `tokio::Mutex<VecDeque<_>>`。
   - ATA / Token 账户缓存改为 `DashMap` 或细粒度 `RwLock`，并清理旧互斥结构。
   - 确保 `scale_compute_unit_limit` 等路径不受修改影响。

3. **锁与状态颗粒度调整**
   - 对不含 `await` 的锁（如 `engine::multi_leg::runtime` throttle、测试桩）换成 `parking_lot::Mutex` 或原子变量。
   - `cache::mod` 保留 `DashMap`，但对热点键增加分区或 TTL 逻辑（如有必要）。

4. **策略执行器队列与落地管线**
   - 按需评估 `lander` 提交路径是否需要 `tokio::sync::Mutex`，避免多地块被串行阻塞。
   - 若引入 `rayon` 后有跨线程共享数据，需要补充 `Send + Sync` 语义审计。

## 实施顺序与交付物

| 阶段 | 关键改动 | 交付物 | 验证 |
| ---- | -------- | ------ | ---- |
| M1 | 引入 `rayon::ThreadPool`，重写 `plan_pair_batch_with_profit` | 新的并发评估模块 + benchmark 脚本 | 单元测试 + 10k 批次压测 |
| M2 | `strategy::copy` 队列重构 | `CopyWallet` 任务处理重构 | integration test（dry-run） |
| M3 | 锁替换与代码清理 | `parking_lot` 替换、删除废弃结构 | `cargo fmt/clippy` + 热路径 tracing |
| M4 | 文档与监控更新 | 更新 `docs/strategy_arch.md`、`monitoring` 指标 | 手动验证 + Grafana 面板说明 |

## 风险与缓解

- **线程安全风险**：Rayon 引入后，需要确认所有共享对象实现 `Send + Sync`。缓解：引入静态断言 / `static_assertions::assert_impl_all!`。
- **行为偏差**：重构可能改变调度顺序。缓解：保留现有监控指标、在 dry-run 模式下对比交易输出。
- **资源占用**：额外线程池可能抢占 CPU。缓解：线程池大小从配置读取，并设置合理默认值（如 `num_cpus::get_physical()` 的一半）。

## 里程碑完成判定

- 关键路径均无 `tokio::sync::Mutex<VecDeque>` 或串行利润计算。
- 新增 `rayon` 线程池默认启用，可以通过 `galileo.yaml` 关闭或配置线程数。
- 监控中 `galileo_multi_leg_plan_latency_ms` / `galileo_copy_queue_depth` 能量化改动效果。
- 文档已指导如何调节线程池、并发队列与指标。 

完成以上四项后即可进入开发阶段执行具体重构任务。 
