"""Utility helpers that match HumidiFi's on-chain swap_id encoding.

The program stores the latest swap id in its config account at offset 0x2b0,
but it XORs the raw value with a fixed mask before persisting it.  During
verification it decodes the stored word, compares it against the incoming
`swap_id`, and fails if the new value is not strictly greater.

The mask below comes from the RBPF disassembly (see `asm/disassembly.out`
around lines 12908-12937).  These helpers make it easy to reproduce the same
transform locally when crafting instructions or debugging account data.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from typing import Iterable, Tuple

SWAP_ID_MASK = 0x6E9DE2B30B19F9EA
INSTRUCTION_MASK = 0xC3EBBAE2FF2FFF3A


def decode_swap_id(stored_value: int) -> int:
    """Return the plain swap id from the masked value stored on-chain."""
    return stored_value ^ SWAP_ID_MASK


def encode_swap_id(raw_value: int) -> int:
    """Apply HumidiFi's mask so the value matches the on-chain encoding."""
    return raw_value ^ SWAP_ID_MASK


def parse_int(value: str) -> int:
    text = value.strip().lower()
    base = 16 if text.startswith("0x") else 10
    return int(text, base=base)


def decode_instruction_word(payload: str) -> int:
    hex_str = payload.strip().lower().removeprefix("0x")
    data = bytes.fromhex(hex_str)
    if len(data) < 8:
        raise ValueError(f"instruction payload too short: {payload!r}")
    hashed = int.from_bytes(data[:8], "little")
    return hashed ^ INSTRUCTION_MASK


def encode_instruction_word(raw_swap_id: int) -> bytes:
    hashed = raw_swap_id ^ INSTRUCTION_MASK
    return hashed.to_bytes(8, "little", signed=False)


@dataclass
class MaskExample:
    label: str
    masked: int

    @property
    def decoded(self) -> int:
        return decode_swap_id(self.masked)


def iter_examples(values: Iterable[str]) -> Tuple[MaskExample, ...]:
    for value in values:
        value = value.strip()
        if not value:
            continue
        yield MaskExample(label=value, masked=parse_int(value))


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Decode or encode HumidiFi swap_id words."
    )
    parser.add_argument(
        "values",
        nargs="+",
        help="Swap id words (decimal or 0x-prefixed hex). "
        "Use raw ids with --encode or masked ids with --decode. "
        "With --instruction, pass full instruction-data hex blobs.",
    )
    codec_group = parser.add_mutually_exclusive_group()
    codec_group.add_argument(
        "--encode",
        action="store_true",
        help="Treat inputs as raw ids and print masked values.",
    )
    codec_group.add_argument(
        "--decode",
        action="store_true",
        help="Treat inputs as masked words and print raw ids (default).",
    )
    parser.add_argument(
        "--instruction",
        action="store_true",
        help="Treat inputs as HumidiFi instruction-data blobs instead of plain words.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.instruction:
        for value in args.values:
            if args.encode:
                hashed = encode_instruction_word(parse_int(value))
                print(f"{value} -> {hashed.hex()}")
            else:
                swap_id = decode_instruction_word(value)
                print(f"{value} -> {swap_id}")
        return 0

    mode_decode = not args.encode
    for example in iter_examples(args.values):
        if mode_decode:
            print(f"{example.label} -> {decode_swap_id(example.masked)}")
        else:
            print(f"{example.label} -> {encode_swap_id(example.masked)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
