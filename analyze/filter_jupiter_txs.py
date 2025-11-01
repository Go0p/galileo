#!/usr/bin/env python3
"""
筛选出包含 JUP 聚合器合约的交易 JSON。
默认读取 analyze/txs，将命中交易复制到 analyze/txs_jupiter，
可通过 --in-place 删除不匹配的交易。
"""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path
from typing import List

TARGET_PROGRAM = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"


def contains_target_program(raw_json: str, target: str) -> bool:
    target_upper = target.upper()
    return target_upper in raw_json.upper()


def list_json_files(path: Path) -> List[Path]:
    return sorted(p for p in path.iterdir() if p.is_file() and p.suffix == ".json")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="筛选出包含 Jupiter 聚合器的交易")
    parser.add_argument(
        "--input",
        default="analyze/txs",
        help="交易 JSON 的输入目录（默认: analyze/txs）",
    )
    parser.add_argument(
        "--output",
        default="analyze/txs_jupiter",
        help="筛选后输出目录（默认: analyze/txs_jupiter）",
    )
    parser.add_argument(
        "--program",
        default=TARGET_PROGRAM,
        help=f"目标合约地址（默认: {TARGET_PROGRAM}）",
    )
    parser.add_argument(
        "--in-place",
        action="store_true",
        help="在原目录中就地删除不匹配的交易，而不复制到新目录",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    input_dir = Path(args.input)
    output_dir = Path(args.output)

    if not input_dir.exists():
        print(f"输入目录不存在: {input_dir}")
        return 1

    files = list_json_files(input_dir)
    if not files:
        print(f"{input_dir} 中未找到任何 JSON 文件")
        return 0

    matches: List[Path] = []
    rejects: List[Path] = []

    for path in files:
        raw = path.read_text(encoding="utf-8", errors="ignore")
        if contains_target_program(raw, args.program):
            matches.append(path)
        else:
            rejects.append(path)

    if args.in_place:
        for path in rejects:
            path.unlink(missing_ok=True)
        kept = len(matches)
        removed = len(rejects)
        print(f"保留 {kept} 个交易，删除 {removed} 个不匹配的交易")
    else:
        output_dir.mkdir(parents=True, exist_ok=True)
        for src in matches:
            dst = output_dir / src.name
            shutil.copy2(src, dst)
        kept = len(matches)
        skipped = len(rejects)
        print(f"复制 {kept} 个匹配交易到 {output_dir}，跳过 {skipped} 个不匹配的交易")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
