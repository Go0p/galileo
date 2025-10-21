# SolFi V2 自定义错误码速查表

- **程序 ID**：`SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF`
- **触发位置**：`rust-solana-solfiv2-sdk/src/state/error.rs`
- **日志格式**：一旦 SolFi V2 指令执行失败，运行时会抛出 `Program returned error: custom program error: 0x??`，其中 `0x??` 为十六进制自定义错误码。将其换算为十进制即可在下表中查阅。
- **示例**：`0x9` → 十进制 `9` → `InvalidBaseUserTokenAccount`（用户提供的 base ATA 与预期不符）。

| 十进制 | 十六进制 | 枚举常量 | 中文含义 | 常见触发原因 / 排查要点 |
| --- | --- | --- | --- | --- |
| 0 | 0x00 | InvalidTokenProgramAccount | Token Program 账户无效 | 检查是否传入 SPL Token Program (`Tokenkeg...`) 或 Token-2022 Program，且与市场使用的程序一致。 |
| 1 | 0x01 | InvalidAssociatedTokenProgramAccount | Associated Token Program 账户无效 | 确认账户指向 `ATokenGPv...`，并与期望的 ATA 程序匹配。 |
| 2 | 0x02 | InvalidSystemProgramAccount | System Program 账户无效 | 检查是否提供了 `11111111111111111111111111111111`。 |
| 3 | 0x03 | InvalidConfigAccount | 配置账户无效 | `config_account` Pubkey 未与市场登记的地址一致，或数据长度错误。 |
| 4 | 0x04 | InvalidMarketAccount | 市场账户无效 | Market PDA、尺寸（`1728` 字节）或所有者不匹配。确认 `market` 传参是否为目标池。 |
| 5 | 0x05 | InvalidBaseMint | Base Mint 无效 | Base mint 与市场头部存储的 mint 不一致。确认路由中 base mint 选择正确。 |
| 6 | 0x06 | InvalidQuoteMint | Quote Mint 无效 | Quote mint 与市场头部记录不一致。 |
| 7 | 0x07 | InvalidBaseVault | Base Vault 无效 | 市场头文件内的 `base_vault` 与指令提供的账户不同，或所有者非 Token Program。 |
| 8 | 0x08 | InvalidQuoteVault | Quote Vault 无效 | 同 `InvalidBaseVault`，针对 quote 资金池。 |
| 9 | 0x09 | InvalidBaseUserTokenAccount | 用户 Base Token 账户无效 | 用户 ATA 未与 base mint 对应，或者 owner 不是交易发起者；检查是否走错方向（Quote→Base 时需要 quote ATA）。 |
| 10 | 0x0a | InvalidQuoteUserTokenAccount | 用户 Quote Token 账户无效 | 同上，确认 quote ATA 与 mint、一致性及余额。 |
| 11 | 0x0b | InvalidTokenAccount | 通用 Token 账户校验失败 | 任一应为 SPL Token Account 的账户存在 owner/mint 对不上或数据损坏。 |
| 12 | 0x0c | UnauthorizedAdminAccount | 管理员账户未授权 | 管理员指令未使用允许的 signer。核对配置中的管理员白名单。 |
| 13 | 0x0d | UnauthorizedConfigUpdaterAccount | 配置更新者未授权 | 更新全局/市场配置时签名者不在 updater 列表。 |
| 14 | 0x0e | UnauthorizedMarketPriceUpdaterAccount | 市场价格更新者未授权 | `market price` 更新指令 signer 不合法。 |
| 15 | 0x0f | InvalidMarketAccounts | Base 与 Quote Mint 相同，禁止创建 | 创建市场指令中 base/quote mint 相同导致拒绝。 |
| 16 | 0x10 | MarketDisabled | 市场被停用 | `MarketConfigV0.enabled == 0`。需联系管理员恢复或切换其他池。 |
| 17 | 0x11 | InsufficientBalance | 余额不足 | Vault 或用户 token 账户余额不足以完成 swap/deposit/withdraw。 |
| 18 | 0x12 | SlippageExceeded | 超出可接受滑点 | 边际费率（edge）过高（`DEFAULT_MAX_EDGE_MILLI_BIPS`），或成交价偏离预期。调宽滑点或降低交易量。 |
| 19 | 0x13 | InvalidEdgeConfig | Edge 配置非法 | `bid/ask/time/trade` spline 数据非法或超界。验证 config 序列化内容。 |
| 20 | 0x14 | InvalidSafetyConfig | 安全配置非法 | 安全参数（退避、阈值等）校验失败。检查配置写入流程。 |
| 21 | 0x15 | InvalidRetreat | 退避参数非法 | Retreat 计算结果不在允许范围，通常是参数符号或 magnitude 异常。 |
| 22 | 0x16 | InvalidRetreatQuoteAmount | 退避目标 quote 数量非法 | `retreat_quote_amount` 为 0 或与 retreat 公式不兼容。 |
| 23 | 0x17 | StaleOraclePrice | 预言机价格过期 | `oracle_account.price_updated_slot` 与当前 slot 差距超过安全区，需刷新 OKX/Oracle 数据。 |
| 24 | 0x18 | MarketPriceNotSet | 市场价格尚未设置 | 价格缓存为空；先调用价格更新流程。 |
| 25 | 0x19 | InvalidSpline | Spline 序列化数据非法 | 解析 `Spline` 结构失败，检查 `Spline.len` 与数组长度。 |
| 26 | 0x1a | InvalidCopyIntoSpline | 写入 Spline 数据失败 | `copy_from_slice` 时长度不匹配或索引越界。 |
| 27 | 0x1b | InvalidCopyIntoEdgeSpline | Edge Spline 写入失败 | 同上，针对 edge spline。 |
| 28 | 0x1c | InvalidEdgeUpdate | Edge 更新非法 | Edge 更新指令参数检查失败，或新边参数超出限制。 |
| 29 | 0x1d | InvalidOracleAccount | 预言机账户无效 | Oracle Pubkey、owner 或数据结构与市场记录不符。确认是否传入正确的 OKX Oracle PDA。 |
| 30 | 0x1e | InvalidToken2022TransferInstruction | Token-2022 转账指令无效 | 指令数据未按 Token-2022 标准构造，或市场不支持 Token-2022。 |
| 31 | 0x1f | InvalidToken2022Mint | Token-2022 Mint 无效 | Token-2022 mint 字段与市场配置不匹配。 |
| 32 | 0x20 | InvalidDepositParams | 存款参数非法 | 存款数量为 0、方向错误或账户组合不完整。 |
| 33 | 0x21 | InvalidWithdrawParams | 提款参数非法 | 提款数量为 0、方向错误或账户组合不完整。 |
| 34 | 0x22 | InvalidSwapParams | Swap 参数非法 | `amount_in == 0`、方向标记错误、或缺少必需账户。 |
| 35 | 0x23 | InvalidMintAccount | Mint 账户无效 | Mint owner/decimals 不符合预期。 |
| 36 | 0x24 | ErrorDeserializingMarketConfigV0 | 反序列化市场配置失败 | `MarketConfigV0` 字节长度或对齐不正确。 |
| 37 | 0x25 | ErrorDeserializingGlobalConfig | 反序列化全局配置失败 | 全局配置账户数据损坏或版本不支持。 |
| 38 | 0x26 | InvalidGlobalConfig | 全局配置账户无效 | PDA、owner 或尺寸校验失败。 |
| 39 | 0x27 | ErrorDeserializingMarketPrice | 反序列化市场价格失败 | 市场价格缓存数据损坏。需重置或重新写入价格账户。 |
| 40 | 0x28 | SwapFairPriceConversionError | 公允价换算失败 | `OracleAccount::swap_fair_price_conversion` 溢出或参数超界。检查 oracle 数据与输入数量。 |
| 41 | 0x29 | InvalidMarketConfig | 市场配置整体无效 | 版本未知或内部字段不满足约束。 |
| 42 | 0x2a | ErrorSerializingMarketLogRecord | 市场日志序列化失败 | 记录结构未能写入日志缓冲；检查日志账户及尺寸。 |
| 43 | 0x2b | InvalidGlobalConfigAccountBytesUpdate | 全局配置字节更新非法 | 尝试局部修改全局配置时数据偏移/长度无效。 |
| 44 | 0x2c | InvalidBufferAccount | 缓冲账户无效 | 更新流程使用的 buffer 账户不是 PDA 或尺寸不足。 |
| 45 | 0x2d | InvalidAccountManagerProgram | Account Manager 程序无效 | 相关指令提交的管理程序 ID 不在白名单。 |
| 46 | 0x2e | InvalidMarketConfigVersion | 市场配置版本无效 | `config_version` 超出 `MarketConfigVersion` 枚举。 |
| 47 | 0x2f | ErrorLoadingSwapConfigs | 加载 Swap 配置失败 | 从账户读取多段配置时反序列化失败或数据缺失。 |
| 48 | 0x30 | ErrorComputingRetreatedFair | 计算退避后公允价失败 | `get_fair_with_inventory_retreat` 计算中出现溢出/除零。 |
| 49 | 0x31 | InvalidAmountOut | 计算出的成交量非法 | `amount_out` 超过公允值或溢出。尝试减小交易量或检查 OKX multiplier。 |
| 50 | 0x32 | InvalidEdgeMilliBips | 边际费率非法 | Edge 计算结果 ≤ 0 或 `NaN`，常见于 multiplier=0。 |
| 51 | 0x33 | ErrorComputingTargetRetreat | 计算目标退避失败 | `retreat_quote_amount` 为 0 或乘除溢出。 |
| 52 | 0x34 | ErrorComputingInventory | 库存计算失败 | 计算库存差值时溢出，或 oracle 数据异常。 |
| 53 | 0x35 | InvalidAdmin | 管理员账户无效 | 用于敏感操作的 signer 不在管理员列表。 |

> **排查建议**：
> 1. 先根据错误码定位具体枚举；如需进一步确认，可在 `rust-solana-solfiv2-sdk` 仓库中搜索该枚举的使用位置。
> 2. 对账户类错误，重点核对指令传入的账户顺序、owner、mint、signer 标记以及 PDA 种子。
> 3. 对配置类错误，优先 dump 市场账户 (`market_account_header` + `MarketConfigV0`) 与全局配置，检查字段范围。
> 4. 若为计算类错误（`ErrorComputing*`），可开启 `hotpath` 或在客户端增加额外日志，确认输入参数是否超出规模。

