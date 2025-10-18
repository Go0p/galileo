# Solana RPC 节点部署与性能隔离指南

> 目标是让验证节点稳定产块、降低延迟，并确保套利 bot 与节点共享同一台物理机时互不干扰。

## CPU 资源隔离

- **锁核策略**：Solana 验证节点的 `Proof of History (PoH)` 线程属于单核运算，需为其专门保留一个物理核（含对应的超线程 sibling），避免任务窃取导致的上下文抖动。
- **命令行参数**：运行 `solana-validator` 时指定 `--experimental-poh-pinned-cpu-core <core_id>`，例如 `--experimental-poh-pinned-cpu-core 2`，同时确保 `<core_id>` 对应的 sibling（如 `cpu2` 的 `thread_siblings_list` 返回的另一个核心）也被隔离。
- **系统调度隔离**：在 `GRUB_CMDLINE_LINUX_DEFAULT` 中配置 `isolcpus`, `nohz_full` 或 `irqaffinity`，让保留给 PoH 的物理核及其超线程完全从调度器和中断绑定里排除，例如：
  ```
  GRUB_CMDLINE_LINUX_DEFAULT="quiet splash isolcpus=2,50 nohz_full=2,50 rcu_nocbs=2,50 irqaffinity=0-1,3-49,51-95"
  ```
  根据 `cat /sys/devices/system/cpu/cpu<core_id>/topology/thread_siblings_list` 的输出来调整上述列表。

## 验证与监控

- 使用 `htop` 或 `taskset -cp <pid>` 验证 PoH 线程是否稳定地运行在目标核心上，确保 `solPohTickProd` 等线程不会漂移。
- 在追块或高负载时观测 `slot behind` 指标，若隔离成功，追块延迟应明显降低。
- 对节点监控补充 CPU 核心利用率、PoH tick 速率以及重放 (`replay`) 线程延迟，以便快速发现隔离策略失效。

## 与套利 Bot 的协同

- 在同一物理机上运行 bot 时，为 Tokio 运行时或关键任务显式设置 CPU 亲和性，避免与节点的 PoH 核竞争。
- 若使用 Galileo，可通过 `galileo.yaml` 中的 `bot.cpu_affinity` 声明绑定核心列表，启动时会自动将 Tokio worker 线程限制在该集合内。
- 将 bot 的突发计算拆分到其余空闲物理核，必要时使用 `core_affinity` 等 Rust 库实现细粒度的任务绑定。
- 监控 bot 侧的请求速率和节点 CPU 使用情况，防止行情波动时 bot 产生的高频 Quote 请求反向抢占节点核心。

## 维护流程

- 修改 GRUB 后需执行 `update-grub` 并重启机器生效。
- 每次内核或验证节点版本升级后，重新检查 `thread_siblings_list` 与 `irqaffinity` 配置，确认物理核编号未发生变化。
- 建议在内部 Wiki/文档记录节点的 CPU 拓扑、保留核编号与检验步骤，便于团队其他成员复现。
