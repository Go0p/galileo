# ZeroFi Swap 逆向笔记

## 背景与目标
- 程序二进制：`reverser/zerofi/zerofi.so`
- 目标：仿照 `tessera_v_decode.py`，仅借助 RPC 解析 ZeroFi 池子，输出构造 swap 指令所需的核心账户。
- 主要参考：`reverser/zerofi/asm/disassembly.out`、`immediate_data_table.out`，以及 Jupiter Router 中的 `DEX-Router-Solana-V1/programs/dex-solana/src/adapters/zerofi.rs`。

## 关键常量
| 含义 | 汇编偏移 | Base58 |
| ---- | -------- | ------ |
| ZeroFi 程序 ID | `0x10002cf88` | `ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY` |
| SPL Token v1 Program | `0x10002cf08` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| Sysvar Instructions | `0x10002ce68` | `Sysvar1nstructions1111111111111111111111111` |
| Swap authority PDA (默认) | `0x10002cfe8` | `Sett1ereLzRw7neSzoUSwp6vvstBkEgAgQeP6wFcw5F` |
| Authority 白名单 1 | `0x10002cf68` | `ELF5Z2V7ocaSnxE8cVESrKjwyydyn3kKqwPcj57ADvKm` |
| Authority 白名单 2 | `0x10002cfa8` | `2UUgGySTVXmKFatH7pGQo84ZrzdSYF5zw9iqrGwBMuuj` |

> 汇编中常量均以 32 字节 `lddw` 载入，使用 base58 解码后得到以上地址。

## Swap 指令账户顺序
与 Jupiter DEX router 的适配器完全一致：

1. `pair` 池子账户  
2. `vault_info_in`  
3. `vault_in`  
4. `vault_info_out`  
5. `vault_out`  
6. `user_source_token`  
7. `user_destination_token`  
8. `swap_authority`（需为白名单地址之一，且带签名）  
9. `token_program`  
10. `sysvar_instructions`

`disassembly.out` 中 `function_2784`（含 “Instruction: swap” 字符串）对上述顺序逐项校验，可作为对照。

## Pair 账户关键偏移
Pair 账户数据中直接存储所有关键公钥，按 32 字节对齐：

| 字段 | 偏移 (hex) | 说明 |
| ---- | ---------- | ---- |
| `vault_info_base` | `0x0BA0` | checkpoint 阶段写入 |
| `vault_base` | `0x0BB8` | 实际 SPL Token 账户 |
| `vault_info_quote` | `0x0BC8` |  |
| `vault_quote` | `0x0BD8` |  |
| `cached_vault_base` | `0x0C00` | 方向切换缓存 |
| `cached_vault_quote` | `0x0C08` |  |
| `base_mint` | `0x0BE8` |  |
| `quote_mint` | `0x0C18` |  |
| `swap_authority_pda` | `0x1968` | `sol_try_find_program_address` 结果 |

在 `function_7831`、`function_7544` 等路径中，这些偏移被读入寄存器，随后写入栈帧并用于构造指令账户。

### 标志位
- `data[0x0791] & 1`：是否启用 Token-2022。
- `data[0x079C]`：fast 路径标志，影响费用/方向逻辑。
- `data[0x079F]`：0 表示 base→quote，1 表示 quote→base。

## 白名单校验
`lbb_9034` 之后程序依次对偏移 `0x1288`~`0x1832` 的多个 32 字节块执行 `memcmp`，这是额外的全局账户白名单。解析脚本可按需导出这些字段以调试。

## 解析流程概要
1. 使用 `getAccountInfo` 获取 pair 数据并 base64 解码。  
2. 读取标志位，确定 swap 方向与 Token Program。  
3. 按上表偏移解析 `vault_info` / `vault` / `mint`。  
4. 若已知用户输入 mint，匹配 base/quote 方向，选择正确的一侧作为输入。  
5. 默认取 `swap_authority_pda`，并校验是否在白名单中。  
6. 输出账户数组，顺序严格与合约调用保持一致；补充 Token Program、Sysvar 等常量。

## 验证建议
- 对任意主网 pair 账户执行脚本，确认字段偏移解析正确。  
- 与 Jupiter Router 中实际的 ZeroFi 交易对比账户顺序。  
- 若需集成流水线，可对离线快照编写单元测试，保证关键偏移读取不被回归。
