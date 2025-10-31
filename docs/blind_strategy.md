# 盲发策略改造说明（草案）

> 目标：彻底移除对 Jupiter /swap-instructions API 的依赖，在本地实现「纯盲发」——直接构造 `route_v2` 指令与账户序列，按配置生成双向套利交易。本文用于梳理需要交付的工作项，方便后续拆分任务。

## 1. 背景与约束
- 当前 `BlindStrategy` 仍依赖 quote → profit → swap 的传统链路，流程中调用 Jupiter API 获取报价与指令。这与“盲发”理念相悖。
- 新需求强调**不再调用 Jupiter 二进制或 API**，而是使用本地逻辑生成交易。唯一需要复用的是 Jupiter `route_v2` 指令格式（程序 id、前置账户固定），我们负责拼装 `data` 与 `remaining_accounts`。
- 配置来源仍为 `blind_strategy` 节点。每个 base mint 通过若干 `lane`（`min` / `max` / `count` / `strategy` / `weight`）描述想要探索的区间，调度器会按这些定义生成正向与反向的交易规模，并可按权重自动扩容以压满 IP。
- 纯盲发配置迁移至独立的 `pure_blind_strategy` 节点：常规盲发仍由 `blind_strategy` 控制，纯盲发的启用、市场缓存与调度策略完全独立。
- 纯盲发闭环可通过 `pure_blind_strategy.overrides` 声明。每条路线以 `legs` 列表按顺序写出市场（当前支持 SolFiV2、TesseraV、HumidiFi、ZeroFi、ObricV2，可混搭），系统会自动解析资产流向并生成正/反向闭环。若路由需要引用 Address Lookup Table，可在同级声明 `lookup_tables`：
  ```yaml
  pure_blind_strategy:
    overrides:
      - legs:
          - market: "<HumidiFi 市场>"
          - market: "<Whirlpool 市场>"
          - market: "<ZeroFi 市场>"
        lookup_tables:
          - "<Address Lookup Table Pubkey>"
  ```
  配置中的 lookup table 会在预检查阶段批量解析、校验激活状态，并在后续交易执行时直接随 `swap_instruction` 一起下发。
- 守则强调性能与可观测性：热路径需加 `hotpath::measure`；所有新流程要补 metrics / tracing。
- **无需考虑向后兼容**：当前仓库尚未上线，重构时可直接移除旧有 quote → swap 逻辑，避免历史包袱。
- 关联资料：  
  - Jupiter IDL：`idls/jup6.json`（用于获取 `route_v2`、`RoutePlanStepV2`、各 DEX variant 的精确字段）  
  - Program → Label 映射：`analyze_dex_accounts/program-id-to-label.json`（用于确认 DEX 标签与程序 id 的对应关系，避免手写错误）
- 首批支持的 DEX 标签：`SolFiV2`、`HumidiFi`、`TesseraV`、`ZeroFi`。后续新增需同步更新文档与映射表。

## 2. `route_v2` 指令手搓要点
### 2.1 指令数据字段
| 字段 | 取值方案 |
| --- | --- |
| `in_amount` | 取自 blind 配置生成的交易规模（例如 10_000_000_000 lamports）。 |
| `quoted_out_amount` | 对盲发而言需**严格大于等于** `in_amount`。推荐 `in_amount * (1 + ε)`，其中 ε 可取 1‱（避免等于导致 0 滑点 + rounding）。 |
| `slippage_bps` | 固定为 `0`，确保 Jupiter 在链上根据池状态自行撮合。 |
| `platform_fee_bps` / `positive_slippage_bps` | 初期均设为 `0`。如需提成可后续引入配置。 |
| `route_plan` | `Vec<RoutePlanStepV2>`，由我们注入（详见下节）。 |

### 2.2 RoutePlanStepV2 构造
单步字段：
- `swap`: 使用 `EncodedSwap::from_name("DexLabel", payload)` 转换。其中：  
  - `TesseraV` 需要 `side`（`Bid` / `Ask`）  
  - `HumidiFi` 需要 `swap_id` & `is_base_to_quote`  
  - `SolFiV2` 需要 `is_quote_to_base`
- `bps`: 盲发场景下用满仓 `10000`，若 future 需分仓，可根据组合策略拆分。
- `input_index` / `output_index`:  
  - 对二跳套利，惯例设 `input_index = 0 → 1 → 0`，保持与 Jupiter 兼容。
  - 反向交易则交换输入/输出索引。

### 2.3 剩余账户填充
`route_v2` 固定前 10 个账户由 `RouteV2Accounts::to_account_metas()` 输出（authority、用户 ATA、mint、token program、event authority、program）。
真正的 DEX 特定账户通过 `remaining_accounts` 附加，需遵循链上程序的排序约定：

| DEX | 账户顺序示例 |
| --- | --- |
| **SolFiV2** | `payer`, `market`, `oracle`, `config`, `base_vault`, `quote_vault`, `user_base_ata`, `user_quote_ata`, `base_mint`, `quote_mint`, `base_token_program`, `quote_token_program`, `Sysvar1nstructions1111111111111111111111111` |
| **TesseraV** | `global_state`、`pool_state`、`user_authority`、`base_vault`、`quote_vault`、`user_base_token`、`user_quote_token`、`base_mint`、`quote_mint`、`base_token_program`、`quote_token_program`、`Sysvar1nstructions1111111111111111111111111` |
| **HumidiFi** | 需调用现有解析工具或自建 decoder 获取 `swap_id` 对应账户；顺序同 Jupiter 规范 |
| **ZeroFi** | `market (pair)`、`vault_info_in`、`vault_in`、`vault_info_out`、`vault_out`、`user_source_token`、`user_destination_token`、`swap_authority`（通常为 payer，自带签名）、`token_program`（Tokenkeg 或 Token-2022）、`Sysvar1nstructions1111111111111111111111111` |

若某 DEX 需要 `remaining_accounts_info`（例如 `HumidiFi`），必须同步填入 `EncodedSwap` payload，保持与账户顺序一致。

## 3. 账户准备流程
1. **ATA 推导**：重用 `derive_associated_token_address`（`engine/precheck.rs:32`）。针对每个 mint 推导 Tokenkeg 与 Token2022 版本，选择实际 owner 对应的 program。  
2. **池账户解码**：  
   - SolFiV2：已实现 `fetch_solfi_v2_swap_info()` → 返回市场必需字段和 `sysvar::instructions::ID`。  
   - TesseraV / HumidiFi：需在 `src/dexes` 下补充 decoder + fetch，避免 runtime 解析。  
3. **缓存策略**：遵循“无缓存原则”。如需提高性能，可在单个 tick 内复用一次 RPC 结果，但不得跨 tick 保存陈旧数据。

## 4. 交易调度与随机化
- **trade size 生成**：每个 base mint 由一组 `lane` 定义（`min` / `max` / `count` / `strategy` / `weight`），系统会根据策略在区间内插值或采样，随后施加 930–999 bp 的轻量扰动。若开启 `auto_scale_to_ip`，会按 lane 的权重自动补充额外档位以压满可用 IP。  
- **批量盲发**：当某个 base mint 达到 `process_delay`，调度器会一次性发送该 mint 的全部 trade size，确保 IP 资源被充分占用。  
- **顺序随机化**：若需要进一步打散顺序，可在调度层（`StrategyEngine::handle_action` 或专用执行器）对返回的任务 `shuffle`。  
- **节流控制**：`process_delay` 用于约束同一 base mint 进入下一轮批量调度的最小间隔；`batch_interval_ms` 仅在需要时作为跨批次退避。`sending_cooldown` 暂未在调度层生效，如需开启需补充实现。

## 5. 监控与性能
- **Tracing**：在关键步骤添加 `hotpath::measure` / `hotpath::measure_block!` 标签，例如：  
  - 指令构造器（SolFiV2 fetch、RoutePlan 生成）  
  - 批量发送交易的调度函数  
- **Metrics**：新增指标建议：  
  - `blind.tx_prepared_total{direction="forward|reverse",dex_pair="A-B"}`  
  - `blind.tx_sent_total{status="ok|err"}`  
  - `blind.build_latency_ms`（构造耗时）  
  指标定义放在 `monitoring::events`，并在文档中说明用途。

## 6. 实现步骤拆分
1. **配置 → 交易对生成重构**：允许在 blind 配置中声明 DEX 组合或默认矩阵（如 Tessera ↔ HumidiFi ↔ SolFiV2）。  
2. **账户解析模块完善**：为 TesseraV / HumidiFi 引入 `fetch_*_swap_info()`，与 SolFiV2 对齐。  
3. **Route 构造器**：新增 `BlindRouteBuilder`，输入（amount, forward|reverse, dex pair）→ 输出 `SwapInstructionsResponse`。  
4. **执行引擎改造**：`SwapInstructionFetcher::fetch` 增加盲发分支；当检测到“纯手搓”模式时跳过 Jupiter API，并通过 `src/strategy/pure_blind_strategy.rs` 调度 `Action::DispatchBlind`。  
5. **测试与干运行**：  
   - 单元测试：对每个 DEX 组合校验 `Instruction` 序列与 `data`。  
  - `cargo run --features=hotpath` 文档更新，说明如何启用性能采样。  
6. **监控文档**：同步更新 `docs/galileo_config_reference.md`、新增看板说明。

## 7. 风险与待确认事项
- **Dex 账户顺序验证**：需通过现有交易样本或 reverse 工具验证账户排序，避免链上执行失败。  
- **`quoted_out_amount` 策略**：盲发要求“>`in_amount`”，但过大可能导致 Jupiter 校验不过（声称超出预期）。建议在实现前验证可接受范围。  
- **Flashloan 兼容性**：盲发交易是否仍需 Marginfi 贷款？若需要，需确认指令顺序（flashloan setup → route_v2 → repay）。  
- **随机化与风控**：需要确保随机顺序不会 violate `sending_cooldown`。可在调度层使用时间窗控制。

## 8. 后续行动
1. 设计 `BlindRouteBuilder` API + 数据结构，提交简要 RFC。  
2. 为 TesseraV / HumidiFi 编写解码器。  
3. 实作盲发执行管线 + 指标。  
4. 补文档与示例配置，包括如何在乾运行中验证 `route_v2`。

---

> 文档维护者：`TODO(填写)`  
> 更新记录：初稿（2025-10-21）

## 9. 纯盲自动路由（2025-02 更新）

- `PureBlindRouteBuilder` 已接入 `MarketCacheHandle`，启动时会解析本地 `markets.json` 快照，通过 `exclude_other_dex_program_ids`、`min_liquidity_usd` 等参数快速筛选候选池子。
- `routing_group`：表征 Jupiter 官方的路由信赖分层（0/1 一级市场，2 次一级，3 长尾或实验池）。当前仅跳过 >3 的值，确保稳健池子优先，同时保留可选长尾。
- `pure_blind_strategy.enable_landers` 允许为纯盲发指定独立的落地器序列，不再复用 `blind_strategy` 设置。
- `pure_blind_strategy.market_cache.exclude_dex_program_ids` 可显式排除指定 Program ID 所属的池子，避免与竞品或不稳定池子打架。
- `pure_blind_strategy.market_cache.proxy` 允许为市场缓存下载设置专用 HTTP 代理，不影响其他 HTTP 请求。
- `pure_blind_strategy.cu_multiplier` 用于在各腿默认 CU 预算之和基础上统一放大/缩放，便于快速调节整体参数。
- 当 `assets.base_mints` 中的入口资产声明 `route_type="2hop"` / `"3hop"` 时，路由构建器会结合 `assets.intermediates` 与缓存元数据自动挑选两腿或三腿闭环，并生成 `route_source=auto` 的路线；手工 `overrides` 仍然保留且优先生效。
- 新增 Prometheus 指标：
  - `galileo_pure_blind_routes_total{route,source}` / `galileo_pure_blind_route_legs{route,source}` —— 记录闭环构建情况。
  - `galileo_pure_blind_orders_total{route,source,direction}` —— 记录每次 tick 下发的盲发指令数量。
- 建议开启 `galileo.yaml` → `bot.prometheus.enable=true` 并将监听地址指向监控节点，结合 `route_source` 标签可以分别监控手工与自动路线的表现。
- 如需刷新缓存，可使用新增 Task：`task pure_blind:cache:refresh`（详见仓库 `Taskfile.yml`），支持离线下载 `markets.json` 并校验过滤结果。
- 纯盲模式推荐将 `engine.backend` 设置为 `none`，以跳过外部聚合器组件的初始化。
