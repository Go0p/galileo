# Saros Swap 逆向笔记

## 背景与目标
- 程序二进制：`reverser/saros/saros.so`。
- 程序 ID：`SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr`（Saros AMM 基于 SPL Token Swap 的分叉）。
- 目标：给定 Saros 池子（swap state）地址，通过 RPC 解析一次标准 `Swap` 指令所需的全部账户；脚本风格与 `reverser/tessera_v/tessera_v_decode.py` 保持一致。

## 关键常量
| 含义 | 说明 |
| ---- | ---- |
| Saros Program ID | `SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr` |
| SPL Token v1 Program | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| SPL Token-2022 Program | `TokenzQdSbnjHr1P1a9wYFwS6gkU1GGzLRtXju6Rjt92` |
| Associated Token Program | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` |
| Sysvar Instructions | `Sysvar1nstructions1111111111111111111111111` |

> Saros 仍沿用 SPL Token Swap 的 PDA 规则：`authority = Pubkey::create_program_address([swap_state, [bump_seed]], program_id)`。`bump_seed` 即 swap state 第三个字节。

## Swap 指令账户顺序
与 SPL Token Swap 的 `Swap` 指令一致，按索引列出：

1. `swap_state`：池子账户（本地简称 `pool`）
2. `authority`：Saros 池子的 PDA
3. `user_transfer_authority`：用户签名者/委托地址
4. `user_source`：用户输入代币账户
5. `pool_source`：池子 vault（token_a 或 token_b，视方向而定）
6. `pool_destination`：另一侧 vault
7. `user_destination`：用户接收代币账户
8. `pool_mint`：LP 代币 mint
9. `fee_account`：平台手续费账户
10. `token_program`：SPL Token v1 或 Token-2022

方向切换仅互换第 4-7 项的来源/目标账户。脚本需要根据输入配置输出两侧序列。

## Swap State 布局
Saros 的账户数据结构与 `spl_token_swap::state::SwapV1` 完全一致，以下偏移均以十六进制表示，字段长度除特别注明均为 32 字节：

| 偏移 | 字段 | 说明 |
| ---- | ---- | ---- |
| `0x00` | `is_initialized (u8)` | `1` 表示池子已初始化 |
| `0x01` | `nonce (u8)` | Saros 自身用途（通常为 `1`） |
| `0x02` | `bump_seed (u8)` | 计算 authority PDA 时使用 |
| `0x03` | `token_program_id` | token program（部分池子与真实金库 owner 会不一致，脚本以金库 owner 为准） |
| `0x23` | `token_a` | Saros 金库 A |
| `0x43` | `token_b` | Saros 金库 B |
| `0x63` | `pool_mint` | LP mint |
| `0x88` | `fee_account` | 平台手续费账户 |
| `0xA3` | `token_a_mint` | Vault A 对应的 mint（实务中建议通过金库账户再确认） |
| `0xC3` | `token_b_mint` | Vault B 对应的 mint（同上） |
| `0xE3` | `token_a_deposit (u64 LE)` | 历史入金累加，用于限制 |
| `0xEB` | `token_b_deposit (u64 LE)` |  |
| `0xF3` | `token_a_fees (u64 LE)` |  |
| `0xFB` | `token_b_fees (u64 LE)` |  |
| `0x103` | `fees (Fees)` | 8 × `u64`：`trade_fee_numerator` 等 |
| `0x143` | `curve_type (u8)` | `0`=ConstantProduct, `1`=ConstantPrice, `2`=Stable, `3`=Offset |
| `0x144` | `curve_parameters` | 72 字节，取决于 curve 类型 |

> 汇编中大量 `ldxdw r*, [rX+0x??]` 读写与以上偏移对应，可在 `function_7796`、`function_5980` 中看到对 `token_a`、`token_b` 等字段的校验。

## 脚本设计要点
1. **输入参数**：`--rpc`（默认 `http://127.0.0.1:8899`）、`pool`（必选）、`--user`（可选）、`--direction`（`a2b` / `b2a`）、`--json`。  
2. **解析流程**：
   - `getAccountInfo` 读取池子原始数据，解析上述字段；
   - 根据 `bump_seed` + `pool` + program ID 计算 authority；
   - 通过 `token_a` / `token_b` 的 SPL Token 账户获取真实的 `mint` + `decimals`，与 state 中记录的值交叉验证；
   - 若提供 `--user`，使用 Associated Token Program 计算用户输入/输出 ATA；
   - 按方向输出账户列表与补充信息（mint、decimals、fee 配置等）。
3. **观测信息**：建议同时输出 `trade_fee_numerator` 等指标，方便后续落地监控。

## 验证建议
- 任选 Saros 主网池子，脚本解析后与实际指令进行对比（可从 Jupiter 路由器交易日志中抓取 swap 调用验证账户顺序）。
- 针对 Token-2022 池子（如存在）做一次回归，确认 ATA 计算逻辑适配。
- 将典型池子数据（Base64）写入单元测试夹具，验证字段偏移解析不会被回归破坏。
