#!/usr/bin/env python3
"""
统计 Jupiter 套利交易中常用的 DEX 池子，并生成归一化的 swap 账户模板。

示例：
    python3 analyze/summarize_swap_markets.py --top 20 --pretty
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

JUPITER_PROGRAM = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

KNOWN_PROGRAM_LABELS: Dict[str, str] = {
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA": "token_program",
    "11111111111111111111111111111111": "system_program",
    "SysvarC1ock11111111111111111111111111111111": "sysvar_clock",
    "SysvarRent111111111111111111111111111111111": "sysvar_rent",
    "Sysvar1nstructions1111111111111111111111111": "sysvar_instructions",
    "So11111111111111111111111111111111111111112": "native_sol",
    "jitodontfront1111111111111111JustUseJupiter": "jit_authority",
}


class MarketInfo:
    def __init__(
        self,
        pubkey: str,
        owner: str,
        account_len: Optional[int],
        account_metas: Optional[int],
        routing_group: Optional[int],
    ) -> None:
        self.pubkey = pubkey
        self.owner = owner
        self.account_len = account_len
        self.account_metas = account_metas
        self.routing_group = routing_group


class DexOwnerInfo:
    def __init__(self, name: Optional[str]) -> None:
        self.name = name
        self.markets: Dict[str, MarketInfo] = {}
        self.account_lengths: Counter[int] = Counter()


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
        info = owners.setdefault(owner, DexOwnerInfo(dex_names.get(owner_upper)))
        if market.pubkey:
            info.markets[market.pubkey] = market
        if market.account_len:
            info.account_lengths.update([market.account_len])
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


def extract_token_accounts(tx: dict, account_keys: List[str]) -> Dict[str, Dict[str, Optional[str]]]:
    token_info: Dict[str, Dict[str, Optional[str]]] = {}
    meta = tx.get("meta") or {}
    for source in ("preTokenBalances", "postTokenBalances"):
        for balance in meta.get(source, []) or []:
            index = balance.get("accountIndex")
            if index is None:
                continue
            if index >= len(account_keys):
                continue
            account = account_keys[index]
            entry = token_info.setdefault(account, {})
            owner = balance.get("owner")
            mint = balance.get("mint")
            program_id = balance.get("programId")
            if owner and not entry.get("owner"):
                entry["owner"] = owner
            if mint and not entry.get("mint"):
                entry["mint"] = mint
            if program_id and not entry.get("programId"):
                entry["programId"] = program_id
    return token_info


def slice_segments(accounts: List[str], owner_set: set[str]) -> List[Dict[str, List[str]]]:
    segments: List[Dict[str, List[str]]] = []
    length = len(accounts)
    i = 0
    seen_owner = False

    while i < length:
        account = accounts[i]
        if account == JUPITER_PROGRAM and seen_owner:
            i += 1
            continue
        if account not in owner_set:
            i += 1
            continue

        seen_owner = True
        owner = account
        chunk = [owner]
        j = i + 1
        while j < length:
            current = accounts[j]
            if current == JUPITER_PROGRAM and seen_owner:
                j += 1
                continue
            if current in owner_set:
                break
            chunk.append(current)
            j += 1

        segments.append({"owner": owner, "accounts": chunk})
        i = j

    return segments


def identify_pool(chunk: List[str], owner_info: Optional[DexOwnerInfo]) -> Optional[MarketInfo]:
    if not owner_info:
        return None
    for account in chunk[1:]:
        if account in owner_info.markets:
            return owner_info.markets[account]
    return None


def normalize_segment(
    chunk: List[str],
    owner: str,
    pool_info: Optional[MarketInfo],
    token_info: Dict[str, Dict[str, Optional[str]]],
    fee_payer: Optional[str],
) -> Tuple[List[Tuple], List[Dict], List[str]]:
    normalized: List[Tuple] = []
    template: List[Dict] = []
    actuals: List[str] = []
    slot_counter = 0

    for account in chunk:
        if account == JUPITER_PROGRAM:
            continue

        entry: Dict[str, object]
        if account == owner:
            entry = {"role": "dex_program", "account": owner, "label": "dex_program"}
            normalized.append(("dex_program", owner))
            entry["sample_key"] = None
        elif pool_info and account == pool_info.pubkey:
            entry = {"role": "pool", "account": pool_info.pubkey, "label": "pool"}
            normalized.append(("pool", pool_info.pubkey))
            entry["sample_key"] = None
        elif fee_payer and account == fee_payer:
            entry = {"role": "payer", "account": fee_payer, "label": "payer"}
            normalized.append(("payer", fee_payer))
            entry["sample_key"] = None
        elif account in KNOWN_PROGRAM_LABELS:
            label = KNOWN_PROGRAM_LABELS[account]
            entry = {"role": label, "account": account, "label": label}
            normalized.append((label, account))
            entry["sample_key"] = None
        elif account in token_info:
            info = token_info[account]
            owner_addr = info.get("owner")
            mint = info.get("mint")
            program_id = info.get("programId")
            role = "user_ata" if owner_addr == fee_payer else "ata"
            label = f"{role}:{owner_addr}:{mint}:{program_id}"
            entry = {
                "role": role,
                "owner": owner_addr,
                "mint": mint,
                "token_program": program_id,
                "label": label,
            }
            normalized.append((role, owner_addr, mint, program_id))
            entry["sample_key"] = label
        else:
            label = f"slot_{slot_counter}"
            entry = {"role": "slot", "label": label}
            normalized.append(("slot", label))
            entry["sample_key"] = label
            slot_counter += 1

        template.append(entry)
        actuals.append(account)

    return normalized, template, actuals


def compile_swap_items(
    template: List[Dict],
    samples: Dict[str, set[str]],
    include_samples: int = 5,
) -> List[Dict]:
    items: List[Dict] = []
    for entry in template:
        role = entry["role"]
        sample_key = entry.get("sample_key")
        item: Dict[str, object] = {"role": role}

        if role in (
            "dex_program",
            "pool",
            "payer",
            "token_program",
            "system_program",
            "sysvar_clock",
            "sysvar_rent",
            "sysvar_instructions",
            "native_sol",
            "jit_authority",
        ):
            item["account"] = entry.get("account")
        elif role in ("user_ata", "ata"):
            item["owner"] = entry.get("owner")
            item["mint"] = entry.get("mint")
            item["token_program"] = entry.get("token_program")
            addrs = sorted(samples.get(sample_key, []))
            if addrs:
                item["accounts"] = addrs[:include_samples]
                if len(addrs) == 1:
                    item["account"] = addrs[0]
            else:
                item["accounts"] = []
        elif role == "slot":
            label = entry.get("label")
            addrs = sorted(samples.get(sample_key, []))
            if len(addrs) == 1:
                item["account"] = addrs[0]
                item["dynamic"] = False
            else:
                item["account"] = f"<{label}>"
                item["dynamic"] = True
                if addrs:
                    item["samples"] = addrs[:include_samples]
        else:
            # 兜底：保留原始 account 信息
            item["account"] = entry.get("account")

        if "label" in entry and role not in ("slot",):
            item["label"] = entry["label"]

        items.append(item)

    return items


def parse_args(script_dir: Path) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="统计常用 Jupiter DEX 池子")
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
        "--output",
        default=None,
        help="输出文件（默认: analyze/high_freq_markets.json）",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=0,
        help="仅输出前 N 个池子，0 表示全部",
    )
    parser.add_argument(
        "--min-count",
        type=int,
        default=1,
        help="最小出现次数过滤（默认: 1）",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="以缩进格式写出 JSON",
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
    output_path = Path(args.output) if args.output else script_dir / "high_freq_markets.json"

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
    print(f"输出文件: {output_path}")

    dex_names = load_dex_mapping(dex_path)
    dex_infos = load_markets(markets_path, dex_names)
    owner_set = set(dex_infos.keys())

    tx_files = sorted(tx_dir.glob("*.json"))
    if not tx_files:
        print(f"{tx_dir} 下未找到交易 JSON")
        return 0

    pool_stats: Dict[str, Dict] = {}
    total_segments = 0

    for tx_file in tx_files:
        try:
            tx = json.loads(tx_file.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            print(f"跳过无法解析的交易: {tx_file}")
            continue

        message = tx["transaction"]["message"]
        account_keys = build_account_keys(tx)
        token_info = extract_token_accounts(tx, account_keys)
        fee_payer = message.get("accountKeys", [None])[0]

        for compiled in message.get("instructions") or []:
            program_index = compiled.get("programIdIndex")
            if program_index is None or program_index >= len(account_keys):
                continue
            program_id = account_keys[program_index]
            if program_id != JUPITER_PROGRAM:
                continue

            indices = compiled.get("accounts") or []
            accounts: List[str] = []
            for idx in indices:
                if isinstance(idx, int) and idx < len(account_keys):
                    accounts.append(account_keys[idx])

            segments = slice_segments(accounts, owner_set)
            for segment in segments:
                owner = segment["owner"]
                owner_info = dex_infos.get(owner)
                if not owner_info:
                    continue
                pool = identify_pool(segment["accounts"], owner_info)
                if not pool:
                    continue

                normalized_key, template, actuals = normalize_segment(
                    segment["accounts"], owner, pool, token_info, fee_payer
                )
                key = tuple(normalized_key)

                pool_entry = pool_stats.setdefault(
                    pool.pubkey,
                    {
                        "pool": pool.pubkey,
                        "owner": owner,
                        "dex_name": owner_info.name,
                        "count": 0,
                        "patterns": {},
                    },
                )
                pattern = pool_entry["patterns"].setdefault(
                    key,
                    {
                        "count": 0,
                        "template": deepcopy(template),
                        "samples": defaultdict(set),
                    },
                )
                pattern["count"] += 1
                for entry, actual in zip(template, actuals):
                    sample_key = entry.get("sample_key")
                    if sample_key:
                        pattern["samples"][sample_key].add(actual)

                pool_entry["count"] += 1
                total_segments += 1

    if total_segments == 0:
        print("未统计到任何 Jupiter 交换段")
        return 0

    top_limit = args.top if args.top and args.top > 0 else None
    min_count = max(1, args.min_count)

    pools_output = []
    for pool_pubkey, data in pool_stats.items():
        if data["count"] < min_count:
            continue
        patterns_output = []
        for pattern_key, pattern in sorted(
            data["patterns"].items(), key=lambda item: item[1]["count"], reverse=True
        ):
            swap_items = compile_swap_items(pattern["template"], pattern["samples"])
            patterns_output.append(
                {
                    "count": pattern["count"],
                    "share": pattern["count"] / data["count"] if data["count"] else 0.0,
                    "swap": swap_items,
                }
            )
        pools_output.append(
            {
                "pool": data["pool"],
                "owner": data["owner"],
                "dex_name": data["dex_name"],
                "count": data["count"],
                "share": data["count"] / total_segments,
                "patterns": patterns_output,
            }
        )

    pools_output.sort(key=lambda item: item["count"], reverse=True)
    if top_limit is not None:
        pools_output = pools_output[:top_limit]

    summary = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_segments": total_segments,
        "pool_count": len(pools_output),
        "pools": pools_output,
    }

    indent = 2 if args.pretty else None
    output_path.write_text(json.dumps(summary, ensure_ascii=False, indent=indent), encoding="utf-8")

    print(f"已写入 {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
