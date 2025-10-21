#!/usr/bin/env python3
"""
输入 ZeroFi 池子 (pair) 地址，解析 swap 指令所需的核心账户。

功能：
 1. 读取 pair 账户原始数据，解析 vault / mint / authority 字段；
 2. 自动判断 Token Program（SPL v1 / Token-2022）；
 3. 可选根据 --user 计算 base / quote ATA；
 4. 支持 JSON 与人类可读两种输出。

账户顺序与合约实际 swap 指令一致：
 0 pair
 1 vault_info_base
 2 vault_base
 3 vault_info_quote
 4 vault_quote
 5 user_base_token_account
 6 user_quote_token_account
 7 swap_authority
 8 token_program
 9 sysvar_instructions
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
ZEROFI_PROGRAM_ID = "ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
TOKEN_PROGRAM_2022 = "TokenzQdSbnjHr1P1a9wYFwS6gkU1GGzLRtXju6Rjt92"

AUTHORITY_WHITELIST = (
    "Sett1ereLzRw7neSzoUSwp6vvstBkEgAgQeP6wFcw5F",
    "ELF5Z2V7ocaSnxE8cVESrKjwyydyn3kKqwPcj57ADvKm",
    "2UUgGySTVXmKFatH7pGQo84ZrzdSYF5zw9iqrGwBMuuj",
)

PAIR_OFFSETS = {
    "vault_info_base": 0x0BA0,
    "vault_base": 0x0BB8,
    "vault_info_quote": 0x0BC8,
    "vault_quote": 0x0BD8,
    "base_mint": 0x0BE8,
    "quote_mint": 0x0C18,
    "swap_authority_pda": 0x1968,
}

FLAG_OFFSET_TOKEN_2022 = 0x0791
FLAG_OFFSET_FAST_PATH = 0x079C
FLAG_OFFSET_BASE_LEFT = 0x079F

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
_B58_INDEX = {c: i for i, c in enumerate(BASE58_ALPHABET)}
ED25519_P = 2**255 - 19
ED25519_D = (-121665 * pow(121666, -1, ED25519_P)) % ED25519_P


class RpcError(RuntimeError):
    pass


def rpc_request(rpc_url: str, method: str, params: typing.List[typing.Any]) -> typing.Any:
    payload = {
        "jsonrpc": "2.0",
        "id": method,
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
    except urllib.error.URLError as exc:  # pragma: no cover - 网络依赖
        raise RpcError(f"RPC 请求失败: {exc}") from exc
    doc = json.loads(body)
    if "error" in doc:
        raise RpcError(f"{method} 返回错误: {doc['error']}")
    result = doc.get("result")
    if isinstance(result, list):
        if not result:
           return None
        if len(result) == 1:
           return result[0]
        raise RpcError(f"{method} 返回了无法解析的列表结果: {result}")
    return result


def b58decode(data: str) -> bytes:
    num = 0
    for ch in data:
        num = num * 58 + _B58_INDEX[ch]
    pad = len(data) - len(data.lstrip("1"))
    raw = num.to_bytes((num.bit_length() + 7) // 8, "big") if num else b""
    raw = b"\x00" * pad + raw
    return raw.rjust(32, b"\x00")


def b58encode(data: bytes) -> str:
    num = int.from_bytes(data, "big")
    if num == 0:
        return "1" * len(data)
    encoded = ""
    while num > 0:
        num, rem = divmod(num, 58)
        encoded = BASE58_ALPHABET[rem] + encoded
    pad = 0
    for byte in data:
        if byte == 0:
            pad += 1
        else:
            break
    return "1" * pad + encoded


def sha256(data: bytes) -> bytes:
    import hashlib

    return hashlib.sha256(data).digest()


def is_on_curve(pubkey: bytes) -> bool:
    if len(pubkey) != 32:
        return False
    y = int.from_bytes(pubkey, "little") & ((1 << 255) - 1)
    sign = pubkey[31] >> 7
    if y >= ED25519_P:
        return False
    y2 = (y * y) % ED25519_P
    u = (y2 - 1) % ED25519_P
    v = (ED25519_D * y2 + 1) % ED25519_P
    if v == 0:
        return False
    x2 = (u * pow(v, ED25519_P - 2, ED25519_P)) % ED25519_P
    x = pow(x2, (ED25519_P + 3) // 8, ED25519_P)
    if (x * x - x2) % ED25519_P != 0:
        x = (x * pow(2, (ED25519_P - 1) // 4, ED25519_P)) % ED25519_P
        if (x * x - x2) % ED25519_P != 0:
            return False
    if (x % 2) != sign:
        x = (-x) % ED25519_P
    return x != 0


def create_program_address(seeds: typing.Iterable[bytes], program_id: bytes) -> bytes:
    seeds_tuple = tuple(seeds)
    if len(seeds_tuple) > 16:
        raise ValueError("Seeds count too large")
    buffers = b"ProgramDerivedAddress"
    for seed in seeds_tuple:
        if len(seed) > 32:
            raise ValueError("Seed too long")
        buffers += seed
    buffers += program_id
    digest = sha256(buffers)
    if is_on_curve(digest):
        raise ValueError("PDA 落在曲线上")
    return digest


def find_program_address(
    seeds: typing.Iterable[bytes],
    program_id: bytes,
) -> tuple[str, int]:
    seeds_tuple = tuple(seeds)
    for bump in range(255, -1, -1):
        try:
            addr = create_program_address(seeds_tuple + (bytes([bump]),), program_id)
            return b58encode(addr), bump
        except ValueError:
            continue
    raise RuntimeError("无法找到合法 PDA")


def find_ata(
    owner: str,
    mint: str,
    token_program: str = TOKEN_PROGRAM_V1,
) -> tuple[str, int]:
    owner_bytes = b58decode(owner)
    mint_bytes = b58decode(mint)
    token_prog_bytes = b58decode(token_program)
    assoc_bytes = b58decode(ASSOCIATED_TOKEN_PROGRAM_ID)
    addr, bump = find_program_address((owner_bytes, token_prog_bytes, mint_bytes), assoc_bytes)
    return addr, bump


def fetch_pair_data(rpc_url: str, pair: str) -> bytes:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [pair, {"encoding": "base64", "commitment": "confirmed"}],
    )
    value = info.get("value")
    if not value or not value.get("data"):
        raise RpcError(f"pair {pair} 数据为空")
    data_b64, _encoding = value["data"]
    return base64.b64decode(data_b64)


def read_pubkey(data: bytes, offset: int) -> str:
    chunk = data[offset : offset + 32]
    if len(chunk) != 32:
        raise ValueError(f"偏移 {offset:#x} 读取公钥失败，长度不足")
    return b58encode(chunk)


def fetch_mint_owner(rpc_url: str, mint: str) -> tuple[str, int]:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [mint, {"encoding": "jsonParsed", "commitment": "confirmed"}],
    )
    value = info.get("value")
    if not value:
        raise RpcError(f"mint {mint} 不存在或无法解析")
    owner = value["owner"]
    parsed = value.get("data", {}).get("parsed", {})
    decimals = int(parsed.get("info", {}).get("decimals", 0))
    return owner, decimals


def decode_pair_layout(data: bytes) -> dict[str, typing.Any]:
    layout = {field: read_pubkey(data, offset) for field, offset in PAIR_OFFSETS.items()}
    uses_token2022 = bool(data[FLAG_OFFSET_TOKEN_2022] & 1)
    fast_flag = data[FLAG_OFFSET_FAST_PATH]
    base_on_left = data[FLAG_OFFSET_BASE_LEFT] == 0
    layout.update(
        {
            "uses_token2022": uses_token2022,
            "fast_flag": fast_flag,
            "base_on_left": base_on_left,
        }
    )
    return layout


def resolve_accounts(
    rpc_url: str,
    pair: str,
    user: str | None,
) -> dict[str, typing.Any]:
    data = fetch_pair_data(rpc_url, pair)
    layout = decode_pair_layout(data)

    base_mint = layout["base_mint"]
    quote_mint = layout["quote_mint"]
    mint_meta = {}

    try:
        base_owner, base_decimals = fetch_mint_owner(rpc_url, base_mint)
    except RpcError:
        base_owner, base_decimals = TOKEN_PROGRAM_V1, 0
    mint_meta["base"] = {"owner": base_owner, "decimals": base_decimals}

    try:
        quote_owner, quote_decimals = fetch_mint_owner(rpc_url, quote_mint)
    except RpcError:
        quote_owner, quote_decimals = base_owner, 0
    mint_meta["quote"] = {"owner": quote_owner, "decimals": quote_decimals}

    token_program = base_owner
    if layout["uses_token2022"]:
        token_program = TOKEN_PROGRAM_2022

    if token_program not in {TOKEN_PROGRAM_V1, TOKEN_PROGRAM_2022}:
        token_program = TOKEN_PROGRAM_V1

    if user:
        try:
            user_base_token, _ = find_ata(user, base_mint, token_program)
        except Exception as exc:  # pragma: no cover - 罕见
            raise RpcError(f"计算用户 base ATA 失败: {exc}") from exc
        try:
            user_quote_token, _ = find_ata(user, quote_mint, token_program)
        except Exception as exc:
            raise RpcError(f"计算用户 quote ATA 失败: {exc}") from exc
        user_authority = user
    else:
        user_base_token = "<user-base-token-account>"
        user_quote_token = "<user-quote-token-account>"
        user_authority = "<user-authority>"

    authority = layout["swap_authority_pda"]
    authority_warning = None
    if authority not in AUTHORITY_WHITELIST:
        authority_warning = f"解析出的 authority {authority} 不在白名单中"

    accounts = [
        ("pair", pair),
        ("vault_info_base", layout["vault_info_base"]),
        ("vault_base", layout["vault_base"]),
        ("vault_info_quote", layout["vault_info_quote"]),
        ("vault_quote", layout["vault_quote"]),
        ("user_base_token", user_base_token),
        ("user_quote_token", user_quote_token),
        ("swap_authority", authority),
        ("token_program", token_program),
        ("sysvar_instructions", SYSVAR_INSTRUCTIONS),
    ]

    return {
        "program_id": ZEROFI_PROGRAM_ID,
        "pair": pair,
        "user_authority": user_authority,
        "accounts": accounts,
        "mints": {
            "base": base_mint,
            "quote": quote_mint,
        },
        "mint_meta": mint_meta,
        "flags": {
            "uses_token2022": layout["uses_token2022"],
            "fast_flag": layout["fast_flag"],
            "base_on_left": layout["base_on_left"],
        },
        "authority_warning": authority_warning,
    }


def print_human_readable(result: dict[str, typing.Any]) -> None:
    print("ZeroFi swap 账户解析结果")
    print(f"- program_id: {result['program_id']}")
    print(f"- pair:       {result['pair']}")
    print(f"- user:       {result['user_authority']}")
    base = result["mints"]["base"]
    quote = result["mints"]["quote"]
    base_meta = result["mint_meta"]["base"]
    quote_meta = result["mint_meta"]["quote"]
    print(
        f"- base_mint:  {base} (owner: {base_meta['owner']}, decimals: {base_meta['decimals']})"
    )
    print(
        f"- quote_mint: {quote} (owner: {quote_meta['owner']}, decimals: {quote_meta['decimals']})"
    )
    flags = result["flags"]
    print(
        f"- flags: uses_token2022={flags['uses_token2022']}, "
        f"fast_flag={flags['fast_flag']}, base_on_left={flags['base_on_left']}"
    )
    if result["authority_warning"]:
        print(f"- warning: {result['authority_warning']}")
    print("\n账户顺序 (swap 指令实际顺序)：")
    for idx, (name, addr) in enumerate(result["accounts"]):
        print(f" {idx:2d} {name:18s} {addr}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="解析 ZeroFi 池子的 swap 账户")
    parser.add_argument("pair", help="ZeroFi pair 地址")
    parser.add_argument("--rpc", default=RPC_DEFAULT, help="RPC 节点 (默认: %(default)s)")
    parser.add_argument("--user", help="可选：用户签名者，用于计算 base / quote ATA")
    parser.add_argument("--json", action="store_true", help="输出 JSON 格式")
    args = parser.parse_args(argv)

    try:
        result = resolve_accounts(args.rpc, args.pair, args.user)
    except RpcError as exc:
        print(f"错误: {exc}", file=sys.stderr)
        return 1
    except Exception as exc:  # pragma: no cover - 调试辅助
        print(f"未处理异常: {exc}", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print_human_readable(result)

    if args.user:
        print("\n已根据 --user 计算 ATA，可直接用于构造 swap 指令。")
    else:
        print("\n提示：提供 --user 可直接输出用户的 ATA。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
