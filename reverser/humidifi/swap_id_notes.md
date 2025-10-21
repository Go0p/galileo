# HumidiFi `swap_id` 生成与指令编码备忘

## 计数存储与读取

- 最新 `swap_id` 存在 **池配置账户**（即 swap 指令里的第二个账户）偏移 `0x2b0`。  
- 保存时与常量 `SWAP_ID_MASK = 0x6e9de2b30b19f9ea` XOR 混淆。  
- 解码步骤：读取 8 字节（little-endian）→ 与 `SWAP_ID_MASK` XOR → 得到 `last_swap_id`。

> Python 参考：`reverser/humidifi/generate_swap_id.py`、`swap_id_utils.py`。

## Jupiter 路由器的生成策略

1. 读取配置账户并解出旧值；  
2. 选取任意 **严格大于** 旧值的 64 位整数（常见写法 `last + 1`，也可自行加随机步长避免并发冲突）；  
3. 构造 `SwapParams { swap_id, amount_in, is_base_to_quote, padding: [0; 7] }`；  
4. Borsh 序列化得到 24 字节，再追加 `HUMIDIFI_SWAP_SELECTOR = 0x04` → 25 字节明文；  
5. 对明文执行双层 XOR 混淆（`HumidifiAccounts::obfuscate_instruction_data`），再发起 CPI。

链上校验只做 `new_swap_id > stored_swap_id`。使用旧值会被直接拒绝。

## 指令解码流程

1. **第一层**：逐 8 字节 XOR `0xC3EBBAE2FF2FFF3A`，并叠加随块递增的 `0x0001_0001_0001_0001`；  
2. **第二层**：对前 8 个 qword 依次 XOR `0xEF4A…` 常量表，尾部不足 8 字节使用 `0xEF42578467DDF083`。

示例（`HumidiFi.txt` 第二笔指令）：

```
原始 ix data : c76fe286da5f5351ed7c6261e0baeac338ff2dffe0bae9c33d
第一层解码   : fd90cd7938e5b892d6834c9e0300000000000000000000003d
最终明文     : swap_id=10572452155976683773
               amount_in=17242971869902566236
               is_base_to_quote=0
               selector=0x04
```

Python 复刻：`loss_program/reverser/humidifi/swap_id_from_pool.py::_decode_humidifi_ix`。

## 配置账户中的其他字段

- Base / Quote mint 各占 32 字节，仍存放在配置账户中但经过 4 组 64-bit 常量异或：  
  - Quote mint 位于偏移 `0x180`；Base mint 位于偏移 `0x1A0`；  
  - 解码时对每个 8 字节片段依次 XOR `0xFB5CE87AAE443C38`、`0x4A2178451BAC3C7`、`0x4A1178751B9C3C6`、`0x4A0178651B8C3C5`。  
- 结合 `getTokenAccountsByOwner(pool)` 可通过 mint 精确定位 base / quote vault，无需依赖历史交易。

## 构造指令时的注意点

- 始终读取最新配置账户得到 `last_swap_id`，再生成新值；  
- Jupiter 路由器期望在 `remaining_accounts[4]` 提供一个“HumidiFi 参数公钥”：前 8 字节为小端编码的 `swap_id`，其余 24 字节填 0；  
- 实际 HumidiFi CPI 账户顺序：
  1. payer / transfer authority（外部签名者）  
  2. pool/config 账户  
  3. pool base vault  
  4. pool quote vault  
  5. 用户 base token account  
  6. 用户 quote token account  
  7. `SysvarC1ock11111111111111111111111111111111`  
  8. SPL Token Program (`Tokenkeg…` 或 Token-2022)  
  9. `Sysvar1nstructions1111111111111111111111111`

## 常量速查

| 名称                   | 数值 / 含义                                 |
|------------------------|---------------------------------------------|
| `SWAP_ID_MASK`         | `0x6e9de2b30b19f9ea`                        |
| `INSTRUCTION_MASK`     | `0xc3ebbae2ff2fff3a`                        |
| `HUMIDIFI_SWAP_SELECTOR` | `0x04`                                   |
| 第一层递增步长         | `0x0001_0001_0001_0001`                     |
| 第二层掩码组           | `0xEF4A578C67D5F08B` … `0xEF4D578B67D2F08C` |
| Mint 掩码序列          | `0xFB5CE87AAE443C38`, `0x4A2178451BAC3C7`, `0x4A1178751B9C3C6`, `0x4A0178651B8C3C5` |

更多示例：`generate_swap_id.py`、`swap_id_utils.py`、`swap_id_from_pool.py`。
