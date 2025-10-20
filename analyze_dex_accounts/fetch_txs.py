#!/usr/bin/env python3
"""
从指定钱包地址抓取最近若干笔交易，解析 Jupiter 路由及各 DEX 的账户排列。

用法示例：
    python3 fetch_txs.py <WALLET_PUBKEY> [COUNT] [--fetch-only]

    - COUNT  默认 10，可传负数（会取绝对值），例如用户示例中的 `-10`。
    - 会将交易 JSON 存放在 ./<wallet>/<signature>.json
    - 若未加 --fetch-only，会额外输出：
        * <wallet>/jupiter_accounts.txt      — Jupiter 指令账户顺序
        * <wallet>/dex/<label>.txt           — 按 Program ID 聚合的 DEX 账户顺序

脚本偏向离线分析，所有 RPC 请求默认走本地节点 (http://127.0.0.1:8899)。
"""
from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Tuple

RPC_ENDPOINT = "http://127.0.0.1:8899"
JUPITER_V6_PROGRAM_ID = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"

IGNORED_PROGRAM_IDS = {
    "11111111111111111111111111111111",  # system
    "ComputeBudget111111111111111111111111111111",
    "Sysvar1nstructions1111111111111111111111111",
    "SysvarC1ock11111111111111111111111111111111",
    "SysvarRent111111111111111111111111111111111",
    "BPFLoaderUpgradeab1e11111111111111111111111",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
}


@dataclass(frozen=True)
class AccountEntry:
    pubkey: str
    writable: bool
    signer: bool


@dataclass
class JupiterOccurrence:
    signature: str
    outer_index: int
    failed: bool
    accounts: List[AccountEntry]


def rpc_request(method: str, params: List[object]) -> Dict[str, object]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    }
    data = json.dumps(payload)
    result = subprocess.run(
        [
            "curl",
            "-sS",
            RPC_ENDPOINT,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            data,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"curl failed: {result.stderr.strip() or result.stdout}"
        )

    response = json.loads(result.stdout)
    if "error" in response:
        raise RuntimeError(f"RPC error: {response['error']}")
    return response


def fetch_signatures(address: str, limit: int) -> List[str]:
    response = rpc_request(
        "getSignaturesForAddress",
        [
            address,
            {
                "limit": limit,
            },
        ],
    )
    result = response.get("result") or []
    signatures = []
    for entry in result:
        signature = entry.get("signature")
        if signature:
            signatures.append(signature)
    return signatures


def fetch_transaction(signature: str) -> Dict[str, object]:
    response = rpc_request(
        "getTransaction",
        [
            signature,
            {
                "encoding": "json",
                "maxSupportedTransactionVersion": 0,
            },
        ],
    )
    result = response.get("result")
    if result is None:
        raise RuntimeError(f"transaction not found for {signature}")
    response["signature"] = signature
    return response


def load_program_labels(root: Path) -> Dict[str, str]:
    mapping_path = root / "program-id-to-label.json"
    with mapping_path.open("r", encoding="utf-8") as fh:
        raw = json.load(fh)
    return {str(k): str(v) for k, v in raw.items()}


def build_account_table(
    result: Dict[str, object]
) -> Tuple[List[AccountEntry], Dict[str, AccountEntry]]:
    transaction = result.get("transaction")
    meta = result.get("meta") or {}
    if not transaction:
        raise ValueError("missing transaction payload in RPC response")

    message = transaction.get("message")
    if not message:
        raise ValueError("missing transaction.message in RPC response")

    account_keys = message.get("accountKeys", [])
    header = message.get("header") or {}
    num_required = int(header.get("numRequiredSignatures", 0))
    readonly_signed = int(header.get("numReadonlySignedAccounts", 0))
    readonly_unsigned = int(header.get("numReadonlyUnsignedAccounts", 0))

    writable_signers = max(num_required - readonly_signed, 0)
    unsigned_count = max(len(account_keys) - num_required, 0)
    writable_unsigned = max(unsigned_count - readonly_unsigned, 0)

    static_entries: List[AccountEntry] = []
    for idx, raw in enumerate(account_keys):
        default_writable = False
        if idx < num_required:
            default_writable = idx < writable_signers
        else:
            unsigned_idx = idx - num_required
            default_writable = unsigned_idx < writable_unsigned

        if isinstance(raw, dict):
            pubkey = str(raw.get("pubkey"))
            writable_flag = raw.get("writable")
            writable = default_writable if writable_flag is None else bool(writable_flag)
        else:
            pubkey = str(raw)
            writable = default_writable

        if not pubkey:
            raise ValueError(f"account key missing pubkey at index {idx}")

        static_entries.append(
            AccountEntry(
                pubkey=pubkey,
                writable=writable,
                signer=idx < num_required,
            )
        )

    loaded_addresses = meta.get("loadedAddresses") or {}
    lookup_writable = [
        AccountEntry(pubkey=str(addr), writable=True, signer=False)
        for addr in loaded_addresses.get("writable", [])
    ]
    lookup_readonly = [
        AccountEntry(pubkey=str(addr), writable=False, signer=False)
        for addr in loaded_addresses.get("readonly", [])
    ]

    all_entries = static_entries + lookup_writable + lookup_readonly
    lookup: Dict[str, AccountEntry] = {}
    for entry in all_entries:
        lookup[entry.pubkey] = entry

    return all_entries, lookup


def resolve_program_id(
    instr: Dict[str, object],
    account_entries: List[AccountEntry],
    lookup: Dict[str, AccountEntry],
) -> Optional[str]:
    program_id = instr.get("programId")
    if program_id:
        return str(program_id)

    program_index = instr.get("programIdIndex")
    if program_index is None:
        return None

    try:
        return account_entries[int(program_index)].pubkey
    except (IndexError, ValueError):
        return None


def resolve_accounts(
    accounts_field: Iterable[object],
    account_entries: List[AccountEntry],
    lookup: Dict[str, AccountEntry],
) -> List[AccountEntry]:
    resolved: List[AccountEntry] = []
    for item in accounts_field:
        if isinstance(item, int):
            try:
                resolved.append(account_entries[item])
            except IndexError:
                continue
        else:
            pubkey = str(item)
            entry = lookup.get(pubkey)
            if entry is None:
                entry = AccountEntry(pubkey=pubkey, writable=False, signer=False)
            resolved.append(entry)
    return resolved


class AnalysisState:
    def __init__(self, labels: Dict[str, str]) -> None:
        self.labels = labels
        self.jupiter_occurrences: List[JupiterOccurrence] = []
        self.dex_sequences: Dict[
            str, Dict[Tuple[Tuple[str, bool], ...], List[Tuple[str, int, int, bool]]]
        ] = {}

    def add_jupiter(
        self,
        signature: str,
        outer_index: int,
        failed: bool,
        accounts: List[AccountEntry],
    ) -> None:
        self.jupiter_occurrences.append(
            JupiterOccurrence(
                signature=signature,
                outer_index=outer_index,
                failed=failed,
                accounts=list(accounts),
            )
        )

    def add_dex(
        self,
        program_id: str,
        accounts: List[AccountEntry],
        signature: str,
        outer_index: int,
        inner_index: int,
        failed: bool,
    ) -> None:
        sequence = tuple((entry.pubkey, entry.writable) for entry in accounts)
        program_bucket = self.dex_sequences.setdefault(program_id, {})
        program_bucket.setdefault(sequence, []).append(
            (signature, outer_index, inner_index, failed)
        )


def analyse_transaction(
    signature: str,
    payload: Dict[str, object],
    state: AnalysisState,
) -> None:
    result = payload.get("result")
    if not result:
        return

    meta = result.get("meta") or {}
    transaction = result.get("transaction")
    if not transaction:
        return

    account_entries, lookup = build_account_table(result)
    message = transaction.get("message") or {}
    compiled_instructions = message.get("instructions") or []

    failed = meta.get("err") is not None

    jupiter_indices: List[int] = []
    for idx, instr in enumerate(compiled_instructions):
        program_index = instr.get("programIdIndex")
        if program_index is None:
            continue
        try:
            program_id = account_entries[int(program_index)].pubkey
        except (IndexError, ValueError):
            continue

        if program_id != JUPITER_V6_PROGRAM_ID:
            continue

        accounts = resolve_accounts(
            instr.get("accounts", []),
            account_entries,
            lookup,
        )
        jupiter_indices.append(idx)
        state.add_jupiter(signature, idx, failed, accounts)

    if not jupiter_indices:
        return

    jupiter_index_set = set(jupiter_indices)

    inner_blocks = meta.get("innerInstructions") or []
    for block in inner_blocks:
        outer_index = int(block.get("index", -1))
        if outer_index not in jupiter_index_set:
            continue

        for inner_index, instr in enumerate(block.get("instructions", [])):
            program_id = resolve_program_id(instr, account_entries, lookup)
            if not program_id:
                continue
            if program_id == JUPITER_V6_PROGRAM_ID:
                continue
            if program_id in IGNORED_PROGRAM_IDS:
                continue

            accounts = resolve_accounts(
                instr.get("accounts", []),
                account_entries,
                lookup,
            )
            if not accounts:
                continue

            state.add_dex(
                program_id=program_id,
                accounts=accounts,
                signature=signature,
                outer_index=outer_index,
                inner_index=inner_index,
                failed=failed,
            )


def format_roles(entry: AccountEntry) -> str:
    roles = []
    if entry.signer:
        roles.append("signer")
    roles.append("writable" if entry.writable else "readonly")
    return ",".join(roles)


def write_jupiter_accounts(wallet_dir: Path, occurrences: List[JupiterOccurrence]) -> None:
    if not occurrences:
        return

    output = []
    for occ in occurrences:
        status = "failed" if occ.failed else "ok"
        output.append(f"signature : {occ.signature} (outer={occ.outer_index}, status={status})")
        for idx, entry in enumerate(occ.accounts):
            output.append(f"{idx:02d} {entry.pubkey} [{format_roles(entry)}]")
        output.append("")

    (wallet_dir / "jupiter_accounts.txt").write_text(
        "\n".join(output).rstrip() + "\n",
        encoding="utf-8",
    )


def write_dex_reports(
    wallet_dir: Path,
    dex_sequences: Dict[str, Dict[Tuple[Tuple[str, bool], ...], List[Tuple[str, int, int, bool]]]],
    labels: Dict[str, str],
) -> None:
    if not dex_sequences:
        return

    dex_dir = wallet_dir / "dex"
    dex_dir.mkdir(exist_ok=True)

    for program_id, sequences in sorted(dex_sequences.items()):
        label = labels.get(program_id, program_id)
        file_name = f"{label.replace(' ', '_')}.txt"
        path = dex_dir / file_name

        lines = [f"program id : {program_id}"]
        if label != program_id:
            lines.append(f"label : {label}")

        for accounts, occurrences in sequences.items():
            lines.append("")
            occ_text = ", ".join(
                f"{sig} (outer={outer}, inner={inner}, {'failed' if failed else 'ok'})"
                for sig, outer, inner, failed in occurrences
            )
            lines.append(f"occurrences : {occ_text}")
            for pubkey, writable in accounts:
                marker = "W" if writable else "R"
                lines.append(f"{pubkey}:{marker}")

        path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def ensure_transactions(
    wallet_dir: Path,
    signatures: List[str],
) -> None:
    for signature in signatures:
        out_path = wallet_dir / f"{signature}.json"
        if out_path.exists():
            continue
        print(f"fetching {signature}")
        data = fetch_transaction(signature)
        out_path.write_text(json.dumps(data, indent=2), encoding="utf-8")


def analyse_wallet(wallet_dir: Path, labels: Dict[str, str]) -> None:
    state = AnalysisState(labels)
    for json_file in sorted(wallet_dir.glob("*.json")):
        payload = json.loads(json_file.read_text())
        signature = payload.get("signature") or json_file.stem
        analyse_transaction(signature, payload, state)

    write_jupiter_accounts(wallet_dir, state.jupiter_occurrences)
    write_dex_reports(wallet_dir, state.dex_sequences, labels)


def load_signatures_from_disk(wallet_dir: Path) -> List[str]:
    return [path.stem for path in wallet_dir.glob("*.json")]


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Fetch Jupiter swaps for a wallet and analyse DEX account order."
    )
    parser.add_argument("wallet", help="钱包公钥")
    parser.add_argument(
        "count",
        nargs="?",
        type=int,
        default=10,
        help="需要抓取的最近交易数量（默认 10，可传负数表示绝对值）",
    )
    parser.add_argument(
        "--fetch-only",
        action="store_true",
        help="只抓取交易 JSON，不进行账户分析",
    )

    args = parser.parse_args()
    wallet = args.wallet.strip()
    if not wallet:
        raise SystemExit("wallet pubkey is required")
    limit = abs(args.count)
    if limit == 0:
        raise SystemExit("count must be non-zero")

    root = Path(__file__).resolve().parent
    wallet_dir = root / wallet
    wallet_dir.mkdir(exist_ok=True)

    try:
        signatures = fetch_signatures(wallet, limit)
    except RuntimeError as exc:
        raise SystemExit(f"failed to fetch signatures: {exc}") from exc

    ensure_transactions(wallet_dir, signatures)

    if args.fetch_only:
        return

    labels = load_program_labels(root)
    analyse_wallet(wallet_dir, labels)


if __name__ == "__main__":
    main()
