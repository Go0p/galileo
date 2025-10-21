# GoonFi 汇编解析笔记

## 概述
- `reverser/goonfi/asm/disassembly.out` 显示入口首先校验前 6 个账户是否匹配固定常量，随后再执行主体逻辑（参见 `reverser/goonfi/asm/disassembly.out:1-44`）。  
  这些常量解码后分别为：  
  - `updapqBoqhn48uaVxD7oKyFVEwEcHmqbgQa1GvHaUuX`：GoonFi 全局状态 PDA。  
  - `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`：SPL Token Program v1。  
  - `11111111111111111111111111111111`：System Program。  
  因此 swap 指令最开头的账户顺序必须固定为 `[global_state, pool_state, ?, ?, ?, ?, token_program, system_program]`，若不满足则直接 `exit`。

- 在 `function_836`（`reverser/goonfi/asm/disassembly.out:861-920`）中再次对第 0/1/2 条账户做 `memcmp`，并在成功后将 `Sysvar1nstructions1111111111111111111111111` 写入栈指针。此处确认所有 swap call 都需要显式携带 Sysvar Instructions。

- 主流程在 `function_2231`、`function_3274`、`function_3867` 等函数中依次校验池子参数、装配账户列表。`function_3867` 会将池子解析出的账户写入 26 个 `AccountMeta` 槽，如果任何校验不通过就提前返回错误码（例如写入 `stw [r8+0x8], 21` 表示校验失败）。

## 核心常量与偏移
- 常量表（`reverser/goonfi/asm/immediate_data_table.out` + `readelf -x .rodata`）中的关键 32 字节：
  - `0x1000165b0` → `updapqBoqhn48uaVxD7oKyFVEwEcHmqbgQa1GvHaUuX`（全局状态 PDA）。
  - `0x1000165d0` → `Sysvar1nstructions1111111111111111111111111`（Sysvar Instructions）。
  - `0x1000165f0` → `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`（Jupiter v6 程序）。
  - `0x100016630` → `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`（SPL Token Program v1）。
  - `0x100016670` → `T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT`（其他撮合程序）。
  - `0x100016680` → `2gav97pP6WnmsZYStGmeX4wUmJgtsUHzhX7dhjqBBZa8`（黑名单相关 PDA）。

- 池子账户数据中的关键偏移（根据 `function_2231`、`function_4705`、`function_6816` 推断）：
  - `0x158`：base vault。`function_2231` 在校验 `pool_state` 时会将该指针写入栈并要求账户 26 的 `AccountOwner == Token Program`。
  - `0x178`：quote vault。逻辑与 base vault 相同。
  - `0x198`：外部市场/撮合器地址（OpenBook / Jupiter route），其 owner 由 `function_5673` 分支判断。
  - `0x1B8`：市场或 vault signer PDA。没有实际账户数据，如果 `getAccountInfo` 返回空说明是 PDA。
  - `0x1B8` 之后的连续 0x20 段会在 `function_4705` / `function_6816` 中被逐个搬运到 `AccountMeta`，推测依次为 `event_queue`、`bids`、`asks`、`open_orders` 等 OpenBook 组件。
  - `0x388`：router type 标志位（`ldxb r1, [r2+0x388]`）。  
    - `0` → Jupiter 直连（对比常量 `JUP6Lkb...`）。  
    - 其他值会触发备用撮合分支（`T1TAN...`、`6m2CD...` 等）。
  - `0x38E`：黑名单开关（`stb [r2+0x38e], {0|1}`）。

## 账户装配流程摘要
- `function_2231`：校验池子结构并写入 base/quote vault、撮合路由配置。如果任一 `memcmp` 失败会返回错误码 6/22。
- `function_4705`：对路由账户（OpenBook 市场及子账户）做结构校验，包括 `account[0].data[0x50] == 165`（表明 Anchor discriminator）。最终要求账户总数为 26（`mov64 r9, 26`）。
- `function_3274`：对风险参数进行范围检查（最大值 1_000_000），并确保池子中多段 u64 单调递增，以此阻止异常曲线。
- `function_6816`：根据 router type 将账户重排进最终 26 个 `AccountMeta` 槽；同时写入每个账户的只读/可写标志（`sth [r10-0xf8], 1` 等语句），并根据 `pool_state` 中的 bitmask 判定用户是否走黑名单流程。

## 未解决点 / 待补充
- 账户顺序虽然可以通过 `function_6816` 推导，但目前尚未完全复刻 26 个槽位与 router type 的映射。需要结合实际链上交易进一步验证。
- `0x38E` 的黑名单流程与 `T1TAN...`、`2gav...` 等 PDA 的含义仍待确认，推测与外部黑名单合约交互有关。
- 汇编中多次调用 `function_11391`、`function_10694` 等数值计算函数，尚未反推出完整费用公式；后续若要做性能优化需再细看。

> 推荐后续步骤：  
> 1. 选取至少一条 GoonFi swap 成功交易，记录 26 个账户顺序，与 `function_6816` 的写入顺序比对，补完账户标签。  
> 2. 解析 `pool_state` 原始二进制，确认风险参数与 router flag 的字段定义，整理成结构体文档。  
> 3. 在 `docs/` 目录新增 GoonFi 说明，沉淀 router 枚举值、黑名单策略及监控指标。
