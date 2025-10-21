# Aquifer Swap 解码笔记

## 汇编入口概览
- `function_32490` 是指令入口，`data[0] == 2` 对应 swap 变体（`reverser/aquifer/asm/disassembly.out:33853`）。
- 指令数据长度至少 0x21 字节，随后读取多段 32/48 字节块并与账户元数据比对。

## PDA 派生关系（推导自汇编）
- `dex = PDA(["dex", dex_owner], PROGRAM_ID)`（`function_24054`，`reverser/aquifer/asm/disassembly.out:24054`）。
- `dex_instance = PDA(["dex_instance", dex, [instance_id]], PROGRAM_ID)`（`function_24274`）。
- `coin = PDA(["coin", dex, mint], PROGRAM_ID)`（`function_22197`）。
- `coin_managed_ta = PDA(["coin_managed_ta", coin], PROGRAM_ID)`（`function_21963`）。

## `dex_instance` 账户推断布局（字节偏移）
> 以 0 为账户起始，默认 8 字节 Anchor 判别符。

| 偏移 | 长度 | 含义 | 依据 |
| ---- | ---- | ---- | ---- |
| 0x00 | 8 | Anchor discriminator | Anchor 模板 |
| 0x08 | 32 | `dex_owner` | `dex` 派生只使用该字段（`function_24054`） |
| 0x28 | 32 | `dex` PDA（校验用） | 与上方 seeds 对比 |
| 0x48 | 1 | `dex_bump` | `ldxb [r8+0x48]` |
| 0x49 | 1 | `instance_id` | `ldxb [r8+0x49]`，允许 0-31（`0x1000615e6` 字符串提示） |
| 0x4A | 1 | `base_coin_bump` | `ldxb [r8+0x4a]` |
| 0x4B | 1 | `quote_coin_bump` | `ldxb [r8+0x4b]` |
| 0x4C | 32 | `base_coin` PDA | `ldxdw [r8+0x58]` 等拷贝校验 |
| 0x6C | 32 | `quote_coin` PDA | `ldxdw [r8+0x78]` |
| 0x8C | 32 | `base_oracle` | `ldxdw [r8+0x8]` 配合分支判断 |
| 0xAC | 32 | `quote_oracle` | 同上 |
| 0xCC | 32 | `mm_state / risk` 账户 | `ldxdw [r8+0x28]` |

> 0x4C 之后的字段抓取引用频繁，与 `function_18162` 等逻辑一起校验做市商账户与 oracle。

## `coin` 账户推断布局
| 偏移 | 长度 | 含义 | 依据 |
| ---- | ---- | ---- | ---- |
| 0x00 | 8 | Anchor discriminator | Anchor 模板 |
| 0x08 | 32 | `dex_instance` | `function_22197` 将其与 swap 提供的 `dex_instance` 比较 |
| 0x28 | 32 | `mint` | 用于复算 `coin` PDA（`function_22351` 比对） |
| 0x48 | 32 | `coin_managed_ta`（vault） | 通过 `["coin_managed_ta", coin]` 校验 |
| 0x68 | 32 | `oracle` | 与指令账户校验（错误字符串 `"Wrong oracle passed..."`） |
| 0x88 | 32 | `mm_risk_account` | `function_18162` 读取 |
| 0xA8 | 4 | `decimals`/`flags` | `ldxb [r2+0x6c]` 等操作 |

## swap 指令校验流程摘要
1. 入口判别变体后，复制指令数据至栈。
2. 校验 `dex`、`dex_instance`、`coin`、`coin_managed_ta`、oracle、MM/Risk 账户与 PDA 一致。
3. 对 `coin` 账户读取 `mint`，复算 PDA，确保同 `dex_instance` + `mint` 对应。
4. 逐字段拷贝 `dex_instance` 中的 base/quote 结构，与实际传入账户比对；若缺少或错误即触发 `0x100060d02` 等错误信息。
5. 完成账户验证后，进入价格、深度、风险等计算分支。

## 实现 decode 脚本的关键要点
- 仅需池子(`dex_instance`)地址即可拉取：
  1. 解析 `dex_owner`、`dex_bump`、`instance_id`；
  2. 取出 base/quote `coin` PDA、oracle、MM 账户；
  3. 拉取各 `coin` 账户，得到 `mint`/vault/`coin_managed_ta`，并复算 PDA 校验；
  4. 继续拉取 `mint` 信息确定 TokenProgram、decimals；
  5. 将必需账户按 swap 指令顺序输出，并可选计算用户 ATA。
- PDA 复算需覆写 `find_program_address`；脚本可复用 Tessera V 的实现。
- 缺失账户应显式报错，而非默 silently 失败。

## TODO 列表
- 进一步确认 `dex_instance` 后续字段（MM 风险、撮合配置）含义。
- 解析 `coin` 账户低位 flags（0x6c 之后字节），定位 decimals / side。
- 补充指令数据字段解析（amount、slippage、side），便于生成落地指令。
- 基于真实池子样本对偏移做回归测试。
