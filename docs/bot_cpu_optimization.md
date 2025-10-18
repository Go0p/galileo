# Bot 与验证节点共机优化建议

> 摘要自社区实战经验，帮助套利 bot 与 Solana 节点部署在同一台服务器时实现 CPU 隔离与稳定低延迟。

## 优化目标
- 保证 `poh` 线程拥有独占物理核，避免 slot 落后。
- 降低 Tokio 运行时因任务窃取产生的抖动，稳定 bot 的响应时间。
- 让系统中断与后台任务绕开验证节点关键核，减少 CPU 抢占。

## 关键措施
- **PoH 锁核**：使用 `--experimental-poh-pinned-cpu-core <core_id>` 将 `solPohTickProd` 绑定到 `core_id` 及其超线程 sibling，或手工通过 `taskset` 设置亲和性。
- **CPU 拓扑确认**：通过 `cat /sys/devices/system/cpu/cpu<id>/topology/thread_siblings_list` 获取物理核与超线程对应关系，配置 `isolcpus` / `irqaffinity` 时必须同时排除两者。
- **GRUB 隔离**：在 `GRUB_CMDLINE_LINUX_DEFAULT` 中设置形如 `irqaffinity=0-1,3-49,51-95` 的值，将 PoH 核及其超线程从中断亲和性里剔除；修改后 `update-grub` 并重启。
- **Bot 核心绑定**：Rust 侧可使用 `core_affinity` 等 crate 为 Tokio worker 或关键异步任务绑定剩余物理核，减少与验证节点的竞争。
- **实时验证**：通过 `htop` 观察 PoH 核使用率，应稳定占满目标核心且 sibling 核空闲；追块阶段检查 slot behind 指标是否改善。

## Galileo 配置示例
```yaml
bot:
  cpu_affinity:
    enable: true
    worker_cores: [3,4,5,6,7,8,9,10]
    max_blocking_threads: 8
    strict: false
```
- `worker_cores`：按顺序列出可用核心 ID（包含超线程编号），程序会按顺序将 Tokio worker 线程绑定到这些核心。
- `max_blocking_threads`：可选限制，避免 Tokio 阻塞线程池额外抢占核心；置空则使用默认值。
- `strict`：启用后若任何核心不存在即直接报错退出，便于在部署脚本中快速发现拓扑变化。

## 实施流程
1. **识别核心**：在节点机器上运行 `cat /sys/devices/system/cpu/cpu2/topology/thread_siblings_list` 等命令，记录每个物理核的两条线程号。
2. **配置引导参数**：依据拓扑更新 GRUB 的 `isolcpus`, `nohz_full`, `irqaffinity` 等选项，重启后确认参数生效。
3. **启动节点**：运行 `solana-validator --experimental-poh-pinned-cpu-core <core_id>`，必要时手动 `taskset -cp <core_id> <validator_pid>` 校准。
4. **调整 bot**：为 bot 进程或 Tokio 线程池设置 CPU 亲和性，确保不会占用 PoH 核。
5. **监控回归**：在追块和高负载行情中持续观察 `slot behind`、PoH 线程占用、Quote 请求延迟，发现波动时重新评估锁核策略。

## 进一步建议
- 将隔离步骤与核心编号记录在团队文档，便于硬件升级或核编号变化时快速复现。
- 若引入 Jito proxy 或其他高优先级服务，复用相同的隔离思路，为关键线程预留独立核心。
- 对于 bot 的高峰流量，结合流控策略防止暴增的 Quote 请求压垮节点 CPU。
