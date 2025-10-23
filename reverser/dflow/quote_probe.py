"""
Quick helper to reproduce a DFlow /quote request带上前端 headers。

默认参数即可跑通：
    python -m reverser.dflow.quote_probe

若需覆盖输入、输出或代理，再通过命令行覆盖对应选项即可。
"""

from __future__ import annotations

import argparse
import hashlib
import sys
import time
import uuid
from typing import Dict, Optional

import requests
from urllib.parse import urlencode

BASE_URL = "https://aggregator-api-proxy.dflow.workers.dev"
DEFAULT_INPUT_MINT = "So11111111111111111111111111111111111111112"
DEFAULT_OUTPUT_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
DEFAULT_AMOUNT = "1000000000"
DEFAULT_SLIPPAGE_BPS = "auto"
DEFAULT_PROXY = "http://192.168.124.4:9999"

STATIC_HEADERS: Dict[str, str] = {
    "host": "aggregator-api-proxy.dflow.workers.dev",
    "accept": "*/*",
    "accept-language": "zh-CN,zh;q=0.9",
    "origin": "https://dflow.net",
    "referer": "https://dflow.net/",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "cross-site",
    "sec-ch-ua": '"Google Chrome";v="141", "Not?A_Brand";v="8", "Chromium";v="141"',
    "sec-ch-ua-platform": '"Windows"',
    "sec-ch-ua-mobile": "?0",
    "priority": "u=1, i",
    "user-agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36"
    ),
}


def make_x_client_headers(path_with_query: str, body: str, little_endian: bool) -> Dict[str, str]:
    timestamp_ms = int(time.time() * 1000)
    payload = f"{path_with_query}5_{body}k".encode("utf-8")
    byteorder = "little" if little_endian else "big"
    ts_bytes = timestamp_ms.to_bytes(8, byteorder=byteorder, signed=False)
    digest = hashlib.sha256(payload + ts_bytes).digest()[:15].hex()

    rnd = uuid.uuid4().hex
    request_id = "-".join(
        [
            digest[0:8],
            digest[8:12],
            f"{rnd[14]}{digest[12:15]}",
            f"{rnd[19]}{digest[15:18]}",
            digest[18:30],
        ]
    ).lower()

    return {
        "x-client-timestamp": str(timestamp_ms),
        "x-client-request-id": request_id,
    }


def request_quote(
    input_mint: str,
    output_mint: str,
    amount: str,
    slippage_bps: str,
    proxy: Optional[str],
    verify: bool,
    little_endian: bool,
) -> None:
    params = {
        "inputMint": input_mint,
        "outputMint": output_mint,
        "amount": amount,
        "slippageBps": slippage_bps,
        "dexes":"HumidiFi,SolFi,SolFi V2,Tessera V,ZeroFi,Whirlpools,Obric V2,Aquifer,Lifinity V2,DFlow JIT Router"
    }
    query = urlencode(params)
    path_with_query = f"/quote?{query}"
    headers = {**STATIC_HEADERS, **make_x_client_headers(path_with_query, "", little_endian)}

    proxies = None
    if proxy:
        proxies = {"http": proxy, "https": proxy}
    if not verify:
        requests.packages.urllib3.disable_warnings()  # type: ignore[attr-defined]

    response = requests.get(
        f"{BASE_URL}/quote",
        params=params,
        headers=headers,
        proxies=proxies,
        verify=verify,
        timeout=10,
    )
    print("Status:", response.status_code)
    print("Headers:", response.headers)
    print("Body:", response.text)


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Send a DFlow /quote request with browser headers.")
    parser.add_argument(
        "--input",
        default=DEFAULT_INPUT_MINT,
        help=f"Input mint pubkey (default: {DEFAULT_INPUT_MINT}).",
    )
    parser.add_argument(
        "--output",
        default=DEFAULT_OUTPUT_MINT,
        help=f"Output mint pubkey (default: {DEFAULT_OUTPUT_MINT}).",
    )
    parser.add_argument(
        "--amount",
        default=DEFAULT_AMOUNT,
        help=f"Raw amount (u64 string, default: {DEFAULT_AMOUNT}).",
    )
    parser.add_argument(
        "--slippage-bps",
        default=DEFAULT_SLIPPAGE_BPS,
        help=f"Slippage (default: {DEFAULT_SLIPPAGE_BPS}).",
    )
    parser.add_argument(
        "--proxy",
        default=DEFAULT_PROXY,
        help=f"HTTP(S) proxy，例如 http://host:port（默认: {DEFAULT_PROXY}）。",
    )
    parser.add_argument(
        "--no-proxy",
        action="store_true",
        help="禁用代理，直接访问目标地址。",
    )
    parser.add_argument(
        "--strict-tls",
        action="store_true",
        help="启用 TLS 证书校验（默认关闭，便于通过公司代理调试）。",
    )
    parser.add_argument(
        "--big-endian",
        action="store_true",
        help="Encode timestamp bytes in big-endian (debug only; real frontend使用little endian).",
    )
    return parser.parse_args()


def main() -> None:
    args = _parse_args()
    try:
        verify = args.strict_tls
        little_endian = not args.big_endian
        proxy = None if args.no_proxy else args.proxy
        request_quote(
            args.input,
            args.output,
            args.amount,
            args.slippage_bps,
            proxy,
            verify,
            little_endian,
        )
    except requests.RequestException as exc:
        print(f"Request failed: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
