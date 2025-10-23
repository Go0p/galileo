"""Helper to reproduce the frontend x-client headers locally."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
import uuid
from typing import Any, Dict


def make_headers(path: str, body: Dict[str, Any] | str) -> Dict[str, str]:
    """
    Generate x-client headers identical to the browser logic.

    Args:
        path: Request path such as "/auth/token".
        body: Request payload dict or JSON string.
    """
    if isinstance(body, dict):
        body = json.dumps(body, separators=(",", ":"), ensure_ascii=False)

    timestamp = int(time.time() * 1000)
    payload = f"{path}5_{body}k".encode("utf-8")
    ts_bytes = timestamp.to_bytes(8, byteorder="big", signed=False)

    digest = hashlib.sha256(payload + ts_bytes).digest()[:15].hex()

    rnd = uuid.uuid4().hex
    # Insert UUID chars to mirror the obfuscated frontend UUID layout.
    segment = [
        digest[0:8],
        digest[8:12],
        rnd[14] + digest[12:15],
        rnd[19] + digest[15:18],
        digest[18:30],
    ]
    request_id = "-".join(segment).lower()

    return {
        "x-client-timestamp": str(timestamp),
        "x-client-request-id": request_id,
    }


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate x-client headers matching the frontend."
    )
    parser.add_argument("--path", required=True, help="Request path, e.g. /auth/token")
    parser.add_argument(
        "--body",
        required=True,
        help="JSON string or @file.json to load from disk.",
    )
    return parser.parse_args()


def _load_body(body_arg: str) -> Dict[str, Any] | str:
    if body_arg.startswith("@"):
        with open(body_arg[1:], "r", encoding="utf-8") as fh:
            return json.load(fh)
    try:
        return json.loads(body_arg)
    except json.JSONDecodeError:
        return body_arg


def main() -> None:
    args = _parse_args()
    try:
        body = _load_body(args.body)
    except OSError as exc:
        print(f"Failed to load body: {exc}", file=sys.stderr)
        sys.exit(1)
    headers = make_headers(args.path, body)
    json.dump(headers, sys.stdout, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
