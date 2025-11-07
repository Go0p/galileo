#!/usr/bin/env python3
"""
Decode an arbitrary Solana transaction blob (base58 or base64) and print a
human-readable breakdown using the existing DFlow decoder utilities.
"""

from __future__ import annotations

import argparse
import base64
import binascii
import sys
from pathlib import Path

from describe_open_transaction import (
    BASE58_ALPHABET,
    TOKEN_METADATA,
    decode_transaction,
    print_summary,
)

BASE58_MAP = {ch: idx for idx, ch in enumerate(BASE58_ALPHABET)}


def base58_decode(value: str) -> bytes:
    """Decode a Solana-style base58 string (no checksum)."""
    if not value:
        raise ValueError("empty base58 string")

    zero_prefix = len(value) - len(value.lstrip("1"))
    result = 0
    for ch in value:
        try:
            digit = BASE58_MAP[ch]
        except KeyError as exc:
            raise ValueError(f"invalid base58 character: {ch}") from exc
        result = result * 58 + digit

    if result == 0:
        decoded = b""
    else:
        decoded = result.to_bytes((result.bit_length() + 7) // 8, "big")

    return b"\x00" * zero_prefix + decoded


def read_blob_value(arg_value: str | None, file_path: str | None) -> str:
    """Load the transaction blob from an argument, file, or stdin."""
    if file_path:
        path = Path(file_path)
        if not path.exists():
            raise SystemExit(f"input file not found: {path}")
        data = path.read_text(encoding="utf-8")
    elif arg_value and arg_value != "-":
        data = arg_value
    else:
        data = sys.stdin.read()

    cleaned = "".join(data.split())
    if not cleaned:
        raise SystemExit("no transaction blob provided")
    return cleaned


def decode_blob(blob: str) -> tuple[bytes, str]:
    """Attempt base58 first (field name hints at bs58), then fall back to base64."""
    try:
        return base58_decode(blob), "base58"
    except ValueError:
        pass

    try:
        return base64.b64decode(blob, validate=False), "base64"
    except binascii.Error as exc:
        raise SystemExit("failed to decode blob as base58 or base64") from exc


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Decode a base58/base64 Solana transaction blob and print its contents."
    )
    parser.add_argument(
        "transaction_blob",
        nargs="?",
        help="Transaction data (set to '-' to read from stdin).",
    )
    parser.add_argument(
        "--file",
        help="Path to a file containing the transaction blob.",
    )
    parser.add_argument(
        "--input-mint",
        default="",
        help="Optional mint address for the input leg (improves amount decoding).",
    )
    parser.add_argument(
        "--output-mint",
        default="",
        help="Optional mint address for the output leg.",
    )
    args = parser.parse_args()

    blob = read_blob_value(args.transaction_blob, args.file)
    raw_tx, encoding = decode_blob(blob)

    (
        account_metas,
        blockhash,
        instructions,
        signatures,
        context,
    ) = decode_transaction(raw_tx)
    mint_info = {"input_mint": args.input_mint, "output_mint": args.output_mint}

    print(f"Decoded using {encoding}")
    print_summary(account_metas, blockhash, instructions, signatures, mint_info, context)

    # Surface the mint metadata if provided for quick inspection.
    input_mint = mint_info["input_mint"]
    output_mint = mint_info["output_mint"]
    if input_mint in TOKEN_METADATA or output_mint in TOKEN_METADATA:
        print("\n=== Provided Amount Metadata ===")
        if input_mint in TOKEN_METADATA:
            symbol, _, label = TOKEN_METADATA[input_mint]
            extra = f" ({label})" if label else ""
            print(f"  input_mint: {input_mint} [{symbol}]{extra}")
        if output_mint in TOKEN_METADATA:
            symbol, _, label = TOKEN_METADATA[output_mint]
            extra = f" ({label})" if label else ""
            print(f"  output_mint: {output_mint} [{symbol}]{extra}")


if __name__ == "__main__":
    main()
