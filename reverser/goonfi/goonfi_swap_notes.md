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

- 池子账户数据中的关键偏移（根据最近一次对链上样本的二进制解析）：
  - `0x0E0`：全局状态 PDA 副本；用于 sanity check，真实值仍以常量 `updapqBo...` 为准。
  - `0x100`：base mint（样本池为 `So111...`）。
  - `0x120`：quote mint（样本池为 `EPjFW...`）。
  - `0x140`：base vault（SPL Token account）。
  - `0x160`：quote vault。
  - `0x180`～`0x1F8`：风险参数与交易限额（连续的 `u64`，例如并发 swap 上限、滑点门限等），非公钥。
  - `0x200` 以后至 ~`0x320` 为其它策略常量，目前仍是 `u64` 数值，未观察到额外的账户公钥。
  - `0x330`：PDA bump（样本池 Jupiter 分支中取值 252），供运行时构造 swap authority 使用。
  - `0x388`：router type 标志位（`ldxb r1, [r2+0x388]`）。  
    - `0` → Jupiter 直连。  
    - 其他值触发备用撮合分支（`T1TAN...`、`6m2CD...` 等）。
  - `0x38E`：黑名单开关（`stb [r2+0x38e], {0|1}`）。

## 账户装配流程摘要
- `function_2231`：校验池子结构并写入 base/quote vault、撮合路由配置。如果任一 `memcmp` 失败会返回错误码 6/22。
- `function_4705`：对路由账户（OpenBook 市场及子账户）做结构校验，包括 `account[0].data[0x50] == 165`（表明 Anchor discriminator）。最终要求账户总数为 26（`mov64 r9, 26`）。
- `function_3274`：对风险参数进行范围检查（最大值 1_000_000），并确保池子中多段 u64 单调递增，以此阻止异常曲线。
- `function_6816`：根据 router type 将账户重排进最终 26 个 `AccountMeta` 槽；同时写入每个账户的只读/可写标志（`sth [r10-0xf8], 1` 等语句），并根据 `pool_state` 中的 bitmask 判定用户是否走黑名单流程。  
  该函数在 Jupiter 分支中会调用 `function_4682` / `function_4705` 逐步写入账户元信息，并将 swap authority seeds + bump 保存在寄存器堆栈上；最终只把 bump（`pool_state[0x330]`）持久化。
- `function_9274`：出现多次 `call function_8841` / `8840`，并从 `.rodata` 的 `0x100017cc0`、`0x100017ca8` 等常量构造字节切片，随后通过 `function_7764`/`7763` 组合成 `Vec<&[u8]>`。该逻辑紧跟着 `sol_try_find_program_address` 导入函数（位于 `.rodata` 0x100017d78），极可能就是 swap authority seeds 的生成位置：先将若干常量块复制到栈上，再调用 `sol_try_find_program_address` 计算 PDA。

## 未解决点 / 待补充
- 账户顺序虽然可以通过 `function_6816` 推导，但目前尚未完全复刻 26 个槽位与 router type 的映射。需要结合实际链上交易进一步验证。
- `0x38E` 的黑名单流程与 `T1TAN...`、`2gav...` 等 PDA 的含义仍待确认，推测与外部黑名单合约交互有关。
- Jupiter 分支 CPI 使用的 swap authority (`CGDgsTDL...`) 未直接存储在池子数据中；只在运行时根据一组 seeds 调用 `find_program_address` 得到。目前仅能从池子中读到 bump 值 252，具体 seeds 需继续跟踪 `function_4682` 内部对 `r9+0x8`、`r9+0x330` 写入的来源。
- 汇编中多次调用 `function_11391`、`function_10694` 等数值计算函数，尚未反推出完整费用公式；后续若要做性能优化需再细看。

## 近期进展摘记
- 新样本池：`4uWuh9fC7rrZKrN8ZdJf69MN1e2S7FPpMqcsyY1aof6K`，链上 swap authority 为 `3aypM9ab212G5jHDhwzorP8ifnXirpGpAbeThvvD7G49`，`pool_state[0x330]` bump=255。与 `4ynTYgJK...`（bump=252）对比，`pool_state[0x330..0x338]`、`0x338..0x340` 的 u64 数值发生变化（0x06fcfdfc → 0x06f7ffff，0x071a667a → 0x071a8d75），推测这些字段是 seeds 构造时的动态索引或长度标记。
- `function_9031/9030` 的调用栈中会读取 `.rodata` 的压缩表（`0x1000172f0`、`0x100017378`、`0x100017b70` 等），同时配合常量 `"0x"`、`"0123456789abcdef"` 生成字符串，说明最终的 seed 片段很可能是十六进制文本。解码流程大致为：读取池上配置的位移 → 按表解码出 nibble/长度 → `function_7764` 将片段写入 `Vec<&[u8]>` → `sol_try_find_program_address`。
- 为定位具体 seeds，当前计划是复刻 `function_9031` 的位运算解码逻辑；若能在本地模拟，便能直接 dump seed 列表并在 Python 中调用 `create_program_address` 校验 `CGDgsTDL...` 与 `3aypM9ab...`。另一条备选方案是利用 `solana_rbpf`/自定义 syscall 钩子，在执行 Jupiter 路径时截获传入 `sol_try_find_program_address` 的 seed 列表。
- 解析 `function_9031` 需要重点还原三个辅助函数：`function_9880`（基于 `0x1000172f0` 的索引表进行二分查找）、`function_9449`（把 `u64` 格式化为 `"0x{..}"` 字符串）、`function_9362`（区间校验）。后续将先把这三段逻辑翻译成 Python，再围绕实际池子的输入跑通 `function_9031` 主流程，拿到完整 seeds。
- 2025-02-14：评估了 `litesvm` 方案。框架初始化时直接把 `sol_try_find_program_address` 等 syscalls 注册到内部 `BuiltinProgram`，外部暂无安全接口可覆写，想偷梁换柱必须用 `unsafe` 操作底层函数表。下一步计划是在自定义 wrapper 内复用 `agave_syscalls` 的解析逻辑，记录 seeds 后再调用原 syscall，从而无需手翻 `function_9031`。

> 推荐后续步骤：  
> 1. 选取至少一条 GoonFi swap 成功交易，记录 26 个账户顺序，与 `function_6816` 的写入顺序比对，补完账户标签。  
> 2. 解析 `pool_state` 原始二进制，确认风险参数与 router flag 的字段定义，整理成结构体文档。  
> 3. 深入 `function_4682`，定位 swap authority seeds 的来源，并在脚本中实现自动推导 `pool_signer`。  
> 4. 在 `docs/` 目录新增 GoonFi 说明，沉淀 router 枚举值、黑名单策略及监控指标。
