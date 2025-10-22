"""
共享的 base58 / PDA 辅助函数，供解码与 seed 推导逻辑复用。
"""

from __future__ import annotations

from typing import Iterable, Sequence, Tuple

import hashlib


BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
ED25519_P = 2**255 - 19
ED25519_D = (-121665 * pow(121666, -1, ED25519_P)) % ED25519_P


def b58encode(data: bytes) -> str:
    num = int.from_bytes(data, "big")
    if num == 0:
        return "1" * len(data)
    encoded = ""
    while num > 0:
        num, rem = divmod(num, 58)
        encoded = BASE58_ALPHABET[rem] + encoded
    leading_zero = 0
    for byte in data:
        if byte == 0:
            leading_zero += 1
        else:
            break
    return "1" * leading_zero + encoded


def b58decode(data: str) -> bytes:
    num = 0
    for char in data:
        num = num * 58 + BASE58_ALPHABET.index(char)
    raw = num.to_bytes(32, "big")
    pad = 0
    for char in data:
        if char == "1":
            pad += 1
        else:
            break
    return b"\x00" * pad + raw[len(raw) - 32 :]


def is_on_curve(pubkey: bytes) -> bool:
    if len(pubkey) != 32:
        return False
    y = int.from_bytes(pubkey, "little") & ((1 << 255) - 1)
    sign = pubkey[31] >> 7
    if y >= ED25519_P:
        return False
    y2 = (y * y) % ED25519_P
    u = (y2 - 1) % ED25519_P
    v = (ED25519_D * y2 + 1) % ED25519_P
    if v == 0:
        return False
    x2 = (u * pow(v, ED25519_P - 2, ED25519_P)) % ED25519_P
    x = pow(x2, (ED25519_P + 3) // 8, ED25519_P)
    if (x * x - x2) % ED25519_P != 0:
        x = (x * pow(2, (ED25519_P - 1) // 4, ED25519_P)) % ED25519_P
        if (x * x - x2) % ED25519_P != 0:
            return False
    if (x % 2) != sign:
        x = (-x) % ED25519_P
    return not (x == 0 and sign == 1)


def create_program_address(
    seeds: Iterable[bytes],
    program_id: bytes,
) -> bytes:
    hasher = hashlib.sha256()
    for seed in seeds:
        if len(seed) > 32:
            raise ValueError("seed 长度超过 32 字节")
        hasher.update(seed)
    hasher.update(program_id)
    hasher.update(b"ProgramDerivedAddress")
    digest = hasher.digest()
    if is_on_curve(digest):
        raise ValueError("PDA 落在曲线上")
    return digest


def find_program_address(
    seeds: Sequence[bytes],
    program_id: bytes,
) -> Tuple[str, int]:
    seeds_tuple = tuple(seeds)
    for bump in range(255, -1, -1):
        try:
            addr = create_program_address(
                seeds_tuple + (bytes([bump]),),
                program_id,
            )
            return b58encode(addr), bump
        except ValueError:
            continue
    raise RuntimeError("无法找到合法 PDA")


__all__ = [
    "BASE58_ALPHABET",
    "b58encode",
    "b58decode",
    "is_on_curve",
    "create_program_address",
    "find_program_address",
]
