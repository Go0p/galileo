#!/usr/bin/env python3
"""
Fetch Solana transaction payloads for the signatures listed in txs.txt.

The script shells out to curl (per workflow guidance) and stores the JSON
response from `getTransaction` under `txs/<signature>.json`.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path


# Prefer local RPC to avoid network egress.
RPC_ENDPOINT = "http://127.0.0.1:8899"
REQUEST_TEMPLATE = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getTransaction",
    "params": [
        None,  # placeholder for signature
        {
            "encoding": "jsonParsed",
            "maxSupportedTransactionVersion": 0,
        },
    ],
}


def fetch(signature: str, out_path: Path) -> None:
    payload = REQUEST_TEMPLATE.copy()
    payload["params"] = payload["params"].copy()
    payload["params"][0] = signature
    data = json.dumps(payload)

    result = subprocess.run(
        [
            "curl",
            "-sS",
            RPC_ENDPOINT,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            data,
        ],
        check=False,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"curl failed for {signature}: {result.stderr.strip() or result.stdout}"
        )

    out_path.write_text(result.stdout)


def main() -> None:
    root = Path(__file__).resolve().parent
    tx_list = root / "txs.txt"
    if not tx_list.exists():
        raise FileNotFoundError(f"missing {tx_list}")

    for raw_line in tx_list.read_text().splitlines():
        signature = raw_line.strip()
        if not signature:
            continue

        out_path = root / f"{signature}.json"
        if out_path.exists():
            continue

        print(f"fetching {signature}")
        fetch(signature, out_path)


if __name__ == "__main__":
    main()
