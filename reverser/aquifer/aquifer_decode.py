#!/usr/bin/env python3
"""
基于本地 RPC 解析 Aquifer swap 指令所需的关键账户。

已知 swap 指令只在 AccountMeta 中携带 5 个核心账户：
  0. 用户签名者（payer / authority）
  1. Dex 全局状态（长度 ~8.3KB，程序自有账户）
  2. Dex 实例状态（长度 ~8.2KB，程序自有账户）
  3. 用户状态（长度 ~1KB，程序自有账户）
  4. 交换所针对的 token mint（SPL mint）

程序其余依赖账户会在指令数据里以压缩方式传入，或由状态账户内的白名单推导。
因此脚本只需解析状态账户内容，校验它们之间的父子关系，并给出构造指令时需要
传入的 5 个 AccountMeta 顺序。

输出额外给出一些可观察字段，便于人工比对状态含义。
"""
from __future__ import annotations

import argparse
import base64
import json
import sys
import typing
import urllib.error
import urllib.request


RPC_DEFAULT = "http://127.0.0.1:8899"
AQUIFER_PROGRAM_ID = "AQU1FRd7papthgdrwPTTq5JacJh8YtwEXaBfKU3bTz45"
FAST_PROGRAM_ID = "fastC7gqs2WUXgcyNna2BZAe9mte4zcTGprv3mv18N3"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

PLACEHOLDERS = {
    "payer": "<user-signer>",
    "user_wrap_sol": "<user-wrap-sol>",
    "user_quote_token": "<user-quote-token>",
}

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


class RpcError(RuntimeError):
    pass


def b58encode(data: bytes) -> str:
    num = int.from_bytes(data, "big")
    if num == 0:
        return "1"
    encoded = ""
    while num:
        num, rem = divmod(num, 58)
        encoded = BASE58_ALPHABET[rem] + encoded
    prefix = 0
    for byte in data:
        if byte == 0:
            prefix += 1
        else:
            break
    return "1" * prefix + encoded


def b58decode(data: str) -> bytes:
    num = 0
    for char in data:
        num = num * 58 + BASE58_ALPHABET.index(char)
    raw = num.to_bytes(32, "big")
    pad = 0
    for char in data:
        if char == "1":
            pad += 1
        else:
            break
    return b"\x00" * pad + raw[len(raw) - 32 :]


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
            raw = resp.read()
    except urllib.error.URLError as exc:  # pragma: no cover
        raise RpcError(f"RPC 请求失败: {exc}") from exc
    result = json.loads(raw)
    if "error" in result:
        raise RpcError(f"{method} 调用失败: {result['error']}")
    return result["result"]


def fetch_account_bytes(rpc_url: str, pubkey: str) -> bytes:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            pubkey,
            {"encoding": "base64", "commitment": "confirmed"},
        ],
    )
    value = info.get("value")
    if not value:
        raise RpcError(f"账户 {pubkey} 不存在或无数据")
    data_b64, _encoding = value["data"]
    return base64.b64decode(data_b64)


def list_program_accounts(
    rpc_url: str,
    program_id: str,
    *,
    data_size: typing.Optional[int] = None,
) -> typing.List[dict[str, typing.Any]]:
    params_filter: dict[str, typing.Any] = {"encoding": "base64", "commitment": "confirmed"}
    if data_size is not None:
        params_filter["filters"] = [{"dataSize": data_size}]
    result = rpc_request(
        rpc_url,
        "getProgramAccounts",
        [
            program_id,
            params_filter,
        ],
    )
    return typing.cast(typing.List[dict[str, typing.Any]], result)


def chunk32(data: bytes, count: int) -> typing.List[str]:
    out: typing.List[str] = []
    for idx in range(count):
        start = idx * 32
        end = start + 32
        if end > len(data):
            break
        out.append(b58encode(data[start:end]))
    return out


def parse_fast_account(raw: bytes) -> dict[str, typing.Any]:
    if len(raw) != 128:
        raise RpcError(f"fast state 长度异常: {len(raw)}")
    mint = b58encode(raw[24:56])
    dex = b58encode(raw[56:88])
    global_state = b58encode(raw[88:120])
    vault_info_ptr = b58encode(raw[32:64])
    index = int.from_bytes(raw[0:8], "little")
    bump = int.from_bytes(raw[8:12], "little")
    return {
        "raw_len": len(raw),
        "mint": mint,
        "dex": dex,
        "global_state": global_state,
        "vault_info": vault_info_ptr,
        "index": index,
        "bump": bump,
    }


def parse_vault_info(raw: bytes) -> dict[str, typing.Any]:
    if len(raw) != 1056:
        raise RpcError(f"vault info 长度异常: {len(raw)}")
    mint = b58encode(raw[952:984])
    instance = b58encode(raw[1016:1048])
    fast_pointer = b58encode(raw[960:992])
    return {
        "raw_len": len(raw),
        "mint": mint,
        "instance": instance,
        "fast_pointer": fast_pointer,
    }


def fetch_token_account(
    rpc_url: str,
    owner: str,
    expected_mint: str,
) -> tuple[str, dict[str, typing.Any]]:
    result = rpc_request(
        rpc_url,
        "getTokenAccountsByOwner",
        [
            owner,
            {"programId": TOKEN_PROGRAM_ID},
            {
                "encoding": "jsonParsed",
                "commitment": "confirmed",
            },
        ],
    )
    entries = typing.cast(typing.List[typing.Any], result.get("value") or [])
    if not entries:
        raise RpcError(f"找不到 {owner} 的 SPL Token 账户")

    best_entry: typing.Optional[dict[str, typing.Any]] = None
    best_amount = -1
    for entry in entries:
        parsed = (
            entry.get("account", {})
            .get("data", {})
            .get("parsed", {})
            .get("info", {})
        )
        mint = parsed.get("mint")
        if mint != expected_mint:
            continue
        token_amount = parsed.get("tokenAmount", {})
        amount_raw = token_amount.get("amount")
        try:
            amount = int(amount_raw)
        except (TypeError, ValueError):
            amount = 0
        if amount >= best_amount:
            best_amount = amount
            best_entry = entry
    if not best_entry:
        raise RpcError(
            f"{owner} 未找到 mint={expected_mint} 的 Token 账户"
        )
    info = (
        best_entry["account"]["data"]["parsed"]["info"]
    )
    info = typing.cast(dict[str, typing.Any], info)
    return best_entry["pubkey"], info


def resolve_fast_states(
    rpc_url: str,
    dex: str,
    base_mint: str,
    quote_mint: str,
) -> dict[str, dict[str, typing.Any]]:
    accounts = list_program_accounts(
        rpc_url,
        FAST_PROGRAM_ID,
        data_size=128,
    )
    targets = {"base": None, "quote": None}
    for entry in accounts:
        data_b64 = entry.get("account", {}).get("data", [])
        if not data_b64:
            continue
        raw = base64.b64decode(data_b64[0])
        parsed = parse_fast_account(raw)
        if parsed["dex"] != dex:
            continue
        if parsed["mint"] == base_mint and targets["base"] is None:
            targets["base"] = {
                **parsed,
                "pubkey": entry["pubkey"],
            }
        elif parsed["mint"] == quote_mint and targets["quote"] is None:
            targets["quote"] = {
                **parsed,
                "pubkey": entry["pubkey"],
            }
        if targets["base"] and targets["quote"]:
            break
    missing = [key for key, value in targets.items() if value is None]
    if missing:
        raise RpcError(f"未找到 fast state: {', '.join(missing)}")
    return typing.cast(dict[str, dict[str, typing.Any]], targets)


def resolve_vault_infos(
    rpc_url: str,
    instance: str,
    base_mint: str,
    quote_mint: str,
) -> dict[str, dict[str, typing.Any]]:
    accounts = list_program_accounts(
        rpc_url,
        AQUIFER_PROGRAM_ID,
        data_size=1056,
    )
    targets: dict[str, typing.Optional[dict[str, typing.Any]]] = {
        "base": None,
        "quote": None,
    }
    for entry in accounts:
        data_b64 = entry.get("account", {}).get("data", [])
        if not data_b64:
            continue
        raw = base64.b64decode(data_b64[0])
        parsed = parse_vault_info(raw)
        if parsed["instance"] != instance:
            continue
        if parsed["mint"] == base_mint and targets["base"] is None:
            targets["base"] = {
                **parsed,
                "pubkey": entry["pubkey"],
            }
        elif parsed["mint"] == quote_mint and targets["quote"] is None:
            targets["quote"] = {
                **parsed,
                "pubkey": entry["pubkey"],
            }
        if targets["base"] and targets["quote"]:
            break
    missing = [key for key, value in targets.items() if value is None]
    if missing:
        raise RpcError(f"未找到 vault info: {', '.join(missing)}")
    return typing.cast(dict[str, dict[str, typing.Any]], targets)


def summarise_dex(data: bytes) -> dict[str, typing.Any]:
    version = int.from_bytes(data[0:8], "little")
    admin = b58encode(data[8:40])
    fields = chunk32(data[40:], 6)
    return {
        "raw_len": len(data),
        "version": version,
        "admin_key": admin,
        "field32_starting_from_40": fields,
    }


def summarise_instance(data: bytes) -> dict[str, typing.Any]:
    parent = b58encode(data[0:32])
    anchors = chunk32(data[32:], 10)
    return {
        "raw_len": len(data),
        "parent_dex": parent,
        "leading_pubkeys": anchors,
    }


def summarise_user(data: bytes) -> dict[str, typing.Any]:
    words = chunk32(data, 6)
    return {
        "raw_len": len(data),
        "leading_pubkeys": words,
        "trailing_pubkey": b58encode(data[-32:]),
    }


def build_output(
    rpc_url: str,
    dex: str,
    instance: str,
    user: str | None,
    signer: str | None,
    base_mint: str,
    quote_mint: str,
    user_wrap_sol: str | None,
    user_quote_token: str | None,
) -> dict[str, typing.Any]:
    dex_bytes = fetch_account_bytes(rpc_url, dex)
    instance_bytes = fetch_account_bytes(rpc_url, instance)
    user_bytes = fetch_account_bytes(rpc_url, user) if user else None
    fast_states = resolve_fast_states(rpc_url, dex, base_mint, quote_mint)
    vault_infos = resolve_vault_infos(rpc_url, instance, base_mint, quote_mint)
    base_vault_token, base_token_info = fetch_token_account(
        rpc_url,
        vault_infos["base"]["pubkey"],
        base_mint,
    )
    quote_vault_token, quote_token_info = fetch_token_account(
        rpc_url,
        vault_infos["quote"]["pubkey"],
        quote_mint,
    )
    vault_infos["base"]["token_account"] = base_vault_token
    vault_infos["base"]["token_info"] = base_token_info
    vault_infos["quote"]["token_account"] = quote_vault_token
    vault_infos["quote"]["token_info"] = quote_token_info

    core_accounts = [
        ("payer", signer or "<user-signer>"),
        ("dex_global", dex),
        ("dex_instance", instance),
        ("user_state", user or "<user-account>"),
        ("swap_mint", base_mint),
    ]

    summary = {
        "program_id": AQUIFER_PROGRAM_ID,
        "swap_accounts_order": core_accounts,
        "dex_summary": summarise_dex(dex_bytes),
        "instance_summary": summarise_instance(instance_bytes),
        "base_mint": base_mint,
        "quote_mint": quote_mint,
        "coin_states": fast_states,
        "vault_infos": vault_infos,
    }
    if user_bytes is not None:
        summary["user_summary"] = summarise_user(user_bytes)

    ordered_accounts = [
        ("sysvar_instructions", SYSVAR_INSTRUCTIONS),
        ("payer", signer or PLACEHOLDERS["payer"]),
        ("token_program_base", TOKEN_PROGRAM_ID),
        ("user_wrap_sol", user_wrap_sol or PLACEHOLDERS["user_wrap_sol"]),
        ("base_mint", base_mint),
        ("token_program_quote", TOKEN_PROGRAM_ID),
        ("user_quote_token", user_quote_token or PLACEHOLDERS["user_quote_token"]),
        ("quote_mint", quote_mint),
        ("dex_global", dex),
        ("dex_instance", instance),
        ("coin_state_base", fast_states["base"]["pubkey"]),
        ("coin_state_quote", fast_states["quote"]["pubkey"]),
        ("base_vault_pda", vault_infos["base"]["pubkey"]),
        ("base_vault_token", base_vault_token),
        ("quote_vault_pda", vault_infos["quote"]["pubkey"]),
        ("quote_vault_token", quote_vault_token),
    ]

    summary["full_account_list"] = ordered_accounts
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="Aquifer swap 账户解析")
    parser.add_argument("dex", help="Dex 全局状态账户地址")
    parser.add_argument("instance", help="Dex 实例账户地址")
    parser.add_argument(
        "--user",
        help="用户状态账户地址（可选，如未提供则输出占位符）",
    )
    parser.add_argument(
        "--signer",
        help="用户签名者地址（可选，默认输出占位符）",
    )
    parser.add_argument(
        "--mint",
        help="swap 使用的 token mint（可选，默认输出占位符）",
    )
    parser.add_argument("--base-mint", help="base 侧 mint (例如 wSOL)")
    parser.add_argument("--quote-mint", help="quote 侧 mint (例如 USDC)")
    parser.add_argument("--user-wrap-sol", help="用户 base token 账户（可选，占位）")
    parser.add_argument("--user-quote-token", help="用户 quote token 账户（可选，占位）")
    parser.add_argument(
        "--rpc",
        default=RPC_DEFAULT,
        help=f"Solana RPC 地址（默认 {RPC_DEFAULT}）",
    )
    args = parser.parse_args()

    try:
        base_mint = args.base_mint or args.mint
        if not base_mint:
            raise RpcError("请通过 --base-mint 指定 base 侧 mint")
        if not args.quote_mint:
            raise RpcError("请通过 --quote-mint 指定 quote 侧 mint")

        result = build_output(
            rpc_url=args.rpc,
            dex=args.dex,
            instance=args.instance,
            user=args.user,
            signer=args.signer,
            base_mint=base_mint,
            quote_mint=args.quote_mint,
            user_wrap_sol=args.user_wrap_sol,
            user_quote_token=args.user_quote_token,
        )
    except RpcError as exc:
        print(f"错误: {exc}", file=sys.stderr)
        sys.exit(1)

    json.dump(result, sys.stdout, indent=2, ensure_ascii=False)
    print()

    accounts = result.get("full_account_list", [])
    if accounts:
        print("\n账户顺序 (swap 指令实际顺序)：")
        max_label_len = max(len(label) for label, _addr in accounts)
        for idx, (label, addr) in enumerate(accounts):
            padded = label.ljust(max_label_len)
            print(f"  {idx:<2d} {padded} {addr}")


if __name__ == "__main__":
    main()
