#!/usr/bin/env python3
"""
重建 Jupiter 指令的账户顺序，并根据已知市场信息拆解出使用的 DEX swap。

输出示例：
  python3 analyze/reconstruct_jupiter_routes.py --limit 5 --pretty

会在控制台打印首 5 笔交易的路由拆解，同时把完整结果写到
  analyze/jupiter_routes/<signature>.json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

JUPITER_PROGRAM = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"


@dataclass(frozen=True)
class MarketInfo:
    pubkey: str
    owner: str
    account_len: Optional[int]
    account_metas: Optional[int]
    routing_group: Optional[int]


@dataclass
class DexOwnerInfo:
    name: str
    markets: Dict[str, MarketInfo]
    account_lengths: Counter


@dataclass
class DexSegment:
    owner: str
    dex_name: Optional[str]
    accounts: List[str]
    market: Optional[str]
    account_count: int
    expected_lengths: List[int]
    status: str


@dataclass
class JupiterInstruction:
    accounts: List[str]
    segments: List[DexSegment]


@dataclass
class TransactionRoute:
    signature: str
    instructions: List[JupiterInstruction]


def load_dex_mapping(path: Path) -> Dict[str, str]:
    mapping = json.loads(path.read_text(encoding="utf-8"))
    return {program.upper(): name for program, name in mapping.items()}


def load_markets(path: Path, dex_names: Dict[str, str]) -> Dict[str, DexOwnerInfo]:
    data = json.loads(path.read_text(encoding="utf-8"))
    owners: Dict[str, DexOwnerInfo] = {}

    for entry in data:
        owner = entry.get("owner")
        if not owner:
            continue
        owner_upper = owner.upper()

        params = entry.get("params") or {}
        swap_size = params.get("swapAccountSize") or {}
        account_len = swap_size.get("account_len")
        account_metas = swap_size.get("account_metas_count")

        market = MarketInfo(
            pubkey=entry.get("pubkey"),
            owner=owner,
            account_len=int(account_len) if account_len else None,
            account_metas=int(account_metas) if account_metas else None,
            routing_group=params.get("routingGroup"),
        )

        info = owners.setdefault(
            owner,
            DexOwnerInfo(
                name=dex_names.get(owner_upper),
                markets={},
                account_lengths=Counter(),
            ),
        )
        if market.account_len:
            info.account_lengths.update([market.account_len])
        if market.pubkey:
            info.markets[market.pubkey] = market

    return owners


def resolve_tx_dir(script_dir: Path, user_path: Optional[str]) -> Path:
    if user_path:
        return Path(user_path)
    candidates = [
        script_dir / "txs",
        script_dir / "analyze" / "txs",
        script_dir.parent / "analyze" / "txs",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def resolve_file(path: Optional[str], candidates: Sequence[Path]) -> Path:
    if path:
        return Path(path)
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[-1]


def build_account_keys(tx: dict) -> List[str]:
    message = tx["transaction"]["message"]
    account_keys = list(message.get("accountKeys") or [])

    loaded = tx.get("meta", {}).get("loadedAddresses") or {}
    account_keys.extend(loaded.get("writable", []))
    account_keys.extend(loaded.get("readonly", []))
    return account_keys


def reconstruct_jupiter_instructions(
    tx: dict,
    dex_infos: Dict[str, DexOwnerInfo],
) -> List[JupiterInstruction]:
    message = tx["transaction"]["message"]
    account_keys = build_account_keys(tx)
    instructions = []

    for compiled in message.get("instructions") or []:
        program_index = compiled.get("programIdIndex")
        if program_index is None:
            continue
        try:
            program_id = account_keys[program_index]
        except IndexError:
            continue
        if program_id != JUPITER_PROGRAM:
            continue

        indices = compiled.get("accounts") or []
        accounts: List[str] = []
        for idx in indices:
            if not isinstance(idx, int):
                continue
            try:
                accounts.append(account_keys[idx])
            except IndexError:
                accounts.append(f"<index_out_of_range:{idx}>")

        segments = slice_segments(accounts, dex_infos)
        instructions.append(JupiterInstruction(accounts=accounts, segments=segments))

    return instructions


def slice_segments(accounts: List[str], dex_infos: Dict[str, DexOwnerInfo]) -> List[DexSegment]:
    owners = set(dex_infos.keys())
    segments: List[DexSegment] = []
    i = 0
    length = len(accounts)
    seen_owner = False

    while i < length:
        account = accounts[i]

        if account == JUPITER_PROGRAM and seen_owner:
            i += 1
            continue

        owner = account
        if owner not in owners:
            i += 1
            continue
        seen_owner = True

        j = i + 1
        while j < length and accounts[j] not in owners:
            if accounts[j] == JUPITER_PROGRAM and seen_owner:
                j += 1
                continue
            j += 1

        chunk = accounts[i:j]
        info = dex_infos.get(owner)
        expected_lengths = sorted(info.account_lengths) if info else []
        market = None
        if info:
            for account in chunk[1:]:
                if account in info.markets:
                    market = account
                    break

        segment = DexSegment(
            owner=owner,
            dex_name=info.name if info else None,
            accounts=chunk,
            market=market,
            account_count=len(chunk),
            expected_lengths=expected_lengths,
            status=classify_length(len(chunk), expected_lengths),
        )
        segments.append(segment)
        i = j

    return segments


def classify_length(actual: int, expected: List[int]) -> str:
    if not expected:
        return "unknown"
    if actual in expected:
        return "match"
    if actual < min(expected):
        return "short"
    if actual > max(expected):
        return "long"
    return "mismatch"


def dump_transaction_route(
    route: TransactionRoute,
    output_dir: Path,
    pretty: bool,
) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / f"{route.signature}.json"
    payload = {
        "signature": route.signature,
        "instructions": [
            {
                "accounts": instr.accounts,
                "segments": [
                    {
                        "owner": seg.owner,
                        "dex_name": seg.dex_name,
                        "market": seg.market,
                        "account_count": seg.account_count,
                        "expected_lengths": seg.expected_lengths,
                        "status": seg.status,
                        "accounts": seg.accounts,
                    }
                    for seg in instr.segments
                ],
            }
            for instr in route.instructions
        ],
    }
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2 if pretty else None),
        encoding="utf-8",
    )


def print_route(route: TransactionRoute, limit_segments: Optional[int] = None) -> None:
    print(f"签名: {route.signature}")
    for idx, instr in enumerate(route.instructions, start=1):
        print(f"  Jupiter 指令 #{idx}，账户总数 {len(instr.accounts)}")
        segments = instr.segments[:limit_segments] if limit_segments else instr.segments
        for seg_idx, seg in enumerate(segments, start=1):
            dex_label = seg.dex_name or "未知 DEX"
            print(
                f"    段 #{seg_idx}: {dex_label} "
                f"(owner={seg.owner}, accounts={seg.account_count}, "
                f"期望={seg.expected_lengths or '未知'}, 状态={seg.status})"
            )
            if seg.market:
                print(f"      命中池子: {seg.market}")
            else:
                print("      命中池子: 未匹配")
        if limit_segments and len(instr.segments) > limit_segments:
            print(f"    ... 其余 {len(instr.segments) - limit_segments} 段已截断")
    print()


def parse_args(script_dir: Path) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="重建 Jupiter 指令使用的 DEX 账户顺序")
    parser.add_argument(
        "--tx-dir",
        default=None,
        help="交易 JSON 目录（默认: 自动匹配 analyze/txs）",
    )
    parser.add_argument(
        "--dexes",
        default=None,
        help="DEX 映射文件（默认: analyze/dexes.json）",
    )
    parser.add_argument(
        "--markets",
        default=None,
        help="markets 列表（默认: analyze/markets_filtered.json）",
    )
    parser.add_argument(
        "--output-dir",
        default=None,
        help="输出目录（默认: analyze/jupiter_routes）",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="仅打印前 N 笔交易，0 表示全部",
    )
    parser.add_argument(
        "--signature",
        action="append",
        help="指定要解析的交易签名（可重复传入多次）；默认处理目录下全部交易",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="以缩进格式写出 JSON",
    )
    parser.add_argument(
        "--segments",
        type=int,
        default=0,
        help="打印时每个指令最多展示的段数，0 表示全部",
    )
    return parser.parse_args()


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    args = parse_args(script_dir)

    tx_dir = resolve_tx_dir(script_dir, args.tx_dir)
    dex_path = resolve_file(
        args.dexes,
        [
            script_dir / "dexes.json",
            script_dir.parent / "analyze" / "dexes.json",
        ],
    )
    markets_path = resolve_file(
        args.markets,
        [
            script_dir / "markets_filtered.json",
            script_dir.parent / "analyze" / "markets_filtered.json",
        ],
    )
    output_dir = Path(args.output_dir) if args.output_dir else script_dir / "jupiter_routes"

    if not tx_dir.exists():
        print(f"交易目录不存在: {tx_dir}")
        return 1
    if not dex_path.exists():
        print(f"DEX 映射文件不存在: {dex_path}")
        return 1
    if not markets_path.exists():
        print(f"markets 文件不存在: {markets_path}")
        return 1

    print(f"读取交易目录: {tx_dir}")
    print(f"使用 DEX 映射: {dex_path}")
    print(f"使用 markets: {markets_path}")
    print(f"输出目录: {output_dir}")

    dex_names = load_dex_mapping(dex_path)
    dex_infos = load_markets(markets_path, dex_names)
    dex_owner_set = set(dex_infos.keys())
    if not dex_owner_set:
        print("Dex 映射为空，无法继续")
        return 1

    tx_files = sorted(tx_dir.glob("*.json"))
    if not tx_files:
        print(f"{tx_dir} 下未找到交易 JSON")
        return 0

    if args.signature:
        requested = {sig.strip() for sig in args.signature if sig.strip()}
        selected_files = []
        missing = []
        index = {f.stem: f for f in tx_files}
        for sig in requested:
            if sig in index:
                selected_files.append(index[sig])
            else:
                missing.append(sig)
        if missing:
            print("未找到以下签名对应的交易文件：")
            for sig in missing:
                print(f"  - {sig}")
        tx_files = selected_files
        if not tx_files:
            print("没有可解析的交易，退出")
            return 0

    limit = args.limit if args.limit and args.limit > 0 else None
    printed = 0

    for tx_file in tx_files:
        signature = tx_file.stem
        try:
            tx = json.loads(tx_file.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            print(f"跳过无法解析的交易: {tx_file}")
            continue

        instructions = reconstruct_jupiter_instructions(tx, dex_infos)
        if not instructions:
            continue

        route = TransactionRoute(signature=signature, instructions=instructions)
        dump_transaction_route(route, output_dir, pretty=args.pretty)

        if limit is None or printed < limit:
            segment_limit = args.segments if args.segments > 0 else None
            print_route(route, limit_segments=segment_limit)
            printed += 1
        elif limit is not None and printed >= limit:
            continue

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
