"""
Quick parser for `reverser/goonfi/swap.txt`.

The notebook-style dump in `swap.txt` makes人工 diff / grep 较麻烦，
这里提供一个极简的解析脚本，把每条 swap 的账户顺序、指令数据整理成结构化
JSON，方便后续分析 PDA seeds 或自动化测试。
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Iterable, Iterator, List


SAMPLE_FILE = Path(__file__).with_name("swap.txt")


@dataclass
class AccountEntry:
    index: int
    label: str
    pubkey: str
    writable: bool
    signer: bool


@dataclass
class SwapSample:
    program: str
    pool: str
    accounts: List[AccountEntry]
    instruction_data: str


ACCOUNT_RE = re.compile(r"#(?P<index>\d+)\s+-\s+(?P<label>[^:]+):(?P<rest>.*)")
BASE58_RE = re.compile(r"[1-9A-HJ-NP-Za-km-z]{32,44}")
SPECIAL_PUBKEYS = {
    "sysvar: instructions": "Sysvar1nstructions1111111111111111111111111",
    "token program program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
}


def parse_accounts(lines: Iterable[str]) -> Iterator[AccountEntry]:
    for raw in lines:
        raw = raw.strip()
        if not raw.startswith("#"):
            break
        m = ACCOUNT_RE.match(raw)
        if not m:
            continue
        idx = int(m.group("index"))
        label = m.group("label").strip()
        rest = m.group("rest")
        matches = BASE58_RE.findall(rest or "")
        if matches:
            pubkey = matches[0]
        else:
            normalized = (rest or "").strip().lower()
            pubkey = SPECIAL_PUBKEYS.get(normalized)
        if not pubkey:
            continue
        writable = "Writable" in raw
        signer = "Signer" in raw
        yield AccountEntry(
            index=idx,
            label=label,
            pubkey=pubkey,
            writable=writable,
            signer=signer,
        )


def parse_samples(text: str) -> list[SwapSample]:
    blocks = text.strip().split("Interact With")
    samples: list[SwapSample] = []
    for block in blocks:
        block = block.strip()
        if not block:
            continue
        lines = block.splitlines()
        program_line = lines[0].strip()
        program = program_line.split("-")[-1].strip()
        pool_line = next((ln for ln in lines if ln.startswith("#2 - Market")), "")
        pool_match = re.search(r"([1-9A-HJ-NP-Za-km-z]{32,44})", pool_line)
        pool = pool_match.group(1) if pool_match else ""
        account_lines = []
        instruction_data = ""
        reading_accounts = False
        for ln in lines:
            if ln.startswith("Input Accounts"):
                reading_accounts = True
                continue
            if ln.startswith("Instruction Data"):
                instruction_data = ln.split("Instruction Data", 1)[1].strip()
                break
            if reading_accounts:
                account_lines.append(ln)
        accounts = list(parse_accounts(account_lines))
        samples.append(
            SwapSample(
                program=program,
                pool=pool,
                accounts=accounts,
                instruction_data=instruction_data,
            )
        )
    return samples


def main() -> None:
    raw = SAMPLE_FILE.read_text(encoding="utf-8")
    parsed = parse_samples(raw)
    print(json.dumps([asdict(sample) for sample in parsed], ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
