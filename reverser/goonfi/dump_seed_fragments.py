"""
Quick-and-dirty inspector for the seed formatting tables embedded in `goonfi.so`.

This script is not a full reimplementation of `function_9031`.  Its purpose is to
surface the raw layout of the formatting fragments so that we can cross-check
our guesses about how swap-authority seeds are assembled.

Current behaviour:
    * Read `.rodata` blob starting at 0x16690 (length 856) that contains the
      concatenated literal pool used by `fmt::Arguments`.
    * Enumerate the primary fragment slice located at 0x09a6 (38 entries, each
      16 bytes) and dump it as `u16` tuples.  Each tuple (a0, a1, ..., a7)
      corresponds to the four little-endian `u32` fields inside a fragment.
      Splitting them into `u16` values makes it easier to spot the half-word
      structure (start/end offsets, argument indices, etc.).

The output is intentionally verbose and designed for manual reverse-engineering
rather than for machine consumption.
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterator, Sequence


GOONFI_SO = Path(__file__).with_name("goonfi.so")

# The giant literal that `function_9031` slices at runtime.
LITERAL_BLOB_OFFSET = 0x16690
LITERAL_BLOB_LENGTH = 856

# The primary `Vec<Piece>` slice described in `function_9031`.
# Encoded as 38 fragments, each 16 bytes wide.
FRAGMENT_TABLE_OFFSET = 0x09A6
FRAGMENT_TABLE_COUNT = 0x26  # 38 entries
FRAGMENT_SIZE = 16


def _read_blob(offset: int, length: int) -> bytes:
    blob = GOONFI_SO.read_bytes()
    return blob[offset : offset + length]


def _iter_fragments() -> Iterator[tuple[int, Sequence[int]]]:
    raw = _read_blob(
        FRAGMENT_TABLE_OFFSET, FRAGMENT_TABLE_COUNT * FRAGMENT_SIZE
    )
    for idx in range(FRAGMENT_TABLE_COUNT):
        start = idx * FRAGMENT_SIZE
        chunk = raw[start : start + FRAGMENT_SIZE]
        # Treat the 16-byte chunk as eight little-endian u16 values.  This makes
        # it straightforward to map the half-word patterns we saw in the
        # disassembly (start/end offsets, placeholder flags, etc.).
        fields = tuple(
            int.from_bytes(chunk[pos : pos + 2], "little")
            for pos in range(0, FRAGMENT_SIZE, 2)
        )
        yield idx, fields


def main() -> None:
    literal_blob = _read_blob(LITERAL_BLOB_OFFSET, LITERAL_BLOB_LENGTH)
    print(
        f"literal_blob@0x{LITERAL_BLOB_OFFSET:05x} "
        f"(len={len(literal_blob)}) preview:"
    )
    preview = literal_blob[:120]
    print(preview.decode("ascii", errors="replace"))

    print("\nfmt::rt::Piece table (@0x{0:04x}, count={1})".format(
        FRAGMENT_TABLE_OFFSET, FRAGMENT_TABLE_COUNT
    ))
    header = "{:<3} {:<11} {:<8} {:<8} {:<7} {:<7} {:<10} {:<10} {}"
    print(
        header.format(
            "idx",
            "kind",
            "data_off",
            "data_len",
            "tag",
            "extra",
            "word2",
            "word3",
            "sample",
        )
    )

    raw = _read_blob(
        FRAGMENT_TABLE_OFFSET, FRAGMENT_TABLE_COUNT * FRAGMENT_SIZE
    )
    for idx in range(FRAGMENT_TABLE_COUNT):
        base_offset = idx * FRAGMENT_SIZE
        word0 = int.from_bytes(raw[base_offset : base_offset + 4], "little")
        word1 = int.from_bytes(raw[base_offset + 4 : base_offset + 8], "little")
        word2 = int.from_bytes(raw[base_offset + 8 : base_offset + 12], "little")
        word3 = int.from_bytes(raw[base_offset + 12 : base_offset + 16], "little")
        data_offset = word0 & 0xFFFF
        data_len = (word0 >> 16) & 0xFFFF
        tag = word1 & 0xFFFF
        extra = (word1 >> 16) & 0xFFFF
        sample = ""
        kind = "placeholder"
        if data_len:
            sample = literal_blob[
                data_offset : data_offset + min(data_len, 32)
            ].decode("ascii", errors="replace")
            kind = "literal"
        sample = sample.replace("\n", "\\n")
        print(
            header.format(
                f"{idx:02d}",
                kind,
                f"0x{data_offset:04x}",
                f"{data_len:4d}",
                f"0x{tag:04x}",
                f"0x{extra:04x}",
                f"0x{word2:08x}",
                f"0x{word3:08x}",
                sample,
            )
        )


if __name__ == "__main__":
    main()
