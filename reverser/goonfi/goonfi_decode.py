#!/usr/bin/env python3
"""
输入 GoonFi 池子地址，尽量还原 swap 所需账户信息。

已知约束：
  * 入口固定要求 global_state / pool_state / Sysvar Instructions / Token Program。
  * 池子数据的 0x100/0x120/0x140/0x160 分别存放 base/quote mint 与 vault。
  * 其余账户仍需通过汇编或链上样本佐证，本脚本输出偏移 + 所属程序，便于人工核对。
"""
from __future__ import annotations

import argparse
import base64
import json
import sys
import typing
import urllib.error
import urllib.request

from goonfi_seeds import RouterProgram, derive_swap_authority_address

from goonfi_utils import (
    b58decode,
    b58encode,
    find_program_address,
)


RPC_DEFAULT = "http://127.0.0.1:8899"
GOONFI_PROGRAM_ID = "goonERTdGsjnkZqWuVjs73BZ3Pb9qoCUdBUL17BnS5j"
GOONFI_GLOBAL_STATE = "updapqBoqhn48uaVxD7oKyFVEwEcHmqbgQa1GvHaUuX"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
TOKEN_PROGRAM_2022 = "TokenzQdBz3aJQezpLJrWcRkLmW5AoWzLFf5Z4xJ9zQ"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"

POOL_GLOBAL_STATE_OFFSET = 0x0E0
POOL_BASE_MINT_OFFSET = 0x100
POOL_QUOTE_MINT_OFFSET = 0x120
POOL_BASE_VAULT_OFFSET = 0x140
POOL_QUOTE_VAULT_OFFSET = 0x160
POOL_ROUTER_FLAG_OFFSET = 0x388
POOL_BLACKLIST_FLAG_OFFSET = 0x38E

KNOWN_ROUTER_PROGRAMS = {
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4": "JupiterV6",
    "6m2CDdhRgxpH4WjvdzxAYbGxwdGUz5MziiL5jek2kBma": "StepAggregator",
    "T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT": "GoonBlacklist",
    "2gav97pP6WnmsZYStGmeX4wUmJgtsUHzhX7dhjqBBZa8": "BlacklistVault",
}

OPENBOOK_PROGRAM_IDS = {
    # Serum v3 / OpenBook 常见 program id 列表，可按需扩充。
    "9xQeWvG816bUx9EPfYaz7828gGvvmtyf4smyuQ5VDnDX",
    "sp3uGft1tXH6145iRhV8JdzS34rnDgYUdr9bS5NjZPa",  # mainnet OpenBook
}

class RpcError(RuntimeError):
    pass


def rpc_request(rpc_url: str, method: str, params: typing.List[typing.Any]) -> typing.Any:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        rpc_url,
        data=data,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req) as resp:
            body = resp.read()
    except urllib.error.URLError as exc:
        raise RpcError(f"RPC 请求失败: {exc}") from exc
    result = json.loads(body)
    if "error" in result:
        raise RpcError(f"{method} 调用失败: {result['error']}")
    return result["result"]


def rpc_get_multiple_accounts(
    rpc_url: str,
    pubkeys: typing.Sequence[str],
    *,
    encoding: str = "base64",
    commitment: str = "confirmed",
    chunk_size: int = 100,
) -> typing.List[typing.Optional[typing.Dict[str, typing.Any]]]:
    if not pubkeys:
        return []
    aggregated: list[typing.Optional[dict[str, typing.Any]]] = []
    for idx in range(0, len(pubkeys), chunk_size):
        chunk = pubkeys[idx : idx + chunk_size]
        response = rpc_request(
            rpc_url,
            "getMultipleAccounts",
            [
                list(chunk),
                {
                    "encoding": encoding,
                    "commitment": commitment,
                },
            ],
        )
        values = response.get("value") or []
        if len(values) < len(chunk):
            values.extend([None] * (len(chunk) - len(values)))
        aggregated.extend(values[: len(chunk)])
    return aggregated


def find_ata(
    owner: str,
    mint: str,
    token_program: str = TOKEN_PROGRAM_V1,
) -> tuple[str, int]:
    owner_bytes = b58decode(owner)
    mint_bytes = b58decode(mint)
    token_prog_bytes = b58decode(token_program)
    assoc_bytes = b58decode(ASSOCIATED_TOKEN_PROGRAM_ID)
    addr, bump = find_program_address(
        (owner_bytes, token_prog_bytes, mint_bytes),
        assoc_bytes,
    )
    return addr, bump



def read_pubkey(data: bytes, offset: int) -> str:
    chunk = data[offset : offset + 32]
    if len(chunk) != 32:
        raise ValueError(f"偏移 {offset:#x} 超出范围")
    if all(b == 0 for b in chunk):
        return "11111111111111111111111111111111"
    return b58encode(chunk)


def scan_pubkeys(
    data: bytes,
    start: int,
    end: int,
) -> typing.List[tuple[int, str]]:
    results: list[tuple[int, str]] = []
    seen: set[str] = set()
    offset = start
    while offset + 32 <= min(len(data), end):
        chunk = data[offset : offset + 32]
        if len(chunk) == 32 and any(chunk):
            key = b58encode(chunk)
            if key not in seen:
                results.append((offset, key))
                seen.add(key)
        offset += 32
    return results


def classify_account_meta(
    info: typing.Optional[dict[str, typing.Any]],
) -> dict[str, typing.Any]:
    if info is None:
        return {
            "exists": False,
            "owner": None,
            "type": "uninitialized-or-pda",
        }
    owner = info.get("owner")
    program = None
    parsed_type = None
    data_entry = info.get("data")
    if isinstance(data_entry, dict):
        program = data_entry.get("program")
        parsed = data_entry.get("parsed") or {}
        parsed_type = parsed.get("type")
    classification = "unknown"
    if program == "spl-token":
        if parsed_type == "account":
            classification = "spl-token-account"
        elif parsed_type == "mint":
            classification = "spl-mint"
    elif owner in OPENBOOK_PROGRAM_IDS:
        classification = "openbook-market"
    elif owner == GOONFI_PROGRAM_ID:
        classification = "goonfi-state"
    elif owner == "11111111111111111111111111111111":
        classification = "system-account"
    return {
        "exists": True,
        "owner": owner,
        "program": program,
        "parsed_type": parsed_type,
        "classification": classification,
    }


def extract_vault_details(
    account_info: typing.Optional[dict[str, typing.Any]]
) -> dict[str, typing.Any]:
    meta = classify_account_meta(account_info)
    mint = None
    owner = None
    decimals: typing.Optional[int] = None
    if account_info and (data := account_info.get("data")) and isinstance(data, dict):
        parsed = data.get("parsed") or {}
        if parsed.get("type") == "account":
            parsed_info = parsed.get("info") or {}
            mint = parsed_info.get("mint")
            owner = parsed_info.get("owner")
            token_amount = parsed_info.get("tokenAmount") or {}
            decimals_raw = token_amount.get("decimals")
            if decimals_raw is not None:
                try:
                    decimals = int(decimals_raw)
                except (TypeError, ValueError):
                    decimals = None
    return {
        "meta": meta,
        "mint": mint,
        "owner": owner,
        "decimals": decimals,
    }


def extract_mint_owner_and_decimals(
    mint_pubkey: str,
    account_info: typing.Optional[dict[str, typing.Any]],
) -> tuple[str, int]:
    if not account_info:
        raise RpcError(f"mint {mint_pubkey} 不存在")
    owner = account_info.get("owner")
    data = account_info.get("data") or {}
    parsed = data.get("parsed") or {}
    decimals = parsed.get("info", {}).get("decimals")
    if decimals is None:
        raise RpcError(f"mint {mint_pubkey} 缺少 decimals")
    try:
        decimals_int = int(decimals)
    except (TypeError, ValueError) as exc:
        raise RpcError(f"mint {mint_pubkey} 的 decimals 字段异常: {decimals}") from exc
    return owner, decimals_int


def parse_pool(
    rpc_url: str,
    pool: str,
    user: str | None,
) -> dict[str, typing.Any]:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            pool,
            {
                "encoding": "base64",
                "commitment": "confirmed",
            },
        ],
    )
    value = info.get("value")
    if not value or not value.get("data"):
        raise RpcError(f"池子 {pool} 数据为空")
    data_b64, _encoding = value["data"]
    raw = base64.b64decode(data_b64)
    if len(raw) < 0x1b8 + 32:
        raise RpcError(
            f"池子 {pool} 数据长度不足以解析核心字段: {len(raw)} 字节"
        )

    global_state_candidate = read_pubkey(raw, POOL_GLOBAL_STATE_OFFSET)
    base_mint = read_pubkey(raw, POOL_BASE_MINT_OFFSET)
    quote_mint = read_pubkey(raw, POOL_QUOTE_MINT_OFFSET)
    base_vault = read_pubkey(raw, POOL_BASE_VAULT_OFFSET)
    quote_vault = read_pubkey(raw, POOL_QUOTE_VAULT_OFFSET)

    router_flag = (
        raw[POOL_ROUTER_FLAG_OFFSET] if len(raw) > POOL_ROUTER_FLAG_OFFSET else None
    )
    blacklist_flag = (
        raw[POOL_BLACKLIST_FLAG_OFFSET] if len(raw) > POOL_BLACKLIST_FLAG_OFFSET else None
    )

    account_cache: dict[str, typing.Optional[dict[str, typing.Any]]] = {}

    def cache_accounts(
        pubkeys: typing.Sequence[str],
        *,
        encoding: str,
    ) -> None:
        pending: list[str] = []
        for key in pubkeys:
            if key and key not in account_cache:
                pending.append(key)
        if not pending:
            return
        infos = rpc_get_multiple_accounts(
            rpc_url,
            pending,
            encoding=encoding,
        )
        if len(infos) != len(pending):
            raise RpcError("getMultipleAccounts 返回长度与请求不一致")
        for key, info in zip(pending, infos):
            account_cache[key] = info

    if not base_mint or not quote_mint:
        raise RpcError("无法从 vault 解析出 base/quote mint")

    cache_accounts(
        [base_vault, quote_vault, base_mint, quote_mint],
        encoding="jsonParsed",
    )

    base_details = extract_vault_details(account_cache.get(base_vault))
    quote_details = extract_vault_details(account_cache.get(quote_vault))

    # 基本 sanity 校验：vault owner 应与池子解析出的 PDA 匹配
    vault_authority = base_details["owner"] if base_details["owner"] else None

    base_token_program, base_decimals = extract_mint_owner_and_decimals(
        base_mint, account_cache.get(base_mint)
    )
    quote_token_program, quote_decimals = extract_mint_owner_and_decimals(
        quote_mint, account_cache.get(quote_mint)
    )

    user_authority = user or "<user-authority>"
    if user:
        try:
            user_base_token, _ = find_ata(user, base_mint, base_token_program)
        except Exception as exc:  # pragma: no cover - 极少触发
            raise RpcError(f"计算用户 base ATA 失败: {exc}") from exc
        try:
            user_quote_token, _ = find_ata(user, quote_mint, quote_token_program)
        except Exception as exc:
            raise RpcError(f"计算用户 quote ATA 失败: {exc}") from exc
    else:
        user_base_token = "<user-base-token-account>"
        user_quote_token = "<user-quote-token-account>"

    extra_pubkeys = scan_pubkeys(raw, POOL_QUOTE_VAULT_OFFSET + 32, len(raw))
    cache_accounts(
        [pubkey for _, pubkey in extra_pubkeys],
        encoding="base64",
    )
    extra_entries: list[dict[str, typing.Any]] = []

    router_program: RouterProgram | None = None
    if router_flag is None:
        # 早期版本 pool_state 长度不足以覆盖 0x388 router flag，默认走 Jupiter 分支
        router_program = RouterProgram.JUPITER_V6
    elif router_flag == 0:
        router_program = RouterProgram.JUPITER_V6
    elif router_flag == 1:
        router_program = RouterProgram.STEP_AGGREGATOR
    elif router_flag == 2:
        router_program = RouterProgram.GOON_BLACKLIST

    pool_signer_detail: dict[str, typing.Any] | None = None
    pool_signer_addr: str | None = None
    if router_program is not None:
        try:
            swap_addr, swap_bump, seeds_used = derive_swap_authority_address(
                pool,
                raw,
                router_program,
            )
            pool_signer_addr = swap_addr
            pool_signer_detail = {
                "address": swap_addr,
                "bump": swap_bump,
                "router": router_program.value,
                "seeds_hex": [seed.hex() for seed in seeds_used],
            }
        except Exception as exc:  # pragma: no cover - 逆向仍进行中
            pool_signer_detail = {
                "error": f"swap authority 解析失败: {exc}",
                "router": router_program.value,
            }
    for offset, pubkey in extra_pubkeys:
        info = account_cache.get(pubkey)
        meta = classify_account_meta(info)
        extra_entries.append(
            {
                "offset": f"0x{offset:03x}",
                "pubkey": pubkey,
                "owner": info.get("owner") if info else None,
                "classification": meta.get("classification"),
                "exists": meta["exists"],
            }
        )

    notes: list[str] = [
        "extra_pubkeys 列出 quote vault 之后的所有 32 字节段，若指向有效账户可进一步标注含义。",
        "global_state_in_data 用于 sanity check；按照当前实现应当等于常量 GOONFI_GLOBAL_STATE。",
        "vault_authority 来源于 base vault 的 owner，可用于核对 PDA。",
    ]
    if pool_signer_detail and pool_signer_detail.get("error"):
        notes.append(pool_signer_detail["error"])

    return {
        "program_id": GOONFI_PROGRAM_ID,
        "global_state": GOONFI_GLOBAL_STATE,
        "pool_state": pool,
        "pool_header": {
            "discriminator": raw[:8].hex(),
            "router_flag": router_flag,
            "blacklist_flag": blacklist_flag,
            "global_state_in_data": global_state_candidate,
            "raw_len": len(raw),
        },
        "core_accounts": {
            "global_state": GOONFI_GLOBAL_STATE,
            "pool_state": pool,
            "user_authority": user_authority,
            "base_vault": base_vault,
            "quote_vault": quote_vault,
            "sysvar_instructions": SYSVAR_INSTRUCTIONS,
            "base_mint": base_mint,
            "quote_mint": quote_mint,
            "base_token_program": base_token_program,
            "quote_token_program": quote_token_program,
            "user_base_token": user_base_token,
            "user_quote_token": user_quote_token,
            "vault_authority": vault_authority,
            "pool_signer": pool_signer_addr,
        },
        "base_vault_meta": base_details,
        "quote_vault_meta": quote_details,
        "mint_decimals": {
            "base": base_decimals,
            "quote": quote_decimals,
        },
        "extra_pubkeys": extra_entries,
        "router_program": router_program.value if router_program else None,
        "derived_pool_signer": pool_signer_detail,
        "notes": notes,
    }


def print_human_summary(result: dict[str, typing.Any]) -> None:
    core = result.get("core_accounts", {})
    order = [
        ("0", "user_authority", core.get("user_authority")),
        ("1", "pool_state", core.get("pool_state")),
        ("2", "user_base_token", core.get("user_base_token")),
        ("3", "user_quote_token", core.get("user_quote_token")),
        ("4", "base_vault", core.get("base_vault")),
        ("5", "quote_vault", core.get("quote_vault")),
        ("6", "pool_signer", core.get("pool_signer") or "<未解析>"),
        ("7", "sysvar_instructions", core.get("sysvar_instructions")),
        ("8", "token_program", core.get("base_token_program")),
    ]
    print()
    print("账户顺序 (swap 指令参考顺序)：")
    for idx, label, value in order:
        print(f"  {idx} {label:<18} {value}")


def main(argv: typing.Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="GoonFi 池子账户解析")
    parser.add_argument("pool", help="池子地址")
    parser.add_argument("--rpc", default=RPC_DEFAULT, help="RPC 终端 (默认: %(default)s)")
    parser.add_argument("--user", help="可选，用户 authority 地址，用于推导 ATA")
    args = parser.parse_args(argv)
    try:
        result = parse_pool(args.rpc, args.pool, args.user)
    except RpcError as exc:
        print(f"错误: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    print_human_summary(result)
    return 0


if __name__ == "__main__":  # pragma: no cover - CLI
    raise SystemExit(main(sys.argv[1:]))
