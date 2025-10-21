# HumidiFi 自定义错误码速查

- **程序 ID**：`9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp`
- **来源**：`reverser/humidifi/asm/disassembly.out` 与 `immediate_data_table.out` 中可见的断言/常量。
- **日志格式**：HumidiFi 指令失败会打印 `Program returned error: custom program error: 0x??`。下表按十六进制（前缀 0x）与十进制列出常见情形，并给出排查建议。

## 常见标准错误（转译自 `ProgramError`）

| 十六进制 | 十进制 | 错误含义 | 典型原因 | 排查建议 |
| --- | --- | --- | --- | --- |
| `0x3` | 3 | `InvalidInstructionData` | 上游指令传参无效。最常见的是前置 Token Program 失败后，HumidiFi 将其折叠成 `InvalidInstructionData` 返回。 | 先查看日志中更早的模块错误。例如你遇到的 `Token Program: Error: Account not associated with this Mint` 表明用户 ATA 与 mint 不匹配，应修正 ATA/方向后再重试。 |
| `0x11` | 17 | `InsufficientFunds` | 用户或 Vault 余额不足，或 Token Program 转账失败。 | 检查 `user_base_token` / `user_quote_token` 余额，确认滑点预留。必要时 dump Vault 余额验证。 |
| `0x14` | 20 | `AccountNotRentExempt` | 需要保持租金豁免的账户余额过低。 | 确保创建的中间账户（如临时 token 账户）已补足租金，或复用既有 ATA。 |

> 说明：HumidiFi 内部使用一张表将常见 `ProgramError::*` 映射到自定义码，因此会看到上述 Solana 标准错误名与自定义十六进制值并存。

## 内部断言 / Panic

| 十六进制 | 十进制 | 断言信息 | 含义与排查 |
| --- | --- | --- | --- |
| `0x1` | 1 | `fixed point square root of a negative number` | 浮点/定点计算出现负数开方。通常发生在库存/权重被错误写入（例如行情倒挂或配置脏数据）。请复核最新的 market config、oracle 输入是否为正值。 |
| `0x20` | 32 | `range start index out of range for slice of length …` | Rust slice 越界，意味着程序尝试读取的缓冲区与实际长度不符。常见于传入的市场/配置账户长度错误或版本不匹配。重新拉取 on-chain 账户，确认数据布局与版本。 |
| `0x2a` | 42 | `called Option::unwrap() on a None value …` | 期望存在的配置项为空，例如某一层 route 或指标未初始化。请检查对应的配置写入流程；必要时参考原合约默认值重新初始化。 |

## 调试建议

1. **保留第一时间的完整日志**：HumidiFi 往往在返回自定义错误前已经打印了关键的 `syscall` 或 Token Program 日志，可定位根因。
2. **Dump 市场配置**：出现 Panic 类错误时，优先对 `market`、`global_config` 账户进行 base64 dump，验证结构体字段是否落在安全区间。
3. **复现时收集输入**：记录本地构造 swap 时传入的 `amount`, `direction`, `user_*_token`, `vault_*` 等，便于对照断言。
4. **持续更新对照表**：若在 `reverser/humidifi/asm/immediate_data_table.out` 中发现新的 panic 字符串，只需按相同方式提取尾部 4 字节即可扩充此表。

