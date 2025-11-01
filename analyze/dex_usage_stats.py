#!/usr/bin/env python3
"""
统计交易中使用到的 DEX 程序。
默认读取 analyze/txs 中的 JSON 交易，并基于 analyze/dexes.json 的映射输出统计。
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Dict, List, Optional, Set


def load_program_mapping(path: Path) -> Dict[str, str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return {program.upper(): name for program, name in data.items()}


def normalize_account_key(entry: object) -> Optional[str]:
    if isinstance(entry, str):
        return entry
    if isinstance(entry, dict):
        value = entry.get("pubkey")
        if isinstance(value, str):
            return value
    return None


def collect_account_keys(tx: Dict[str, object]) -> List[str]:
    transaction = tx.get("transaction")
    if not isinstance(transaction, dict):
        return []

    message = transaction.get("message")
    if not isinstance(message, dict):
        return []

    keys: List[str] = []
    raw_keys = message.get("accountKeys") or []
    for item in raw_keys:
        key = normalize_account_key(item)
        if key:
            keys.append(key)

    meta = tx.get("meta")
    if isinstance(meta, dict):
        loaded = meta.get("loadedAddresses") or {}
        if isinstance(loaded, dict):
            for bucket in ("writable", "readonly"):
                extra = loaded.get(bucket) or []
                for addr in extra:
                    if isinstance(addr, str):
                        keys.append(addr)

    return keys


def program_ids_from_instruction(
    inst: Dict[str, object],
    account_keys: List[str],
) -> Optional[str]:
    program_id = inst.get("programId")
    if isinstance(program_id, str):
        return program_id

    index = inst.get("programIdIndex")
    if isinstance(index, int):
        if 0 <= index < len(account_keys):
            return account_keys[index]
    return None


def gather_program_ids(tx: Dict[str, object]) -> Set[str]:
    programs: Set[str] = set()
    account_keys = collect_account_keys(tx)

    transaction = tx.get("transaction")
    message = transaction.get("message") if isinstance(transaction, dict) else None

    if isinstance(message, dict):
        for field in ("instructions", "compiledInstructions"):
            items = message.get(field) or []
            for inst in items:
                if isinstance(inst, dict):
                    program_id = program_ids_from_instruction(inst, account_keys)
                    if program_id:
                        programs.add(program_id)

    meta = tx.get("meta")
    if isinstance(meta, dict):
        inner = meta.get("innerInstructions") or []
        for container in inner:
            if not isinstance(container, dict):
                continue
            for inst in container.get("instructions") or []:
                if isinstance(inst, dict):
                    program_id = program_ids_from_instruction(inst, account_keys)
                    if program_id:
                        programs.add(program_id)

    return programs


def list_json_files(path: Path) -> List[Path]:
    return sorted(
        p for p in path.iterdir()
        if p.is_file() and p.suffix == ".json"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="统计交易中使用到的 DEX 程序")
    parser.add_argument(
        "--tx-dir",
        default=None,
        help="交易 JSON 所在目录（默认: 自动匹配脚本附近的 txs 目录）",
    )
    parser.add_argument(
        "--dexes",
        default=None,
        help="DEX 程序映射文件（默认: 自动匹配脚本附近的 dexes.json）",
    )
    parser.add_argument(
        "--show-unknown",
        action="store_true",
        help="输出未在 dexes 映射中的 programId 统计",
    )
    return parser.parse_args()


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    args = parse_args()

    if args.tx_dir:
        tx_dir = Path(args.tx_dir)
    else:
        candidates = [
            script_dir / "txs",
            script_dir / "analyze/txs",
            script_dir.parent / "analyze/txs",
        ]
        tx_dir = next((p for p in candidates if p.exists()), candidates[0])

    if not tx_dir.exists():
        print(f"交易目录不存在: {tx_dir}")
        return 1

    if args.dexes:
        dex_path = Path(args.dexes)
    else:
        dex_candidates = [
            script_dir / "dexes.json",
            script_dir.parent / "analyze/dexes.json",
        ]
        dex_path = next((p for p in dex_candidates if p.exists()), dex_candidates[0])
    if not dex_path.exists():
        print(f"DEX 映射文件不存在: {dex_path}")
        return 1

    print(f"读取交易目录: {tx_dir}")
    print(f"使用 DEX 映射: {dex_path}")

    dex_mapping = load_program_mapping(dex_path)
    files = list_json_files(tx_dir)
    if not files:
        print(f"{tx_dir} 下未找到任何交易 JSON")
        return 0

    dex_counter: Counter[str] = Counter()
    program_counter: Counter[str] = Counter()
    total_txs = 0

    for path in files:
        try:
            tx = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            print(f"跳过无法解析的交易: {path}")
            continue

        total_txs += 1
        programs = gather_program_ids(tx)
        seen_dexes: Set[str] = set()

        for program in programs:
            program_upper = program.upper()
            program_counter[program_upper] += 1
            dex_name = dex_mapping.get(program_upper)
            if dex_name:
                seen_dexes.add(dex_name)

        for dex_name in seen_dexes:
            dex_counter[dex_name] += 1

    print(f"统计交易数: {total_txs}")
    if not dex_counter:
        print("未匹配到任何已知 DEX")
    else:
        print("DEX 使用情况（按涉及交易数排序）:")
        for dex_name, count in dex_counter.most_common():
            percentage = count / total_txs * 100 if total_txs else 0
            print(f"  - {dex_name}: {count} tx ({percentage:.1f}%)")

    if args.show_unknown:
        known_programs = set(dex_mapping.keys())
        unknown_counter = Counter({
            program: count
            for program, count in program_counter.items()
            if program not in known_programs
        })
        if unknown_counter:
            print("未在映射中的 programId（按出现次数排序）:")
            for program, count in unknown_counter.most_common():
                print(f"  - {program}: {count}")
        else:
            print("所有 programId 均在映射表中")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
