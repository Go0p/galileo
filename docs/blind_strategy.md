# 盲发策略改造说明（草案）

> 目标：彻底移除对 Jupiter /swap-instructions API 的依赖，在本地实现「纯盲发」——直接构造 `route_v2` 指令与账户序列，按配置生成双向套利交易。本文用于梳理需要交付的工作项，方便后续拆分任务。

## 1. 背景与约束
- 当前 `BlindStrategy` 仍依赖 quote → profit → swap 的传统链路，流程中调用 Jupiter API 获取报价与指令。这与“盲发”理念相悖。
- 新需求强调**不再调用 Jupiter 二进制或 API**，而是使用本地逻辑生成交易。唯一需要复用的是 Jupiter `route_v2` 指令格式（程序 id、前置账户固定），我们负责拼装 `data` 与 `remaining_accounts`。
- 配置来源仍为 `blind_strategy` 节点。`trade_range_count` 为 `N` 时，需要在一个循环周期内发送 `2N` 笔交易（正向 + 反向各 `N` 笔），并允许随机化顺序。
- 新增配置开关 `blind_strategy.pure_mode`：`false` 保持原有 quote → swap 流程，`true` 则启用纯盲发（不启动 Jupiter、本地组装指令）。
- 纯盲发需要通过 `blind_strategy.pure_routes` 明确买入/卖出市场对（当前支持 SolFiV2、TesseraV，可混搭），例如 `buy_market` 指向买入池，`sell_market` 指向卖出池。路由顺序默认先在 `sell_market` 执行 `BaseToQuote`，再在 `buy_market` 执行 `QuoteToBase`，形成 2-hop route；随后会自动补上一条反向（Quote → Base → Quote）路径。
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
- **trade size 生成**：沿用 `generate_amounts_for_base()` 逻辑（`src/cli/strategy.rs:296`），得到升序 `Vec<u64>`。  
- **双向盲发**：对每个 `amount`，生成两条路线：  
  1. `DEX_A (买入)` → `DEX_B (卖出)`  
  2. `DEX_B (买入)` → `DEX_A (卖出)`  
  这样 `trade_range_count = 3` ⇒ 共 6 笔交易。
- **顺序随机化**：可在调度层（`StrategyEngine::handle_action` 或新建执行器）对双向指令进行 `shuffle`，避免可预测节奏。
- **节流控制**：沿用 `process_delay`、`sending_cooldown`。需确保盲发过程中仍尊重这些参数，否则会与守则“评估 Quote → Swap → 落地耗时”冲突。

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
