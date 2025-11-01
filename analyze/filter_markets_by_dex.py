#!/usr/bin/env python3
"""
根据 DEX 名称过滤 markets.json 中的池子。
默认筛选最近统计中出现的 DEX，并将结果写入 analyze/markets_filtered.json。
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, Iterable, List, Set

DEFAULT_DEX_NAMES = [
    "Raydium CLMM",
    "Saros",
    "TesseraV",
    "HumidiFi",
    "Meteora DLMM",
    "ZeroFi",
    "Whirlpool",
    "GoonFi",
    "PancakeSwap",
    "Raydium CP",
    "SolFi V2",
    "Lifinity V2",
    "Byreal",
    "Aquifer",
    "OpenBook V2",
]


def load_dex_mapping(path: Path) -> Dict[str, str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return {name: program for program, name in data.items()}


def parse_args(script_dir: Path) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="按 DEX 名称过滤市场列表")
    parser.add_argument(
        "--markets",
        default=script_dir / "markets.json",
        help="markets.json 路径（默认: 脚本同目录下）",
    )
    parser.add_argument(
        "--dexes",
        default=script_dir / "dexes.json",
        help="dexes.json 路径（默认: 脚本同目录下）",
    )
    parser.add_argument(
        "--names",
        nargs="+",
        default=DEFAULT_DEX_NAMES,
        help="需要保留的 DEX 名称列表（默认: 最新统计结果）",
    )
    parser.add_argument(
        "--output",
        default=script_dir / "markets_filtered.json",
        help="输出文件路径（默认: markets_filtered.json）",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="以缩进格式写出 JSON，便于阅读",
    )
    return parser.parse_args()


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    args = parse_args(script_dir)

    markets_path = Path(args.markets)
    dexes_path = Path(args.dexes)
    output_path = Path(args.output)

    if not markets_path.exists():
        print(f"markets 文件不存在: {markets_path}")
        return 1
    if not dexes_path.exists():
        print(f"dexes 文件不存在: {dexes_path}")
        return 1

    dex_mapping = load_dex_mapping(dexes_path)
    target_names: Set[str] = {name.strip() for name in args.names}
    missing_names = [name for name in target_names if name not in dex_mapping]
    if missing_names:
        print("警告: 下列 DEX 名称未在映射表中找到，将被忽略：")
        for name in missing_names:
            print(f"  - {name}")
        target_names -= set(missing_names)

    target_programs = {dex_mapping[name] for name in target_names}

    markets = json.loads(markets_path.read_text(encoding="utf-8"))
    if not isinstance(markets, list):
        print(f"markets 文件格式异常，期望为列表: {markets_path}")
        return 1

    filtered = [
        item for item in markets
        if isinstance(item, dict) and item.get("owner") in target_programs
    ]

    indent = 2 if args.pretty else None
    output_path.write_text(
        json.dumps(filtered, ensure_ascii=False, indent=indent),
        encoding="utf-8",
    )

    print(f"总计市场数: {len(markets)}")
    print(f"筛选后市场数: {len(filtered)}")
    print(f"输出文件: {output_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
