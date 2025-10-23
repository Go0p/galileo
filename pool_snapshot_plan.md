# Galileo Pool Snapshot Roadmap

## 背景
- 现有策略依赖 `intermedium` 手工维护的池子白名单，缺乏统一的快照输出。
- 启动时仍需直接访问 RPC 扫描池子，会带来不稳定性和可观测缺口。
- 目标：先实现一个“轻量池子采集 + intermedium 筛选 + 快照输出”的闭环，为后续余额/市值增强打地基。

## 目标状态
- 独立的池子索引流水线，根据 `enable_dexs` 自动拉取对应协议的池子。
- 仅保留被 `intermedium` 允许的池子，生成轻量 `markets.json` 快照。
- 引擎启动优先加载快照，缺失时再触发实时采集。
- 留出扩展点：未来可补充池子余额、市值、评分等字段。

## 里程碑
1. **DX 约束梳理**
   - 明确各 `dexes/*` 模块可复用的解析方法。
   - 列出 `getProgramAccounts` 所需的过滤条件（owner、memcmp）。
2. **接口设计**
   - 在 `dexes` 层定义 `PoolIndexer` trait（示例：`fetch_pools(&self, rpc) -> Result<Vec<PoolMeta>>`）。
   - `PoolMeta` 仅包含协议标识、池子 pubkey、双币 mint 等基础字段。
3. **Intermedium 集成**
   - 新增 `apply_pool_filters` 流程，将采集结果与现有规则对齐。
   - 确保白名单/黑名单、资产组合等约束可以复用。
4. **快照落盘**
   - 设计 `markets.json`（或同等结构）的 schema 与存储路径。
   - 提供 CLI/内部 API 生成、刷新快照；引擎启动读取快照。
5. **监控与增量计划**
   - 记录快照生成时间与槽位，便于后续增量更新。
   - 规划下一阶段：余额/市值解析、池子健康度评分。

## 行动项（当前循环）
- [x] 彙总 `enable_dexs` 涉及协议的 `getProgramAccounts` 参数。
- [x] 起草 `PoolIndexer` 与 `PoolMeta` 接口定义。
- [x] Draft `intermedium` 过滤入口点（不包含余额逻辑）。
- [ ] 选定快照文件名与加载顺序，记录于文档/配置。
- [ ] 评估测试方式（单元 + 集成），确保快照生成可验证。

## 快照 Schema 与加载顺序（草案）
- **文件路径**：`data/pools/markets.json`（可通过 `GALILEO_POOL_SNAPSHOT` 环境变量覆盖）。
- **结构示例**：
  ```json
  {
    "generated_slot": 123456789,
    "generated_at": "2024-04-01T12:00:00Z",
    "pools": [
      {
        "dex": "HumidiFi",
        "program_id": "9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp",
        "market": "DB3sUCP2H4icbeKmK6yb6nUxU5ogbcRHtGuq7W2RoRwW",
        "base_mint": "So11111111111111111111111111111111111111112",
        "quote_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "raw_data_len": 640
      }
    ]
  }
  ```
- **加载顺序**：
  1. 引擎启动时优先尝试加载快照；若文件缺失或解析失败，记录告警并回退到实时索引。
  2. 每次成功生成新快照后覆盖旧文件，并输出摘要（池子数量、涉及 DEX 等）。
  3. 预留增量流程：未来可利用 `generated_slot` 做增量刷新或 TTL 校验。

## 测试规划（草案）
- **单元测试**：针对各 `PoolIndexer` mock `RpcClient` 响应，验证 GPA 参数与解码逻辑；对 `intermedium::filter_pools` 提供样例池子确保过滤正确。
- **集成测试**：在本地 `solana-test-validator` 或 fork RPC 环境，执行一次完整的“采集 → 过滤 → 输出快照”流程，校验文件写入。
- **回归检查**：在 CI 中新增 schema 校验，若 `markets.json` 结构变化需同步更新文档。

## 风险与注意事项
- 不同协议的账户布局差异大，需逐个验证以防解码失败。
- RPC 查询跨度大时要注意批量与节流；必要时加入缓存。
- 快照与实时状态之间可能存在滞后，需在后续版本引入 TTL/更新策略。

## 下一步
完成上述行动项后，进入实现阶段：按照文档中接口逐步落地，并同步更新 `docs/` 与 `intermedium` 配置说明，确保团队对流程一致。*** End Patch
