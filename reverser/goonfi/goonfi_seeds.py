"""
Experimental decoder for GoonFi swap-authority seeds.

This module is a direct translation target for the eBPF helpers
`function_9031`, `function_9880`, `function_9449` 与 `function_9362`.
目前仅梳理出 rodata 表和基础工具函数，后续会逐步填入完整的逻辑。
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from enum import Enum
from typing import Callable, Iterable, Sequence, Tuple, Union

from goonfi_utils import b58decode, find_program_address

GOONFI_SO = Path(__file__).with_name("goonfi.so")
PROGRAM_ID = b58decode("goonERTdGsjnkZqWuVjs73BZ3Pb9qoCUdBUL17BnS5j")

JUPITER_ROUTER_SEED_OFFSETS = (0x248, 0x268, 0x288)
POOL_SIGNER_BUMP_OFFSET = 0x330
POOL_ROUTER_FLAG_OFFSET = 0x388
POOL_BLACKLIST_FLAG_OFFSET = 0x38E


def _read_rodata_slice(offset: int, length: int) -> bytes:
    data = GOONFI_SO.read_bytes()
    return data[offset : offset + length]


def _read_varint_stream(raw: bytes) -> list[int]:
    """
    GoonFi 在多处 .rodata 中使用“7-bit continuation”编码来压缩 u16 序列。

    - 若高位未置位(`byte & 0x80 == 0`)，则直接表示一个 0..127 的数。
    - 若高位置位，后续 7 bit 与紧随其后的 8 bit 共同组成 15 bit 的数值。

    该格式同 `function_9274`/`function_9362` 内的解析完全一致。
    """

    values: list[int] = []
    idx = 0
    end = len(raw)
    while idx < end:
        head = raw[idx]
        idx += 1
        if not (head & 0x80):
            values.append(head)
            continue
        payload = head & 0x7F
        if idx >= end:
            values.append(payload << 8)
            break
        tail = raw[idx]
        idx += 1
        values.append((payload << 8) | tail)
    return values


def _deltas_to_intervals(deltas: list[int]) -> list[tuple[int, int]]:
    """
    在 `function_9274` 的 fallback 分支里，`deltas` 定义了一系列差分边界。
    每次减去一个 delta 都会翻转 parity，相当于构造 `[(a0, b0), (a1, b1)...)`
    这样的闭区间列表。后续只要判断取值是否落在区间内即可。
    """

    intervals: list[tuple[int, int]] = []
    parity = 0
    cursor = 0
    start = 0
    for delta in deltas:
        cursor += delta
        parity ^= 1
        if parity:
            start = cursor
        else:
            intervals.append((start, cursor))
    return intervals


# === function_9449: 把 u64 转成 "0x..." 字符串 ===
def _encode_hex_u64(value: int) -> bytes:
    if value < 0:
        raise ValueError("value must be non-negative")
    return f"0x{value:x}".encode("ascii")


# === function_9880: 压缩表二分查询框架 (待完善) ===
@dataclass
class BitfieldTable:
    entries: Sequence[int]

    @classmethod
    def from_rodata(cls, offset: int, count: int) -> "BitfieldTable":
        raw = _read_rodata_slice(offset, count * 4)
        ints = [int.from_bytes(raw[i : i + 4], "little") for i in range(0, len(raw), 4)]
        return cls(ints)


TABLE_172F0 = BitfieldTable.from_rodata(0x172F0, 96)  # placeholder sizing
TABLE_17378 = _read_rodata_slice(0x17378, 752)


def _decode_fast_opcode_table() -> dict[int, tuple[int, ...]]:
    """
    解析 `0x100016ff0`/`0x100017040` rodata 对。

    - 0x16ff0: (hi_byte, length) 列表，共 40 条。
    - 0x17040: 实际的低位 opcode 列表，按上述 length 拼接。
    """

    index = _read_rodata_slice(0x16FF0, 80)  # 40 * 2
    payload = _read_rodata_slice(0x17040, 290)
    table: dict[int, tuple[int, ...]] = {}
    offset = 0
    for pos in range(0, len(index), 2):
        hi = index[pos]
        length = index[pos + 1]
        chunk = payload[offset : offset + length]
        table[hi] = tuple(chunk)
        offset += length
    return table


def _decode_dense_opcode_table() -> dict[int, tuple[int, ...]]:
    """
    解析 `0x100016ce2`/`0x100016d3a` rodata 对（对应 function_9362 的“密集”路径）。

    - 0x16ce2: 同样是 (hi_byte, length) 对，只是条目更多（44 个）。
    - 0x16d3a: 低位 opcode 的字节串。
    """

    index = _read_rodata_slice(0x16CE2, 88)  # 44 * 2
    payload = _read_rodata_slice(0x16D3A, 400)
    table: dict[int, tuple[int, ...]] = {}
    offset = 0
    for pos in range(0, len(index), 2):
        hi = index[pos]
        length = index[pos + 1]
        chunk = payload[offset : offset + length]
        table[hi] = tuple(chunk)
        offset += length
    return table


def _decode_opcode_pairs(index_offset: int, pair_count: int) -> list[tuple[int, int]]:
    raw = _read_rodata_slice(index_offset, pair_count * 2)
    return [(raw[i], raw[i + 1]) for i in range(0, len(raw), 2)]


def _decode_opcode_payload(
    payload_offset: int,
    expected_items: int,
) -> list[int]:
    """
    解析 function_9031 内使用的 6-bit 编码流。

    编码规则与 ASCII 类似：
      * 最高位未置位时，取单字节数值。
      * 否则根据首字节高 3 位决定后续是否需要追加 1/2/3 个字节，
        每个追加字节仅取低 6 bit，并拼成 12/18/24 bit 的结果。
    """

    data = GOONFI_SO.read_bytes()
    stream = data[payload_offset:]
    values: list[int] = []
    idx = 0
    length = len(stream)

    while len(values) < expected_items:
        if idx >= length:
            raise ValueError(
                f"opcode payload truncated: need {expected_items}, got {len(values)}"
            )
        first = stream[idx]
        idx += 1
        if first < 0x80:
            values.append(first)
            continue
        if first <= 0xDF:
            if idx >= length:
                raise ValueError("missing continuation byte (2-byte symbol)")
            second = stream[idx]
            idx += 1
            value = ((first & 0x1F) << 6) | (second & 0x3F)
            values.append(value)
            continue
        if first <= 0xEF:
            if idx + 1 >= length:
                raise ValueError("missing continuation bytes (3-byte symbol)")
            second = stream[idx]
            third = stream[idx + 1]
            idx += 2
            value = ((first & 0x1F) << 12) | ((second & 0x3F) << 6) | (third & 0x3F)
            values.append(value)
            continue
        if idx + 2 >= length:
            raise ValueError("missing continuation bytes (4-byte symbol)")
        second = stream[idx]
        third = stream[idx + 1]
        fourth = stream[idx + 2]
        idx += 3
        value = (
            ((first & 0x1F) << 18)
            | ((second & 0x3F) << 12)
            | ((third & 0x3F) << 6)
            | (fourth & 0x3F)
        )
        values.append(value)

    return values


def _build_opcode_index_map(
    index_offset: int,
    payload_offset: int,
    pair_count: int,
) -> tuple[
    dict[int, tuple[int, ...]],
    dict[tuple[int, int], int],
]:
    """
    同时解析 `(hi → lo 列表)` 与 `(hi, lo) → symbol index` 映射。
    """

    pairs = _decode_opcode_pairs(index_offset, pair_count)
    total_items = sum(count for _, count in pairs)
    payload = _read_rodata_slice(payload_offset, total_items)

    hi_to_los: dict[int, tuple[int, ...]] = {}
    mapping: dict[tuple[int, int], int] = {}

    offset = 0
    for hi, count in pairs:
        chunk = payload[offset : offset + count]
        if len(chunk) != count:
            raise ValueError(f"opcode payload exhausted: hi=0x{hi:02x}")
        hi_to_los[hi] = tuple(chunk)
        offset += count

    # 6-bit 编码的 symbol index 紧随其后（payload_offset + total_items）
    index_values = _decode_opcode_payload(payload_offset + total_items, total_items)

    idx_iter = iter(index_values)
    for hi, los in hi_to_los.items():
        for lo in los:
            try:
                symbol_index = next(idx_iter)
            except StopIteration as exc:  # pragma: no cover - 安全防御
                raise ValueError("symbol index list shorter than expected") from exc
            mapping[(hi, lo)] = symbol_index

    return hi_to_los, mapping


FAST_OPCODE_BYTES, FAST_OPCODE_INDEX = _build_opcode_index_map(0x16FF0, 0x17040, 40)
DENSE_OPCODE_BYTES, DENSE_OPCODE_INDEX = _build_opcode_index_map(0x16CE2, 0x16D3A, 44)
FALLBACK_SHORT_INTERVALS = _deltas_to_intervals(_read_varint_stream(_read_rodata_slice(0x17162, 398)))
FALLBACK_LONG_INTERVALS = _deltas_to_intervals(_read_varint_stream(_read_rodata_slice(0x16E0A, 486)))


@dataclass
class BitfieldMatch:
    """
    还原自 function_9880 的中间结果。

    Attributes
    ----------
    slot_index:
        对应 TABLE_172F0 中“上界”所在的槽位 (upper bound)，取值 0..33。
    prev_threshold:
        前一个槽位的阈值，亦即 value 所落区间的下界。
    next_threshold:
        当前槽位的阈值 (上界)。
    bit_offset:
        与该区间关联的 bit offset (`table[idx] >> 21`)，用于后续在压缩流里取字节。
    bit_limit:
        下一槽位的 bit offset；若 slot=33 则视为 751（与汇编一致）。
    parity:
        汇编最终 `bit_offset' & 1` 的返回值，function_9880 仅以该布尔值做分支。
    """

    slot_index: int
    prev_threshold: int
    next_threshold: int
    bit_offset: int
    bit_limit: int
    parity: int


@dataclass(frozen=True)
class SeedToken:
    """
    表示 `function_9031` 中的中间 token。

    Attributes
    ----------
    tier:
        依据 index 的范围决定后续查表使用的切片区（0..3）。
    symbol_index:
        原始 6-bit 流解码后的符号编号。
    """

    tier: int
    symbol_index: int


SPECIAL_SYMBOL_LITERALS: dict[int, bytes] = {
    0: b"\\0",
    9: b"\\t",
    10: b"\\n",
    13: b"\\r",
    34: b'\\"',
    92: b"\\\\",
}


class RouterProgram(Enum):
    JUPITER_V6 = "jupiter_v6"
    STEP_AGGREGATOR = "step_aggregator"
    GOON_BLACKLIST = "goon_blacklist"


@dataclass(frozen=True)
class RouterLiteralBlock:
    market_literal: bytes
    vault_literal: bytes
    program_ids: tuple[bytes, ...]


@dataclass(frozen=True)
class FormatFragment:
    """
    复刻 `function_7764` 处理的片段布局。

    汇编行为可概括为：
      * 若首字节为 0x80，则通过上下文指针调用“静态查表”函数；
        0x04..0x07 处的 `u32` 被视作 symbol id。
      * 否则视为内联数据：0x0A/0x0B 分别给出截取范围的起始/结束，
        将 `[base + start, base + end)` 拷贝到目标缓冲。
    """

    raw: bytes

    @property
    def is_static(self) -> bool:
        return bool(self.raw and self.raw[0] == 0x80)

    @property
    def inline_bounds(self) -> tuple[int, int]:
        if len(self.raw) < 0x0C:
            raise ValueError("fragment 长度不足 0x0c 字节")
        start = self.raw[0x0A]
        end = self.raw[0x0B]
        if end < start:
            raise ValueError(f"fragment 终止偏移早于起点: start={start}, end={end}")
        return start, end

    @property
    def static_symbol(self) -> int:
        if len(self.raw) < 0x08:
            raise ValueError("fragment 用作静态 symbol 时至少需要 8 字节")
        return int.from_bytes(self.raw[0x04:0x08], "little")

    def slice_inline_payload(self) -> bytes:
        start, end = self.inline_bounds
        if end > len(self.raw):
            raise ValueError(
                f"fragment 内联数据越界: end={end}, len={len(self.raw)}"
            )
        return self.raw[start:end]


@dataclass
class FormatDispatchTable:
    """
    对标 `function_7764` 的上下文指针（`r2` 指向的结构）。

    - `resolve_static(symbol_id)` 模拟 `[ctx+0x28]->fn(ctx[..], symbol_id)`。
    - `emit_chunk(bytes_chunk)` 对应 `[ctx+0x20]->sink`。
    """

    resolve_static: Callable[[int], bytes]
    emit_chunk: Callable[[bytes], None]


def replay_fragment(fragment: FormatFragment, table: FormatDispatchTable) -> None:
    """
    Python 版本的 `function_7764`：根据 fragment 的布局执行复制 / 查表。

    Parameters
    ----------
    fragment:
        `FormatFragment` 结构，来自汇编栈上的 `Vec<fmt::rt::Piece>`。
    table:
        汇编里通过 `[r2+0x20]`、`[r2+0x28]` 访问的调度表，抽象成两个回调。
    """

    if fragment.is_static:
        symbol = fragment.static_symbol
        table.emit_chunk(table.resolve_static(symbol))
        return

    payload = fragment.slice_inline_payload()
    if not payload:
        return
    table.emit_chunk(payload)


def validate_utf8_slice(data: bytes, start: int, end: int) -> None:
    """
    复刻 `function_9030`/`9031` 在 Rust 标准库中对 `&str[start..end]` 的边界校验。

    - `start`、`end` 需满足 `0 <= start <= end <= len(data)`。
    - `start` 与 `end` 必须位于 UTF-8 码点边界（高两位不为 `10`）。
    违反任一条件时抛出 `ValueError`，模拟汇编里走向 panic 分支。
    """

    length = len(data)
    if start < 0 or end < 0:
        raise ValueError("slice index 不得为负数")
    if start > end or end > length:
        raise ValueError(
            f"slice index 越界: start={start}, end={end}, len={length}"
        )

    def is_char_boundary(idx: int) -> bool:
        if idx == 0 or idx == length:
            return True
        return (data[idx] & 0xC0) != 0x80

    if not is_char_boundary(start) or not is_char_boundary(end):
        raise ValueError(
            f"slice index 不在 UTF-8 边界: start={start}, end={end}"
        )


def _load_router_literal_block() -> RouterLiteralBlock:
    """
    解析 `.rodata@0x1000167eb` 的静态 seed 字符串与路由 Program ID。

    布局推测：
      * 0x00..0x20 → `"marketprogram/src/state/market.rs"`（33 字节）
      * 0x21..0x25 → `"vault"`（5 字节）
      * 0x26       → 常量 0x04（暂未解意义，疑似占位符）
      * 之后连续三个 32 字节 Program ID：JUP6 / 6m2CD / T1TAN。
    """

    literal_offset = 0x167EB
    literal_length = 33 + 5 + 1 + (3 * 32)
    raw = _read_rodata_slice(literal_offset, literal_length)
    market_literal = raw[:33]
    vault_literal = raw[33:38]
    payload = raw[39:]  # 跳过 0x04 标记
    if len(payload) < 3 * 32:
        raise ValueError("router literal block 尺寸异常")
    program_ids = tuple(
        payload[i : i + 32] for i in range(0, 3 * 32, 32)
    )
    return RouterLiteralBlock(
        market_literal=market_literal,
        vault_literal=vault_literal,
        program_ids=program_ids,
    )


ROUTER_LITERAL_BLOCK = _load_router_literal_block()

MARKET_LITERAL = ROUTER_LITERAL_BLOCK.market_literal
MARKET_LITERAL_PREFIX32 = MARKET_LITERAL[:32]
MARKET_LITERAL_SUFFIX = MARKET_LITERAL[32:]
VAULT_LITERAL = ROUTER_LITERAL_BLOCK.vault_literal

ROUTER_PROGRAM_IDS: dict[RouterProgram, bytes] = {
    RouterProgram.JUPITER_V6: ROUTER_LITERAL_BLOCK.program_ids[0],
    RouterProgram.STEP_AGGREGATOR: ROUTER_LITERAL_BLOCK.program_ids[1],
    RouterProgram.GOON_BLACKLIST: ROUTER_LITERAL_BLOCK.program_ids[2],
}


@dataclass(frozen=True)
class AccountMetaView:
    """
    表示内部 Vec<AccountMeta> 中的一项。

    仅记录地址与读写/签名标志，方便在 Python 端复刻 PDA seed。
    """

    pubkey: bytes
    is_signer: bool
    is_writable: bool


# Jupiter/OpenBook 静态账户列表（来自 `.rodata@0x1680c` 起的一段常量）
JUPITER_STATIC_ACCOUNT_OFFSETS: tuple[int, ...] = (
    0x1680C,
    0x1682C,
    0x1684C,
    0x1686C,
    0x1688C,
    0x168AC,
    0x168CC,
    0x168EC,
    0x1690C,
    0x1692C,
    0x1694C,
    0x1696C,
    0x1698C,
    0x169AC,
    0x169CC,
    0x169EC,
    0x16A0C,
    0x16A2C,
    0x16A4C,
    0x16A6C,
    0x16A8C,
    0x16AAC,
    0x16ACC,
    0x16AEC,
    0x16B0C,
    0x16B2C,
    0x16B4C,
)


def load_jupiter_static_accounts() -> list[bytes]:
    """
    读取 `.rodata` 中 Jupiter/OpenBook 固定账户的原始 32 字节。

    这些地址在 `function_4705` 里逐一比对，用于拼装内部 Vec<AccountMeta>。
    """

    accounts: list[bytes] = []
    for offset in JUPITER_STATIC_ACCOUNT_OFFSETS:
        slab = _read_rodata_slice(offset, 32)
        accounts.append(slab)
    return accounts


JUPITER_ROUTE_HASHES: dict[int, str] = {
    0x1CAD320090A3B756: "jupiter_route_a",
    0x9DE0E18EF62CBF0E: "jupiter_route_b",
    0xAFF11FB02126E0F0: "jupiter_route_c",
    0x14AFC431CCFA64BB: "jupiter_route_d",
    0x819CD641339B20C1: "jupiter_route_e",
    0xE9D8FE7C935398D1: "jupiter_route_f",
    0x2AADE37A97CB17E5: "jupiter_route_g",
}


def identify_router(program_id: bytes, route_discriminant: int) -> Tuple[RouterProgram, str]:
    """
    根据 aggregator CPI 账户的程序 ID ＋ route key（account[0x24..0x2c]）推断 router 分支。

    目前仅覆盖 Jupiter v6，若命中未知组合会抛出 KeyError。
    """

    for router, pid in ROUTER_PROGRAM_IDS.items():
        if program_id == pid:
            if router is RouterProgram.JUPITER_V6:
                try:
                    variant = JUPITER_ROUTE_HASHES[route_discriminant]
                except KeyError as exc:
                    raise KeyError(
                        f"未识别的 Jupiter route discriminant 0x{route_discriminant:016x}"
                    ) from exc
                return router, variant
            raise KeyError(f"{router.value} 尚未补齐 discriminant 映射")
    raise KeyError("未知 router program id")


def get_router_program_id(router: RouterProgram) -> bytes:
    """
    返回指定 router 对应的 Program ID 原始 32 字节。
    """

    try:
        return ROUTER_PROGRAM_IDS[router]
    except KeyError as exc:  # pragma: no cover - 使用方须保证枚举完整
        raise KeyError(f"未注册的 router: {router}") from exc


def _slice_pool_bytes(pool_state: bytes, offset: int, length: int, label: str) -> bytes:
    end = offset + length
    if end > len(pool_state):
        raise ValueError(f"{label} 超出 pool_state 长度: 0x{offset:x}+{length} > {len(pool_state)}")
    segment = pool_state[offset:end]
    if not any(segment):
        raise ValueError(f"{label} 读取结果全为 0，疑似缺失")
    return segment


def _build_jupiter_swap_seeds(pool_state: bytes, router: RouterProgram) -> list[bytes]:
    seeds: list[bytes] = [
        MARKET_LITERAL_PREFIX32,
        MARKET_LITERAL_SUFFIX,
        VAULT_LITERAL,
        get_router_program_id(router),
    ]
    for idx, offset in enumerate(JUPITER_ROUTER_SEED_OFFSETS):
        seeds.append(_slice_pool_bytes(pool_state, offset, 32, f"router_seed[{idx}]"))
    return seeds


def build_swap_authority_seeds(pool_state: bytes, router: RouterProgram) -> list[bytes]:
    """
    根据 router 分支从 `pool_state` 中提取 swap authority seeds。

    目前仅支持 Jupiter v6，Step/T1TAN 尚在解析。
    """

    if router is RouterProgram.JUPITER_V6:
        return _build_jupiter_swap_seeds(pool_state, router)
    raise NotImplementedError(f"{router.value} seeds 构造尚未实现")


def read_pool_signer_bump(pool_state: bytes) -> int:
    if POOL_SIGNER_BUMP_OFFSET >= len(pool_state):
        raise ValueError("pool_state 数据长度不足 0x330 字节")
    return pool_state[POOL_SIGNER_BUMP_OFFSET]


def derive_swap_authority_address(
    pool_pubkey: Union[str, bytes],
    pool_state: bytes,
    router: RouterProgram,
) -> tuple[str, int, list[bytes]]:
    """
    根据池子原始数据推导 swap authority 地址。

    返回值：(address, bump, seeds)，其中 seeds 为最终用于 PDA 的字节序列。
    会优先尝试 `seeds + pool_pubkey` 与程序 ID 拼接的组合，若 bump 对不上，
    再尝试不附加 pool_pubkey 的方案，便于定位实际实现。
    """

    seeds = build_swap_authority_seeds(pool_state, router)
    bump = read_pool_signer_bump(pool_state)
    if isinstance(pool_pubkey, str):
        pool_bytes = b58decode(pool_pubkey)
    else:
        pool_bytes = pool_pubkey

    addr_with_pool, bump_with_pool = find_program_address(
        tuple(seeds) + (pool_bytes,),
        PROGRAM_ID,
    )
    if bump_with_pool == bump:
        return addr_with_pool, bump_with_pool, [*seeds, pool_bytes]

    addr_without_pool, bump_without_pool = find_program_address(seeds, PROGRAM_ID)
    if bump_without_pool == bump:
        return addr_without_pool, bump_without_pool, list(seeds)

    raise ValueError(
        "推导 swap authority 失败：计算得到的 bump 与池子内保存值不一致 "
        f"(with_pool={bump_with_pool}, without_pool={bump_without_pool}, stored={bump})"
    )


def extract_route_discriminant(account_data: bytes) -> int:
    """
    解析 aggregator CPI 账户数据中 `account[0x24..0x2c]` 的 discriminant。

    在汇编中，该值用于区分 Jupiter CP swap 的具体市场/路由分支。
    """

    if len(account_data) < 0x2C:
        raise ValueError("account data 长度不足 0x2c 字节")
    return int.from_bytes(account_data[0x24:0x2C], "little")


def _decode_bitfield(value: int) -> BitfieldMatch:
    """
    逐条翻译 function_9880 的二进制逻辑。

    该函数并不会修改输入数组，仅根据 `value` 在 `TABLE_172F0` / `TABLE_17378`
    中定位所属区间，并返回 parity 等元信息，供上层汇编判断是否合法。
    """

    TABLE = TABLE_172F0.entries
    BIT_MASK = (1 << 21) - 1

    # === Binary search upper bound（与汇编保持 11-bit 左移对齐） ===
    low = 0
    span = 34
    cmp_value = (value << 11) & 0xFFFFFFFF
    while True:
        step = span >> 1
        mid = low + step

        entry = TABLE[mid] & BIT_MASK
        entry_cmp = (entry << 11) & 0xFFFFFFFF
        if entry_cmp <= cmp_value:
            low = mid

        span -= step
        if span <= 1:
            break

    base_entry = TABLE[low] & BIT_MASK
    base_cmp = (base_entry << 11) & 0xFFFFFFFF

    idx = low
    add_one = 1 if base_cmp < cmp_value else 0
    equal_one = 1 if base_cmp == cmp_value else 0
    idx += add_one + equal_one
    if idx > 33:
        raise ValueError(f"bitfield index overflow: value={value}, idx={idx}")

    # === 取出阈值与 bit offset ===
    prev_threshold = TABLE[idx - 1] & BIT_MASK if idx > 0 else 0
    next_threshold = TABLE[idx] & BIT_MASK
    bit_offset = TABLE[idx] >> 21
    bit_limit = 751 if idx == 33 else TABLE[idx + 1] >> 21

    # === 复制剩余循环用于复现 parity 计算 ===
    diff = bit_limit - bit_offset - 1
    if diff <= 0:
        parity = bit_offset & 1
        return BitfieldMatch(
            slot_index=idx,
            prev_threshold=prev_threshold,
            next_threshold=next_threshold,
            bit_offset=bit_offset,
            bit_limit=bit_limit,
            parity=parity,
        )

    remaining = (value - prev_threshold) & 0xFFFFFFFF
    weights = TABLE_17378
    acc = 0
    for delta in range(diff):
        pos = bit_offset + delta
        if pos >= len(weights):
            raise ValueError(f"weights overflow at pos={pos}")
        acc += weights[pos]
        if acc > remaining:
            bit_offset += delta
            break
    else:
        bit_offset += diff

    parity = bit_offset & 1
    return BitfieldMatch(
        slot_index=idx,
        prev_threshold=prev_threshold,
        next_threshold=next_threshold,
        bit_offset=bit_offset,
        bit_limit=bit_limit,
        parity=parity,
    )


# === function_9362: 范围校验 + 下游派发 (WIP) ===
def _dispatch_seed_builder(opcode: int) -> Iterable[bytes]:
    """
    Lifts function_9362 + function_9274 调度流程。
    """
    if opcode < 32:
        return ()
    if opcode < 127:
        # 直接把 opcode 视作 ASCII
        yield bytes([opcode])
        return
    lo = opcode & 0xFF
    if opcode < 0x10000:
        hi = (opcode >> 8) & 0xFF
        bucket = FAST_OPCODE_BYTES.get(hi)
        if bucket and lo in bucket:
            symbol_index = FAST_OPCODE_INDEX[(hi, lo)]
            literal = SPECIAL_SYMBOL_LITERALS.get(symbol_index)
            if literal is not None:
                yield literal
                return
            if 32 <= symbol_index <= 126:
                yield bytes([symbol_index])
                return
            if symbol_index > 767:
                bitfield = _decode_bitfield(symbol_index)
                if bitfield.parity:
                    yield _encode_hex_u64(symbol_index)
                    return
            yield SeedToken(_symbol_tier(symbol_index), symbol_index)
            return
        if _value_in_intervals(opcode, FALLBACK_SHORT_INTERVALS):
            raise NotImplementedError(
                f"short opcode 0x{opcode:04x} 命中差分表但尚未解析具体 seed"
            )
        raise ValueError(f"opcode 0x{opcode:04x} not accepted in fast table")
    if opcode < 0x20000:
        hi = (opcode >> 8) & 0xFF
        bucket = DENSE_OPCODE_BYTES.get(hi)
        if bucket and lo in bucket:
            symbol_index = DENSE_OPCODE_INDEX[(hi, lo)]
            literal = SPECIAL_SYMBOL_LITERALS.get(symbol_index)
            if literal is not None:
                yield literal
                return
            if 32 <= symbol_index <= 126:
                yield bytes([symbol_index])
                return
            if symbol_index > 767:
                bitfield = _decode_bitfield(symbol_index)
                if bitfield.parity:
                    yield _encode_hex_u64(symbol_index)
                    return
            yield SeedToken(_symbol_tier(symbol_index), symbol_index)
            return
        masked = opcode & 0xFFFF
        if _value_in_intervals(masked, FALLBACK_LONG_INTERVALS):
            raise NotImplementedError(
                f"dense opcode 0x{opcode:04x} 命中差分表但尚未解析具体 seed"
            )
        raise ValueError(f"opcode 0x{opcode:04x} not accepted in dense table")
    if opcode < 0x10000:
        raise AssertionError("unreachable")
    # TODO: 参考 function_9362 的 >=0x20000 分支，继续补齐剩余指令集
    raise NotImplementedError("extended opcode path not yet implemented")


def _value_in_intervals(value: int, intervals: Sequence[tuple[int, int]]) -> bool:
    for start, end in intervals:
        if value < start:
            return False
        if value < end:
            return True
    return False


def _symbol_tier(index: int) -> int:
    if index < 128:
        return 0
    if index < 2048:
        return 1
    if index < 65536:
        return 2
    return 3


def derive_pool_signer(pool_account: bytes, seeds: Sequence[bytes]) -> tuple[str, int]:
    """
    通过显式 seeds 计算 PDA。
    """
    addr, bump = find_program_address(seeds + (pool_account,), PROGRAM_ID)
    return addr, bump


__all__ = [
    "_encode_hex_u64",
    "_decode_bitfield",
    "_dispatch_seed_builder",
    "build_swap_authority_seeds",
    "read_pool_signer_bump",
    "derive_swap_authority_address",
    "derive_pool_signer",
    "load_jupiter_static_accounts",
    "AccountMetaView",
    "SeedToken",
    "RouterProgram",
    "identify_router",
    "get_router_program_id",
    "extract_route_discriminant",
]
