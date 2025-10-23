# Pool Indexing: getProgramAccounts Requirements

> 目标：为每个盲发支持的 DEX 记录 `getProgramAccounts` 采集所需的基础信息，后续实现池子快照时可直接复用。

下表汇总当前代码中使用到的 DEX。若信息缺失，已以 TODO 标记，后续需要根据链上样本或黄石解析库补完。

| DEX            | Program ID | 账户类型/说明 | GPA 过滤建议 | 备注 |
|----------------|------------|----------------|---------------|------|
| **RaydiumClmm** | `CAMMCz6GM1DNKyvHRAAmcX6vLLANBYdmsKtfgfcJQ68X` | `PoolState`（`yellowstone_vixen_raydium_clmm_parser`） | 1. `owner = PROGRAM_ID`；<br>2. **待确认**：`PoolState` 的 8 字节鉴别符，可用于 `memcmp` offset `0`。 | 需要 tick array PDA 推导函数（已在 `decoder.rs` 中提供）。 |
| **MeteoraDlmm** | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | `LbPair`（`yellowstone_vixen_meteora_parser`） | 1. `owner = PROGRAM_ID`；<br>2. **待确认**：`LbPair` 鉴别符。 | Bin array 扩展依赖 `active_id`，后续快照只需记录主 pair。 |
| **Whirlpool** | `whirLbMiicVYDaHEM...` (`ORCA_WHIRLPOOL_PROGRAM_ID`) | `Whirlpool` account | 1. `owner = PROGRAM_ID`；<br>2. **可选**：`Whirlpool` Anchor 鉴别符。 | 需要额外保留 `whirlpool` → oracle PDA 推导。 |
| **SolFiV2** | `SoLSwwapV2i2TAPTufq...` (`SOLFI_V2_PROGRAM_ID`) | 手工解析（参考 `solfi_v2::decoder`） | 1. `owner = PROGRAM_ID`；<br>2. **待确认**：数据长度或自定义标记。 | 非 Anchor，需要根据 decoder 中的 offset 解析。 |
| **HumidiFi** | `9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp` | 配置账户（`ConfigAccount` 自行解析） | 1. `owner = PROGRAM_ID`；<br>2. **建议**：校验 `SWAP_ID_MASK` / mint mask，过滤噪声账户。 | 额外需通过 `getTokenAccountsByOwner(pool, Mint)` 找到 vault。 |
| **TesseraV** | `TeSSerA9sfTKRksu...` (`TESSERA_V_PROGRAM_ID`) | `PoolState`（decoder 自定义） | 1. `owner = PROGRAM_ID`；<br>2. **待确认**：池子账户的固定 header。 | 需要额外抓取 `clmm_state` / `strategy_config` 等 PDA。 |
| **ZeroFi** | `ZeroChg3cd8vZjz...` (`ZEROFI_PROGRAM_ID`) | `PoolState`（decoder 自定义） | 1. `owner = PROGRAM_ID`；<br>2. **待确认**：`ZERO_POOL_TAG` 或 8 字节鉴别符。 | 池子账户同时暴露 base/quote vault，可直接读取。 |
| **ObricV2** | `obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y` | `TradingPair`（自定义结构） | 1. `owner = PROGRAM_ID`；<br>2. 可利用 `pair_state` 标志位过滤。 | 解析时需额外检查 Pyth price ID 归属（`PYTH_V2_PROGRAM_IDS`）。 |

## Notes & TODOs
- **Anchor 鉴别符**：对于 Anchor 合约（Raydium、Meteora、Whirlpool），可以从对应 `yellowstone` crate 查询 `AccountType::DISCRIMINATOR`。待后续补齐具体字节。
- **自定义协议**：HumidiFi / TesseraV / ZeroFi / SolFiV2 / ObricV2 使用了手写解析，需要根据 `decoder` 中的 magic 值或数据长度构造 `memcmp` 过滤。
- **缺失 DEX**：配置中仍包含 `Perps`、`GoonFi` 等占位符，当前代码未实现对应解码；后续确认后在此文档补齐。
- **RPC 成本**：所有查询都需控制并发，并考虑 `data_slice` 以减少传输量。

以上信息将为后续实现 `PoolIndexer` trait 时提供默认参数模板，补完待确认内容后即可落实自动化池子采集。*** End Patch
