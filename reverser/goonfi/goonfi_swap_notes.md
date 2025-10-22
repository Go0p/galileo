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
- 2025-02-15：细读 `litesvm-0.8.1` 源码确认 `LiteSVM::set_builtins` 固定调用 `agave_syscalls::create_program_runtime_environment_v1` 构造 `BuiltinProgram`，且 `BuiltinProgram` 的函数注册表 `FunctionRegistry` 只暴露不可变引用，外部无法覆写既有符号。要在 `sol_try_find_program_address` 处打桩只能 fork/clone `litesvm`，然后：
  - 把 `create_program_runtime_environment_v1` 的实现复制到本地（可放 `third_party/litesvm_patched/src/runtime.rs`），唯一改动为将 `result.register_function("sol_try_find_program_address", ...)` 替换成自定义 wrapper，例如 `recording::SyscallTryFindProgramAddressWithLog::vm`。
  - wrapper 内部直接搬运原 `SyscallTryFindProgramAddress::rust` 逻辑：先解析 seeds，再写入 `MockSlice`。额外插入 `Recorder::global().lock().push(SeedCapture { seeds, program_id })`，随后调用原始实现完成地址搜索，保持返回值兼容。
  - 在 patched `LiteSVM::set_builtins` 中改为调用新的 `create_program_runtime_environment_v1_with_hook`，并保留 `program_runtime_v2` 走原始路径（无需拦截）。
  - 通过 `[patch.crates-io]` 将 `Cargo.toml` 的 `litesvm = "0.8.1"` 指向本地 fork，避免全局 `unsafe` 指针操作。运行模拟前记得在 `Recorder` 上暴露 `drain()` 接口，把 seeds dump 成 JSON 供后续 diff。
  - 当前 `agave_syscalls` 授权为 Apache-2.0，复制实现时需保留版权头并在新文件注明来源 commit（`agave-syscalls-3.0.7`），防止后续升级忘记同步差异。
- 2025-02-16：完成 `function_9880` 的翻译。重要结论：
  - `.rodata@0x1000172f0` 是 35 个 `u32` 的查表序列，低 21 bit 为升序阈值，高 11 bit 为比特位偏移。`function_9880` 对输入值执行 upper-bound 二分查找，找到首个阈值大于输入的槽位。
  - 对应槽位的 bit offset 称为 `bit_offset`，下一槽位的 offset 用于计算跨度 `span = bit_offset_next - bit_offset - 1`。`span == 0` 时直接取 `parity = bit_offset & 1`。
  - 若 `span > 0`，会去 `.rodata@0x100017378`（752 字节）连续累加权重，直到累加和首次超过 `value - prev_threshold`。命中的下标 `bit_offset + delta` 将作为最终 parity 来源。
  - 函数返回值仅为 `parity`（0/1），但 `slot_index`、`prev_threshold`、`bit_offset` 等都可以从查表过程推导。我们已将完整翻译落在 `goonfi_seeds._decode_bitfield`，便于后续脚本直接拿到这些元数据。
- 2025-02-16（晚）：把 `function_9362` / `function_9274` 的 fast-path 数据解压成结构化表：
  - `goonfi_seeds.py` 新增 `_read_varint_stream` / `_deltas_to_intervals` 工具，现已可以将 `.rodata@0x100017162` 与 `.rodata@0x100016e0a` 的差分序列还原为显式区间列表，便于脚本直接做 membership 判断。
  - `FAST_OPCODE_BYTES` 复刻了汇编里 `(hi_byte → 允许的 lo_byte 集合)`，共 40 个分段；`FALLBACK_SHORT_INTERVALS` 则对应 fallback parity 检查的 183 个区间。
  - `_dispatch_seed_builder` 现可对 `< 0x10000` 的 opcode 做与汇编一致的校验，若命中 fallback 则抛出 `NotImplementedError`，提醒后续继续逆向 seed 拼装逻辑。
- 2025-02-17：补完 `function_9362` 的“密集”分支（`65536 ≤ opcode < 131072`）：
  - `.rodata@0x100016ce2/0x100016d3a` 被解读为第二套 `(hi_byte → lo_bytes…)` 索引表，脚本中已暴露为 `DENSE_OPCODE_BYTES`。
  - `_dispatch_seed_builder` 现在能够直接返回上述表项对应的原始字节（目前推测为单字节 seed 片段）。若 opcode 命中差分区间但未出现在两张显式表里，仍会抛出 `NotImplementedError`，提示需要进一步分析真正的动态构造逻辑。
- 2025-02-17（晚）：解析 `function_9031` 使用的 6-bit 编码流，构建 `(hi, lo) → symbol_index` 映射。
  - `goonfi_seeds.py` 新增 `_decode_opcode_payload` 与 `_build_opcode_index_map`，自动将 `.rodata@0x17040` / `.rodata@0x16d3a` 中的压缩索引展开，输出 `FAST_OPCODE_INDEX` / `DENSE_OPCODE_INDEX`。
  - `_dispatch_seed_builder` 现在返回 `SeedToken { tier, symbol_index }`，下一步只需按 tier 去 `Vec<&[u8]>` 查表即可复刻 `function_9031` 的 seed 组装。
- 2025-02-18：核对 `.rodata@0x100017c10` / `0x100017c60` 等内存块，确认它们仅存放 panic 字符串，无法直接映射到池子 seeds，后续需继续追踪栈上构造逻辑。
  - `function_9031` 针对 `symbol_index` ∈ {0, 9, 10, 13, 34, 92} 存在硬编码分支，分别写入 `b"\\0"`, `b"\\t"`, `b"\\n"`, `b"\\r"`, `b"\\""`, `b"\\\\"`。
  - 根据该结论更新了 `goonfi_seeds._dispatch_seed_builder`，上述符号现已直接返回字节字面量，其余符号维持 `SeedToken{tier, symbol_index}` 占位，后续可在 Python 侧逐步复刻剩余拼装指令。
- 2025-02-18（续）：统计 `FAST/DENSE_OPCODE_INDEX` 后发现，除上述转义字符外，大部分 `symbol_index` 落在 ASCII 32..126 区间，且与字符编码一一对应（例如 0x378 → 32 `' '`, 0xfffe → 48 `'0'`）。  
  - 已在 `_dispatch_seed_builder` 中将这些索引直接转换为单字节 `bytes([symbol_index])`，这样 498 条 opcode 映射现在会立即产出可读字符，`SeedToken` 仅保留在 tier>0 或数值≥127 的复杂分支上。  
  - 进一步缩小了未解码范围：当前仍需解析的是 tier=1/2 的 9 个符号，以及 `0x1a7e → symbol_index=4_841_719` 等高位指令，它们对应的字符串尚需结合 bitfield 表和 `function_9031` 的递归逻辑继续研究。
- 2025-02-18（补充）：走读 `function_8757`/`function_9031` 之后确认这套 opcode 解码链条实际用于构造 Rust panic 信息（`\"number not in the range 0..=\"` 等），并不会参与 swap authority 的种子生成。`function_9880`/`function_9449` 在该上下文里只负责把超出表范围的 symbol index 格式化成 `"0x{...}"`，与池子状态无关。  
  - 这意味着我们此前假定 `function_9031` 负责拼装 PDA seeds 是错误线索；真正的种子生成逻辑需从 `function_6816` 附近继续追踪，重点关注访问 `pool_state[0x330]`、`sol_try_find_program_address` 之前的栈写入。
- 2025-02-19：翻译 `function_4682` 内的 router 判别分支。  
  - 汇编通过连续 `memcmp` 判断 aggregator CPI 账户的 `program_id`，三种合法值分别是 `JUP6...`（Jupiter v6）、`6m2CD...`（Step Aggregator）、`T1TAN...`（Goon 黑名单）。  
  - Jupiter 分支还会读取 `route_discriminant = u64::from_le_bytes(account[0x24..0x2c])`，与 7 个常量比较：`0x1cad320090a3b756`、`0x9de0e18ef62cbf0e`、`0xaff11fb02126e0f0`、`0x14afc431ccfa64bb`、`0x819cd641339b20c1`、`0xe9d8fe7c935398d1`、`0x2aade37a97cb17e5`。比较结果写回 `r1`，对应不同的 router variant。  
  - 在 Python 中新增 `extract_route_discriminant` / `identify_router` helper：前者直接从 aggregator account data 的 `[0x24..0x2c]` 解析路由 discriminant，后者根据 `(program_id, discriminant)` 得到 `RouterProgram` 枚举以及临时命名的 `jupiter_route_*` variant。Step/T1TAN 的 discriminant 仍待整理。  
  - 这些分支决定后续 PDA seeds 的取值：`"marketprogram/src/state/market.rs"`、`"vault"`、常量程序 ID（`0x1000165f0` 等）以及 `pool_state[0x330]` bump 会按分支写入栈上 `Vec<&[u8]>`，最终由 `sol_try_find_program_address` 消耗。下一阶段需要精确复刻这些写入顺序，以便脚本可还原 swap authority。我们已确认：  
    - `.rodata@0x1000165b0` → GoonFi 全局状态 PDA（`updap...`），`.rodata@0x100016650` → System Program。  
    - `.rodata@0x1000165f0` / `0x100016570` / `0x100016670` 连续存放 Jupiter / Step / T1TAN 程序 ID；根据 router variant 选择其中一个写入 seeds。  
    - `.rodata@0x1000167eb` 字节串以 `"marketprogram/src/state/market.rs"` + `"vault"` 开头，随后的 0x20 字节为未解码哈希；这些切片在 `function_4682` 中通过 `call function_9989` 被拷贝到栈上，推测是固定的 seed literal。

> 推荐后续步骤：  
> 1. 选取至少一条 GoonFi swap 成功交易，记录 26 个账户顺序，与 `function_6816` 的写入顺序比对，补完账户标签。  
> 2. 解析 `pool_state` 原始二进制，确认风险参数与 router flag 的字段定义，整理成结构体文档。  
> 3. 深入 `function_4682`，定位 swap authority seeds 的来源，并在脚本中实现自动推导 `pool_signer`。  
> 4. 在 `docs/` 目录新增 GoonFi 说明，沉淀 router 枚举值、黑名单策略及监控指标。
