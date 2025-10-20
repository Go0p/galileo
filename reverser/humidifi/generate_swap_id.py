"""Fetch HumidiFi config account and emit the next swap_id."""

from __future__ import annotations

import argparse
import base64
import json
import urllib.request

SWAP_ID_MASK = 0x6E9DE2B30B19F9EA
INSTRUCTION_MASK = 0xC3EBBAE2FF2FFF3A
CONFIG_SWAP_ID_OFFSET = 0x2B0


def fetch_account_data(address: str, rpc_url: str) -> bytes:
    payload = json.dumps(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [address, {"encoding": "base64"}],
        }
    ).encode("utf-8")
    req = urllib.request.Request(
        rpc_url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        body = resp.read()
    doc = json.loads(body)
    error = doc.get("error")
    if error:
        raise RuntimeError(f"RPC error: {error}")
    value = doc.get("result", {}).get("value")
    if value is None:
        raise RuntimeError("RPC returned no account data")
    data_field = value.get("data")
    if not data_field or not isinstance(data_field, list):
        raise RuntimeError("unexpected account data shape")
    return base64.b64decode(data_field[0])


def decode_last_swap_id(data: bytes) -> int:
    if len(data) < CONFIG_SWAP_ID_OFFSET + 8:
        raise RuntimeError("account data shorter than expected")
    masked = int.from_bytes(
        data[CONFIG_SWAP_ID_OFFSET : CONFIG_SWAP_ID_OFFSET + 8], "little"
    )
    return masked ^ SWAP_ID_MASK


def encode_instruction_prefix(raw_swap_id: int) -> str:
    encoded = raw_swap_id ^ INSTRUCTION_MASK
    return encoded.to_bytes(8, "little").hex()


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate next HumidiFi swap_id.")
    parser.add_argument("account", help="HumidiFi config account address.")
    parser.add_argument(
        "--rpc",
        default="https://rpc.shyft.to?api_key=FgU1AVt-7pIiUk2j",
        help="Solana RPC endpoint (default: mainnet-beta).",
    )
    args = parser.parse_args()

    data = fetch_account_data(args.account, args.rpc)
    last_swap = decode_last_swap_id(data)
    next_swap = last_swap + 1

    print(f"last_swap_id = {last_swap}")
    print(f"next_swap_id = {next_swap}")
    print(f"instruction_prefix = {encode_instruction_prefix(next_swap)}")
    print(f"jupiter_style_swap_id = {next_swap}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
