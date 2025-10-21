"""
Experimental decoder for GoonFi swap-authority seeds.

This module is a direct translation target for the eBPF helpers
`function_9031`, `function_9880`, `function_9449` 与 `function_9362`.
目前仅梳理出 rodata 表和基础工具函数，后续会逐步填入完整的逻辑。
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

from goonfi_decode import b58decode, find_program_address

GOONFI_SO = Path(__file__).with_name("goonfi.so")
PROGRAM_ID = b58decode("goonERTdGsjnkZqWuVjs73BZ3Pb9qoCUdBUL17BnS5j")


def _read_rodata_slice(offset: int, length: int) -> bytes:
    data = GOONFI_SO.read_bytes()
    return data[offset : offset + length]


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


def _decode_bitfield(value: int) -> tuple[int, int]:
    """
    function_9880 translation (WIP):
    Given一个编码后的整数, 返回 (bit_offset, bit_width)。
    """
    # TODO: precise translation; for now emit placeholder.
    raise NotImplementedError("function_9880 translation pending")


# === function_9362: 范围校验 + 下游派发 (WIP) ===
def _dispatch_seed_builder(opcode: int) -> Iterable[bytes]:
    """
    Lifts function_9362 + function_9274 调度流程。
    当前仅保留框架，细节仍待回填。
    """
    if opcode < 32:
        return ()
    if opcode < 127:
        # 直接把 opcode 视作 ASCII
        yield bytes([opcode])
        return
    # TODO: 参考 0x100017162 / 0x100016e0a 表做完整分支
    raise NotImplementedError("complex opcode path not yet implemented")


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
    "derive_pool_signer",
]
