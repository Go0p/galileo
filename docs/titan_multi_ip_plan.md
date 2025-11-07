# Titan 多 IP 订阅重构计划

## 背景
当前 Titan 腿的报价流按 `(pair, amount)` 去重，所有 IP 共享同一条 WebSocket stream。这导致：
- 单 IP 场景：Titan 会尝试覆盖所有 trade size，无法限定为“两条固定订阅”。
- 多 IP 场景：流仍然共用，无法把不同 trade size 平均分配到多个 IP，也无法保证每个 IP 只维护两个订阅。

## 目标
1. **单 IP**：Titan 只建立 2 条 WebSocket 流（每条对应一个 trade size），并用这两条流驱动卖腿套利。
2. **多 IP**：有 N 个 IP 就建立 `2 * N` 条流；每个 IP 只维持自己的两条订阅，互不共享。
3. 保持 Titan 卖腿报价与落地流程不变，仅重构订阅 / 分配逻辑。

## 设计要点
- **流键扩展**：将当前 `StreamKey { pair, amount }` 扩展为 `StreamKey { pair, amount, ip }`，确保不同 IP 订阅不会互相复用。
- **订阅分配器**：新增 `TitanSubscriptionPlanner`，负责为每个 IP 选择两条 trade size。策略：
  - 从策略配置导出的 trade size 列表中，按 base mint / amount 排序；
  - 轮询 IP 列表，依次分配两条 size；
  - 若 size 数量不足 `IP * 2`，则循环分配；若 size 过多，则只取前 `IP * 2` 条。
- **QuoteDispatcher 扩展**：
  - 在生成 `QuoteBatchPlan` 时附带“指定 IP”信息（例如新增 `batch.preferred_ip`）。
  - 获取 Titan quote 时优先使用该 IP；若 IP 忙或不可用，可 fallback 到下一空闲 IP，并更新 planner 状态。
- **TitanWsQuoteSource 调整**：
  - `quote()` 内部不再调用 `IpAllocator`；外层预先传入 IP。
  - stream map 由 `HashMap<StreamKey, TitanStreamState>`（key 含 IP）。
- **资源回收**：当 IP stream 被关闭或 IP 失效，需要通知 planner 重新分配。

## 实施步骤
1. **订阅计划器**
   - 新建 `engine::titan::subscription` 模块，输入：trade pairs、trade size 列表、IP 列表；输出：`Vec<(ip, TradeSize)>`。
   - 在策略初始化时（构造 `StrategyEngine`），根据当前 IP allocator 的 slot 列表生成计划。

2. **Batch 标记优先 IP**
   - 扩展 `QuoteBatchPlan`，新增 `preferred_ip: Option<IpAddr>` 字段。
   - `StrategyContext::push_quote_tasks` 在调用 planner 后，对 Titan backend 的批次写入专属 IP。

3. **QuoteDispatcher 绑定 IP**
   - 在调度 Titan quote 时（可通过 backend 信息判断），如果 `preferred_ip` 存在，则调用 `IpAllocator::acquire_specific(ip)`（需新增接口）拿对应句柄；若失败，尝试回退。

4. **Titan Provider 改造**
   - `TitanWsQuoteSource::quote` 改为 `quote(&self, intent, ip)`，由外层传 IP。
   - stream key / map 改为 `(pair, amount, ip)`。
   - 仍复用现有超时 / watch channel 机制。

5. **多 IP 动态更新（可选）**
   - 若运行期间 IP 变化或 trade size 热更新，需要重新生成计划。第一版可在重启时生效。

6. **测试与验证**
   - 单 IP：确保只建立 2 条连接（日志输出）。
   - 多 IP：模拟 3 个 IP，观察共 6 条流并分别绑定。
   - 回归：未开启 Titan 的策略不受影响。

## 风险与缓解
- **IP allocator 缺少“指定 IP”接口**：需要新增 `acquire_specific(ip, kind, mode)`，若 IP 忙则返回 Err。初期可允许 fallback。
- **trade size 少于 IP*2**：计划器需循环分配，保证每个 IP 至少有 1 条流。
- **现有 Titan 流缓存失效**：变更 key 后旧逻辑作废，需完整测试推流连接 / 重连流程。

## 开发计划
1. 基础设施：Planner + `QuoteBatchPlan` 扩展 + allocator 接口。（约 1.5 天）
2. Titan provider / stream map 改造（约 1 天）。
3. 调度整合、回归测试（约 1 天）。

完成后，Titan 每个 IP 仅维持两条固定订阅，多 IP 也能线性扩展覆盖更多 trade size。EOF
