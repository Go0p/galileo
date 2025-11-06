# 多 IP 轮询调度模型（草案）

> 目标：还原“单 IP 轮询”这一最直观的心智模型，再让多 IP 仅仅作为其并行扩展，不重新引入复杂的并发概念。便于理解、配置和后续扩展。

---

## 1. 单 IP 语义：最小可理解单位

- **执行单位**：`QuoteBatchPlan`（一次 trade size 请求），多腿场景下内部仍是买/卖腿并发，但对调度而言它就是“发出去一次 quote 工作”。
- **调度规则**：
  1. 顺序取出待执行的 trade size。
  2. 立即执行这一组 quote。
  3. 完成后等待固定的 `inter_batch_delay_ms`。
  4. 重复步骤 1–3，直到这一轮 trade size 用尽。
- **可配置项**（单 IP 模式仅需一项）：
  - `inter_batch_delay_ms`：两次 trade size 之间的固定间隔。
- **执行结束后**：可以选择再等待一个 `cycle_cooldown_ms` 再开启下一轮（可选项，如果不需要间隔，设为 0）。

> 整个策略在单 IP 下只需要一个“组间隔”参数即可完全描述，无需考虑任何并发概念，日志也很好解释。

---

## 2. 多 IP = 多个“单 IP 轮询器”

- 新增一个并发槽位数 `max_concurrent_slots`：
  - 表示“同时最多有多少个槽位在跑单 IP 轮询流程”。
  - 每个槽位都有自己独立的循环：取 trade size → quote → 等待 `inter_batch_delay_ms`。
  - 若设为 1，则退化为严格串行（单 IP 模式）。
- `per_ip_inflight_limit` 固定为 1（无需暴露）。一个 IP 同一时间只服务一个槽位，避免混乱。

---

## 3. IP 分配与轮换

- 仍然使用 `IpAllocator` 管理 IP 池，但新增一个参数 `max_active_ips`=
  `max_concurrent_slots`，表示同时活跃的槽位数量。
- **分配策略**：
  - 对于每个槽位，需要执行新一次 quote 时，向 IpAllocator 请求一个 IP。
  - 分配时随机/轮训遍历整个 IP 池（即使池内有 250 个 IP，也会轮着用）。这样可以均衡利用 IP，防止部分 IP 长期闲置或被单点打爆。
  - 若池子内 IP 不足（例如 250 个 IP 只允许 50 个活跃槽位），其他 IP 仍保留在库里备用，未来随时可以调整 `max_concurrent_slots` 后立即投入使用。

> 多 IP 的意义：我们不需要操心调度层的逻辑，只需配置“同时有几个槽位在跑”；每个槽位看起来都像一个独立的单 IP 工人。

---

## 4. 可配置项总结

| 配置项 | 含义 | 默认建议 |
| --- | --- | --- |
| `inter_batch_delay_ms` | 单个槽位在两次 trade size 之间的等待时间 | 0 或者具体节奏（如 500ms） |
| `cycle_cooldown_ms`（可选） | 一轮所有 trade size 完成后的休息时间 | 0 或具体时间 |
| `max_concurrent_slots` | 同时运行的槽位数量（决定并发） | 单 IP 模式设 1，多 IP 模式可调大 |
| `max_active_ips` | 同时分配给槽位的 IP 数（通常等于槽位数） | 与 `max_concurrent_slots` 相同 |

> 只有当 `max_concurrent_slots > 1` 时，系统才真正利用多 IP 并发；否则仍是单 IP 顺序轮询。 \
> 无论槽位多少，每个槽位内部的行为完全一致，只有 `inter_batch_delay_ms` 决定节奏。

---

## 5. 设计目标达成点

- **易懂**：任何人只需记住“每个槽位都像一个单 IP 轮询器”，配置非常少。
- **多 IP 扩展**：并发数 = 槽位数；想利用更多 IP，只需调大 `max_concurrent_slots`，不需要再考虑一堆 spacing/parallelism 的乘法。
- **IP 利用率**：随机轮换池中 IP，即便只启用了 50 个槽位，也能让 250 个 IP 逐渐轮换，避免僵化或封禁风险。
- **实现切入点**：调度层重构为“槽位驱动”；IpAllocator 只负责随机分配 IP 给槽位；槽位内部用极简状态机即可。

---

> 本文在记录从旧的 `group_parallelism` / `intra_group_spacing_ms` 等参数迁移到槽位模型的过程；当前代码与配置已经使用新字段，后续若有调整，可在此文件补充。
