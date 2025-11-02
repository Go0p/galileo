#!/usr/bin/env python3
"""
从 Jupiter 交易中剥离 swap 账户信息，按 (program_id, pool) 聚合生成初版 dexes_swap_market。

示例：
    python3 analyze/build_swap_market_demo.py --top 20 --pretty
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

JUPITER_PROGRAM = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

def resolve_tx_dir(script_dir: Path, override: Optional[str]) -> Path:
    if override:
        return Path(override)
    candidates = [
        script_dir / "txs",
        script_dir / "analyze" / "txs",
        script_dir.parent / "analyze" / "txs",
    ]
    for directory in candidates:
        if directory.exists():
            return directory
    return candidates[0]


def resolve_file(path: Optional[str], candidates: Sequence[Path]) -> Path:
    if path:
        return Path(path)
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[-1]


def load_json(path: Path):
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def build_account_keys(tx: dict) -> List[str]:
    message = tx["transaction"]["message"]
    keys = list(message.get("accountKeys") or [])
    loaded = tx.get("meta", {}).get("loadedAddresses") or {}
    keys.extend(loaded.get("writable", []))
    keys.extend(loaded.get("readonly", []))
    return keys


def extract_token_info(tx: dict, account_keys: List[str]) -> Dict[str, Dict]:
    meta = tx.get("meta") or {}
    token_info: Dict[str, Dict] = {}
    balances = (meta.get("preTokenBalances") or []) + (meta.get("postTokenBalances") or [])
    for entry in balances:
        index = entry.get("accountIndex")
        if index is None or index >= len(account_keys):
            continue
        address = account_keys[index]
        info = token_info.setdefault(address, {})
        for key in ("owner", "mint", "programId"):
            value = entry.get(key)
            if value:
                info.setdefault(key, value)
    return token_info


def slice_segments(accounts: List[str], owner_set: set[str]) -> List[Tuple[str, List[str]]]:
    """返回 (owner_program, accounts_chunk)。"""
    segments: List[Tuple[str, List[str]]] = []
    i = 0
    seen_owner = False
    while i < len(accounts):
        account = accounts[i]
        if account == JUPITER_PROGRAM and seen_owner:
            i += 1
            continue
        if account.upper() not in owner_set:
            i += 1
            continue
        owner = account
        seen_owner = True
        chunk = [owner]
        j = i + 1
        while j < len(accounts):
            current = accounts[j]
            if current == JUPITER_PROGRAM and seen_owner:
                j += 1
                continue
            if current.upper() in owner_set and current != owner:
                break
            chunk.append(current)
            j += 1
        segments.append((owner, chunk))
        i = j
    return segments


@dataclass
class AccountAggregate:
    addresses: Counter = field(default_factory=Counter)
    owners: Counter = field(default_factory=Counter)
    mints: Counter = field(default_factory=Counter)
    token_programs: Counter = field(default_factory=Counter)

    def update(self, address: str, token_info: Optional[Dict]):
        self.addresses.update([address])
        if token_info:
            owner = token_info.get("owner")
            mint = token_info.get("mint")
            prog = token_info.get("programId")
            if owner:
                self.owners.update([owner])
            if mint:
                self.mints.update([mint])
            if prog:
                self.token_programs.update([prog])

    def summary(self, max_samples: int = 5) -> Dict[str, object]:
        result: Dict[str, object] = {
            "account": self.addresses.most_common(1)[0][0] if self.addresses else None,
            "account_samples": [addr for addr, _ in self.addresses.most_common(max_samples)],
        }
        if self.owners:
            result["owner"] = self.owners.most_common(1)[0][0]
        if self.mints:
            result["mint"] = self.mints.most_common(1)[0][0]
        if self.token_programs:
            result["token_program"] = self.token_programs.most_common(1)[0][0]
        return result


@dataclass
class PoolAggregate:
    program_id: str
    dex_name: Optional[str]
    pool: str
    fields: List[str]
    count: int = 0
    first_seen: Optional[int] = None
    last_seen: Optional[int] = None
    account_stats: Dict[int, AccountAggregate] = field(default_factory=lambda: defaultdict(AccountAggregate))

    def ensure_capacity(self, length: int) -> None:
        while len(self.fields) < length:
            self.fields.append(f"slot_{len(self.fields)}")

    def update(
        self,
        accounts: List[str],
        block_time: Optional[int],
        token_info: Dict[str, Dict],
    ) -> None:
        self.count += 1
        if block_time:
            if self.first_seen is None or block_time < self.first_seen:
                self.first_seen = block_time
            if self.last_seen is None or block_time > self.last_seen:
                self.last_seen = block_time

        self.ensure_capacity(len(accounts))

        for index, address in enumerate(accounts):
            aggregate = self.account_stats[index]
            aggregate.update(address, token_info.get(address))

    def to_record(self) -> Dict[str, object]:
        first_dt = PoolAggregate._format_time(self.first_seen)
        last_dt = PoolAggregate._format_time(self.last_seen)

        swap_accounts: List[Dict[str, object]] = []
        for index in sorted(self.account_stats):
            field_name = self.fields[index]
            summary = self.account_stats[index].summary()
            entry: Dict[str, object] = {"field": field_name}
            entry.update(summary)
            swap_accounts.append(entry)

        return {
            "program_id": self.program_id,
            "dex_name": self.dex_name,
            "pool": self.pool,
            "count": self.count,
            "first_seen_at": first_dt,
            "last_seen_at": last_dt,
            "swap_accounts": swap_accounts,
        }

    @staticmethod
    def _format_time(ts: Optional[int]) -> Optional[str]:
        if not ts:
            return None
        return datetime.fromtimestamp(ts, tz=timezone.utc).isoformat()


def parse_args(script_dir: Path) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="构建 dexes_swap_market 初版数据")
    parser.add_argument("--tx-dir", default=None, help="Jupiter 交易 JSON 目录")
    parser.add_argument("--dexes", default=None, help="dexes.json 路径（默认自动寻找）")
    parser.add_argument("--markets", default=None, help="markets_filtered.json 路径")
    parser.add_argument("--output", default=None, help="输出文件（默认 analyze/dexes_swap_market.json）")
    parser.add_argument("--top", type=int, default=0, help="仅保留出现次数最多的前 N 个池子（0 表示全部）")
    parser.add_argument("--pretty", action="store_true", help="以缩进格式写出 JSON")
    parser.add_argument("--min-count", type=int, default=1, help="过滤出现次数低于阈值的池子")
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
    output_path = Path(args.output) if args.output else script_dir / "dexes_swap_market.json"

    if not tx_dir.exists():
        print(f"交易目录不存在: {tx_dir}")
        return 1
    if not dex_path.exists():
        print(f"dexes 文件不存在: {dex_path}")
        return 1
    if not markets_path.exists():
        print(f"markets 文件不存在: {markets_path}")
        return 1

    print(f"读取交易目录: {tx_dir}")
    print(f"使用 dex 映射: {dex_path}")
    print(f"使用 markets: {markets_path}")
    print(f"输出文件: {output_path}")

    dex_mapping = load_json(dex_path)
    program_to_name_upper = {program.upper(): name for program, name in dex_mapping.items()}

    schema_path = script_dir / "dex_field_schema.json"
    if schema_path.exists():
        dex_schemas_raw = load_json(schema_path)
    else:
        print("警告: 未找到 dex_field_schema.json，无法映射字段")
        dex_schemas_raw = {}

    schema_map: Dict[str, Dict[str, object]] = {}
    for program_id, entry in dex_schemas_raw.items():
        fields = entry.get("fields", [])
        name = entry.get("name")
        schema_map[program_id.upper()] = {
            "name": name,
            "fields": list(fields),
        }
    markets = load_json(markets_path)
    pool_to_owner = {}
    owner_to_pools: Dict[str, set] = defaultdict(set)
    for entry in markets:
        owner = entry.get("owner")
        pool = entry.get("pubkey")
        if owner and pool:
            pool_to_owner[pool] = owner
            owner_to_pools[owner].add(pool)

    owner_programs = set(schema_map.keys())
    owner_programs.update(owner.upper() for owner in owner_to_pools.keys())

    # 聚合容器
    aggregates: Dict[Tuple[str, str], PoolAggregate] = {}
    # 仅统计池子出现次数

    tx_files = sorted(tx_dir.glob("*.json"))
    for tx_file in tx_files:
        try:
            tx = load_json(tx_file)
        except json.JSONDecodeError:
            print(f"跳过无法解析的交易: {tx_file}")
            continue

        account_keys = build_account_keys(tx)
        token_info = extract_token_info(tx, account_keys)
        block_time = tx.get("blockTime") or tx.get("slot")

        for inst in tx["transaction"]["message"].get("instructions") or []:
            program_index = inst.get("programIdIndex")
            if program_index is None or program_index >= len(account_keys):
                continue
            program_id = account_keys[program_index]
            if program_id != JUPITER_PROGRAM:
                continue

            accounts = []
            for idx in inst.get("accounts") or []:
                if isinstance(idx, int) and idx < len(account_keys):
                    accounts.append(account_keys[idx])

            segments = slice_segments(accounts, owner_programs)
            for owner, chunk in segments:
                schema_entry = schema_map.get(owner.upper())
                possible_pools = owner_to_pools.get(owner) or owner_to_pools.get(owner.upper())
                pool = None
                if possible_pools:
                    for acc in chunk[1:]:
                        if acc in possible_pools:
                            pool = acc
                            break
                if not pool:
                    # 如果没找到池子，跳过
                    continue

                key = (owner, pool)
                if key not in aggregates:
                    if schema_entry:
                        fields = list(schema_entry.get("fields", []))
                        dex_name = schema_entry.get("name") or program_to_name_upper.get(owner.upper())
                    else:
                        fields = [f"slot_{i}" for i in range(len(chunk))]
                        dex_name = program_to_name_upper.get(owner.upper()) or owner
                    aggregates[key] = PoolAggregate(
                        program_id=owner,
                        dex_name=dex_name,
                        pool=pool,
                        fields=fields,
                    )

                aggregate = aggregates[key]
                aggregate.update(chunk, block_time, token_info)

    records = [aggregate.to_record() for aggregate in aggregates.values()]
    records.sort(key=lambda item: item["count"], reverse=True)

    if args.min_count > 1:
        records = [item for item in records if item["count"] >= args.min_count]

    if args.top and args.top > 0:
        records = records[:args.top]

    summary = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "tx_dir": str(tx_dir),
        "total_pools": len(records),
        "total_swaps": sum(item["count"] for item in records),
        "pools": records,
    }

    indent = 2 if args.pretty else None
    output_path.write_text(json.dumps(summary, ensure_ascii=False, indent=indent), encoding="utf-8")
    print(f"已写入 {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
