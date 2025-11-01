#!/usr/bin/env python3
"""
拉取指定钱包的交易并写入 analyze/txs 目录。
支持多个 RPC 端点轮询、HTTP 代理与交易详情并行抓取。
"""

from __future__ import annotations

import concurrent.futures
import argparse
import json
import sys
import threading
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional
from urllib import error as urlerror
from urllib import request
from urllib.request import OpenerDirector


DEFAULT_RPCS = [
    "https://mainnet.helius-rpc.com/?api-key=767f42d9-06c2-46f8-8031-9869035d6ce4",
    "https://pump-fe.helius-rpc.com/?api-key=1b8db865-a5a1-4535-9aec-01061440523b",
]
PUMP_FE_ORIGIN = "https://swap.pump.fun"
DEFAULT_PROXY = "http://192.168.124.4:2080"


class RpcRoundRobin:
    def __init__(self, endpoints: List[str]) -> None:
        if not endpoints:
            raise ValueError("至少需要一个 RPC 端点")
        self._endpoints = endpoints
        self._index = 0
        self._lock = threading.Lock()

    def __len__(self) -> int:
        return len(self._endpoints)

    def next(self) -> str:
        with self._lock:
            endpoint = self._endpoints[self._index]
            self._index = (self._index + 1) % len(self._endpoints)
            return endpoint


def rpc_request(
    rotator: RpcRoundRobin,
    payload: Dict[str, Any],
    opener_factory: Callable[[], OpenerDirector],
    *,
    timeout: float = 20.0,
) -> Dict[str, Any]:
    errors: List[str] = []
    for _ in range(len(rotator)):
        endpoint = rotator.next()
        headers = {"Content-Type": "application/json"}
        if "pump-fe" in endpoint:
            headers["Origin"] = PUMP_FE_ORIGIN

        data = json.dumps(payload).encode("utf-8")
        req = request.Request(endpoint, data=data, headers=headers, method="POST")
        opener = opener_factory()

        try:
            with opener.open(req, timeout=timeout) as resp:
                raw = resp.read()
        except urlerror.HTTPError as exc:
            errors.append(f"{endpoint}: HTTP {exc.code}")
            continue
        except urlerror.URLError as exc:
            errors.append(f"{endpoint}: {exc.reason}")
            continue

        try:
            decoded = json.loads(raw)
        except json.JSONDecodeError:
            errors.append(f"{endpoint}: 无法解析 JSON 响应")
            continue

        if "error" in decoded:
            message = decoded["error"].get("message", "未知错误")
            errors.append(f"{endpoint}: {message}")
            continue

        return decoded

    joined = "; ".join(errors)
    raise RuntimeError(f"RPC 请求失败: {joined}")


def fetch_signatures(
    rotator: RpcRoundRobin,
    wallet: str,
    total: int,
    opener_factory: Callable[[], OpenerDirector],
) -> List[str]:
    signatures: List[str] = []
    before: Optional[str] = None

    while len(signatures) < total:
        remaining = total - len(signatures)
        limit = min(1000, remaining)

        params: List[Any] = [wallet, {"limit": limit}]
        if before:
            params[1]["before"] = before

        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": params,
        }
        response = rpc_request(rotator, payload, opener_factory=opener_factory)

        batch = response.get("result", [])
        if not batch:
            break

        for entry in batch:
            sig = entry.get("signature")
            if sig:
                signatures.append(sig)

        before = batch[-1].get("signature")
        if not before:
            break

    return signatures[:total]


def fetch_transaction(
    rotator: RpcRoundRobin,
    signature: str,
    opener_factory: Callable[[], OpenerDirector],
) -> Dict[str, Any]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            signature,
            {
                "encoding": "json",
                "maxSupportedTransactionVersion": 0,
                "commitment": "confirmed",
            },
        ],
    }
    response = rpc_request(rotator, payload, opener_factory=opener_factory)
    return response.get("result", {})


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="下载指定钱包的交易 JSON")
    parser.add_argument(
        "--wallet",
        required=True,
        help="顶级套利者的钱包地址",
    )
    parser.add_argument(
        "--count",
        type=int,
        default=10,
        help="拉取交易的数量（默认: 10）",
    )
    parser.add_argument(
        "--rpc",
        nargs="+",
        default=DEFAULT_RPCS,
        help="RPC 端点列表，将轮询使用",
    )
    parser.add_argument(
        "--out-dir",
        default="analyze/txs",
        help="输出目录（默认: analyze/txs）",
    )
    parser.add_argument(
        "--proxy",
        default=DEFAULT_PROXY,
        help=f"用于 RPC 请求的 HTTP(S) 代理，默认: {DEFAULT_PROXY}",
    )
    parser.add_argument(
        "--no-proxy",
        action="store_true",
        help="禁用代理并直接访问 RPC 端点",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=8,
        help="并行下载交易详情的线程数（默认: 8）",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.count <= 0:
        print("count 参数需大于 0", file=sys.stderr)
        return 1

    proxy_url: Optional[str] = None
    if not args.no_proxy and isinstance(args.proxy, str):
        stripped = args.proxy.strip()
        proxy_url = stripped or None

    if proxy_url:
        proxy_mapping = {"http": proxy_url, "https": proxy_url}
        print(f"通过代理 {proxy_url} 访问 RPC")
    else:
        proxy_mapping = {}
        print("未使用代理访问 RPC")

    def opener_factory() -> OpenerDirector:
        return request.build_opener(request.ProxyHandler(proxy_mapping))

    rotator = RpcRoundRobin(args.rpc)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"开始拉取 {args.wallet} 的 {args.count} 笔交易...")
    try:
        signatures = fetch_signatures(rotator, args.wallet, args.count, opener_factory)
    except Exception as exc:
        print(f"获取交易签名失败: {exc}", file=sys.stderr)
        return 1

    if not signatures:
        print("未获取到任何交易签名")
        return 0

    total = len(signatures)
    max_workers = max(1, min(args.workers, total))
    if max_workers > 1:
        print(f"共获取 {total} 个签名，使用 {max_workers} 个线程并行下载交易详情")
    else:
        print(f"共获取 {total} 个签名，使用单线程下载交易详情")

    failures: List[str] = []
    completed = 0

    def download(sig: str) -> Dict[str, Any]:
        tx = fetch_transaction(rotator, sig, opener_factory)
        if not tx:
            raise RuntimeError("空交易结果")
        return tx

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_sig = {executor.submit(download, sig): sig for sig in signatures}
        for future in concurrent.futures.as_completed(future_to_sig):
            sig = future_to_sig[future]
            try:
                tx = future.result()
            except Exception as exc:
                failures.append(f"{sig}: {exc}")
                continue

            output_path = out_dir / f"{sig}.json"
            output_path.write_text(
                json.dumps(tx, ensure_ascii=False, indent=2),
                encoding="utf-8",
            )
            completed += 1
            print(f"[{completed}/{total}] 写入 {output_path}")

    if failures:
        print("以下交易下载失败：", file=sys.stderr)
        for item in failures:
            print(f"  - {item}", file=sys.stderr)

    print("完成")
    return 0 if not failures else 2


if __name__ == "__main__":
    raise SystemExit(main())
