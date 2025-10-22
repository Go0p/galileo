"""
辅助脚本：解析 `goonfi.so` 中静态账户与 AccountMeta 模板。

输出内容：
  1. `RODATA_ADDR_TABLE`（`0x165b0` 起）处的 32 字节公钥列表；
  2. `ACCOUNT_META_TEMPLATE`（`0x1680c` 起）处每 48 字节一组的 AccountMeta 模板，
     包含 `pubkey`、`is_signer`、`is_writable` 标志。

结合 `function_4705 -> 7337 -> 7291 -> 7505` 可快速核对模板是否与推测一致。
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Iterator

from goonfi_utils import b58encode


GOONFI_SO = Path(__file__).with_name("goonfi.so")

RODATA_ADDR_TABLE = 0x165B0
RODATA_ADDR_TABLE_COUNT = 40

# `vault\x04...` 模板区
ACCOUNT_META_TEMPLATE_START = 0x1680C
ACCOUNT_META_TEMPLATE_END = 0x16B50  # 留有余量，脚本会在遍历时防止越界
ACCOUNT_META_SIZE = 48  # 32 pubkey + 1 signer + 1 writable + padding

PUBKEY_LABELS = {
    "updapqBoqhn48uaVxD7oKyFVEwEcHmqbgQa1GvHaUuX": "global_state",
    "Sysvar1nstructions1111111111111111111111111": "sysvar_instructions",
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4": "jupiter_v6_program",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA": "spl_token_program",
    "11111111111111111111111111111111": "system_program",
}


@dataclass
class AccountMetaTemplate:
    index: int
    pubkey: bytes
    is_signer: int
    is_writable: int
    raw_tail: bytes

    def to_dict(self) -> dict[str, object]:
        return {
            "index": self.index,
            "pubkey": b58encode(self.pubkey),
            "is_signer": self.is_signer,
            "is_writable": self.is_writable,
            "tail_hex": self.raw_tail.hex(),
            "tail_words": tuple(
                int.from_bytes(self.raw_tail[i : i + 8], "little")
                for i in range(0, len(self.raw_tail), 8)
                if i + 8 <= len(self.raw_tail)
            ),
            "ascii_hint": "".join(
                chr(b) if 32 <= b <= 126 else "."
                for b in self.pubkey[:16]
            ),
        }


def iter_pubkeys(data: bytes, start: int, count: int) -> Iterator[tuple[int, bytes]]:
    for idx in range(count):
        offset = start + idx * 32
        chunk = data[offset : offset + 32]
        if len(chunk) < 32:
            break
        yield idx, chunk


def iter_account_meta_templates(data: bytes) -> Iterator[AccountMetaTemplate]:
    offset = ACCOUNT_META_TEMPLATE_START
    idx = 0
    end = min(len(data), ACCOUNT_META_TEMPLATE_END)
    while offset + ACCOUNT_META_SIZE <= end:
        chunk = data[offset : offset + ACCOUNT_META_SIZE]
        pubkey = chunk[:32]
        is_signer = chunk[32]
        is_writable = chunk[33]
        tail = chunk[34:]
        yield AccountMetaTemplate(
            index=idx,
            pubkey=pubkey,
            is_signer=is_signer,
            is_writable=is_writable,
            raw_tail=tail,
        )
        idx += 1
        offset += ACCOUNT_META_SIZE


def main() -> None:
    blob = GOONFI_SO.read_bytes()
    print("== Static address table ==")
    for idx, pubkey in iter_pubkeys(blob, RODATA_ADDR_TABLE, RODATA_ADDR_TABLE_COUNT):
        key = b58encode(pubkey)
        label = PUBKEY_LABELS.get(key, "")
        suffix = f"  ({label})" if label else ""
        print(f"{idx:02d}: {key}{suffix}")

    print("\n== AccountMeta templates ==")
    for tmpl in iter_account_meta_templates(blob):
        info = tmpl.to_dict()
        print(
            f"{info['index']:02d}: {info['pubkey']}  signer={info['is_signer']} "
            f"writable={info['is_writable']} tail={info['tail_hex']} tail_words={info['tail_words']} "
            f"ascii_hint={info['ascii_hint']}"
        )


if __name__ == "__main__":
    main()
