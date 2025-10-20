#!/usr/bin/env python3
"""
输入 Tessera V 池子地址，解析 swap 所需的核心账户。

流程参考 loss_program/client/tessera_v_swap.go：
  1. pool data 提供 base/quote mint。
  2. mint owner 即 token program（Token Program v1 或 Token-2022）。
  3. vault 通过查询 Tessera 全局状态（8ek...）名下、指定 mint 的 SPL Token 账户，
     取余额最大的那个作为池金库。
  4. 如用户额外提供 signer 地址，可一并计算其 base/quote ATA，方便直接构造指令。

输出账户顺序与实际 swap 指令一致：
  0 global_state
  1 pool_state
  2 user_authority (可选，若未提供则以 "<user-signature>" 占位)
  3 base_vault
  4 quote_vault
  5 user_base_token_account (占位或计算出的 ATA)
  6 user_quote_token_account
  7 base_mint
  8 quote_mint
  9 base_token_program
 10 quote_token_program
 11 sysvar_instructions
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
TESERA_PROGRAM_ID = "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH"
TESERA_GLOBAL_STATE = "8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
ED25519_P = 2**255 - 19
ED25519_D = (-121665 * pow(121666, -1, ED25519_P)) % ED25519_P


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
    except urllib.error.URLError as exc:  # pragma: no cover - 环境依赖
        raise RpcError(f"RPC 请求失败: {exc}") from exc
    result = json.loads(body)
    if "error" in result:
        raise RpcError(f"{method} 调用失败: {result['error']}")
    return result["result"]


def b58encode(data: bytes) -> str:
    num = int.from_bytes(data, "big")
    if num == 0:
        return "1"
    encoded = ""
    while num > 0:
        num, rem = divmod(num, 58)
        encoded = BASE58_ALPHABET[rem] + encoded
    leading_zero = 0
    for byte in data:
        if byte == 0:
            leading_zero += 1
        else:
            break
    return "1" * leading_zero + encoded


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
    return b"\x00" * pad + raw[len(raw) - (32) :]


def fetch_pool_mints(rpc_url: str, pool: str) -> tuple[str, str, int]:
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
    if len(raw) < 0x38 + 32:
        raise RpcError(f"池子 {pool} 数据长度异常: {len(raw)}")
    base_mint = b58encode(raw[0x18 : 0x18 + 32])
    quote_mint = b58encode(raw[0x38 : 0x38 + 32])
    pool_id = raw[0x10]
    return base_mint, quote_mint, pool_id


def fetch_mint_owner_and_decimals(
    rpc_url: str,
    mint: str,
) -> tuple[str, int]:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            mint,
            {
                "encoding": "jsonParsed",
                "commitment": "confirmed",
            },
        ],
    )
    value = info.get("value")
    if not value:
        raise RpcError(f"mint {mint} 不存在")
    owner = value.get("owner")
    parsed = value.get("data", {}).get("parsed", {})
    decimals = parsed.get("info", {}).get("decimals")
    if decimals is None:
        raise RpcError(f"mint {mint} 缺少 decimals")
    return owner, int(decimals)


def fetch_vault(
    rpc_url: str,
    mint: str,
    owner: str,
) -> tuple[str, int, typing.Optional[int]]:
    resp = rpc_request(
        rpc_url,
        "getTokenAccountsByOwner",
        [
            owner,
            {"mint": mint},
            {
                "encoding": "jsonParsed",
                "commitment": "confirmed",
            },
        ],
    )
    values = resp.get("value") or []
    best_pubkey = None
    best_amount = -1
    decimals: typing.Optional[int] = None
    for entry in values:
        pubkey = entry.get("pubkey")
        parsed = (
            entry.get("account", {})
            .get("data", {})
            .get("parsed", {})
        )
        info = parsed.get("info", {})
        token_amount = info.get("tokenAmount", {})
        amount_raw = token_amount.get("amount")
        if amount_raw is None:
            continue
        try:
            amount = int(amount_raw)
        except ValueError:
            continue
        if amount >= best_amount:
            best_amount = amount
            best_pubkey = pubkey
            decimals = token_amount.get("decimals")
    if best_pubkey is None:
        raise RpcError(f"{owner} 未持有 mint {mint} 对应的 vault")
    return best_pubkey, best_amount, int(decimals) if decimals is not None else None


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
    return not (x == 0 and sign == 1)


def create_program_address(
    seeds: typing.Iterable[bytes],
    program_id: bytes,
) -> bytes:
    hasher = __import__("hashlib").sha256()
    for seed in seeds:
        if len(seed) > 32:
            raise ValueError("seed 长度超过 32 字节")
        hasher.update(seed)
    hasher.update(program_id)
    hasher.update(b"ProgramDerivedAddress")
    digest = hasher.digest()
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
            addr = create_program_address(
                seeds_tuple + (bytes([bump]),),
                program_id,
            )
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
    addr, bump = find_program_address(
        (owner_bytes, token_prog_bytes, mint_bytes),
        assoc_bytes,
    )
    return addr, bump


def resolve_accounts(
    rpc_url: str,
    pool: str,
    user: str | None,
) -> dict[str, typing.Any]:
    base_mint, quote_mint, pool_id = fetch_pool_mints(rpc_url, pool)
    base_token_program, base_decimals = fetch_mint_owner_and_decimals(rpc_url, base_mint)
    quote_token_program, quote_decimals = fetch_mint_owner_and_decimals(rpc_url, quote_mint)

    base_vault, base_vault_amount, base_vault_decimals = fetch_vault(
        rpc_url, base_mint, TESERA_GLOBAL_STATE
    )
    quote_vault, quote_vault_amount, quote_vault_decimals = fetch_vault(
        rpc_url, quote_mint, TESERA_GLOBAL_STATE
    )

    user_authority = user or "<user-authority>"
    if user:
        try:
            user_base_token, _ = find_ata(user, base_mint, base_token_program)
        except Exception as exc:  # pragma: no cover - rare
            raise RpcError(f"计算用户 base ATA 失败: {exc}") from exc
        try:
            user_quote_token, _ = find_ata(user, quote_mint, quote_token_program)
        except Exception as exc:
            raise RpcError(f"计算用户 quote ATA 失败: {exc}") from exc
    else:
        user_base_token = "<user-base-token-account>"
        user_quote_token = "<user-quote-token-account>"

    return {
        "program_id": TESERA_PROGRAM_ID,
        "global_state": TESERA_GLOBAL_STATE,
        "pool_state": pool,
        "pool_id": pool_id,
        "accounts": {
            "global_state": TESERA_GLOBAL_STATE,
            "pool_state": pool,
            "user_authority": user_authority,
            "base_vault": base_vault,
            "quote_vault": quote_vault,
            "user_base_token": user_base_token,
            "user_quote_token": user_quote_token,
            "base_mint": base_mint,
            "quote_mint": quote_mint,
            "base_token_program": base_token_program,
            "quote_token_program": quote_token_program,
            "sysvar_instructions": SYSVAR_INSTRUCTIONS,
        },
        "mint_decimals": {
            "base": base_decimals,
            "quote": quote_decimals,
        },
        "vault_metadata": {
            "base_vault_amount": base_vault_amount,
            "base_vault_decimals": base_vault_decimals,
            "quote_vault_amount": quote_vault_amount,
            "quote_vault_decimals": quote_vault_decimals,
        },
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="解析 Tessera V 池子的 swap 账户"
    )
    parser.add_argument("pool", help="池子地址")
    parser.add_argument(
        "--rpc",
        default=RPC_DEFAULT,
        help="RPC 节点 (默认: %(default)s)",
    )
    parser.add_argument(
        "--user",
        help="可选：用户签名者，用于计算自己的 ATA",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="输出 JSON 格式",
    )
    args = parser.parse_args(argv)

    try:
        report = resolve_accounts(args.rpc, args.pool, args.user)
    except (RpcError, ValueError, RuntimeError) as exc:
        print(f"解析失败: {exc}", file=sys.stderr)
        return 1

    if args.json:
        json.dump(report, sys.stdout, ensure_ascii=False, indent=2)
        print()
        return 0

    accounts = report["accounts"]
    print("=== Tessera V Swap 账户 ===")
    print(f"Program ID          : {report['program_id']}")
    print(f"Global State        : {report['global_state']}")
    print(f"Pool State          : {report['pool_state']} (pool_id={report['pool_id']})")
    print()
    print("账户顺序：")
    for idx, key in enumerate(
        [
            "global_state",
            "pool_state",
            "user_authority",
            "base_vault",
            "quote_vault",
            "user_base_token",
            "user_quote_token",
            "base_mint",
            "quote_mint",
            "base_token_program",
            "quote_token_program",
            "sysvar_instructions",
        ]
    ):
        print(f"{idx:2d}. {key:<21} {accounts[key]}")

    decimals = report["mint_decimals"]
    print("\nMint 精度：")
    print(f"  base ({accounts['base_mint']})  -> {decimals['base']} decimals")
    print(f"  quote({accounts['quote_mint']}) -> {decimals['quote']} decimals")

    vault_meta = report["vault_metadata"]
    print("\nVault 余额 (raw amount)：")
    print(
        f"  base_vault  ({accounts['base_vault']})  amount={vault_meta['base_vault_amount']} "
        f"decimals={vault_meta['base_vault_decimals']}"
    )
    print(
        f"  quote_vault ({accounts['quote_vault']}) amount={vault_meta['quote_vault_amount']} "
        f"decimals={vault_meta['quote_vault_decimals']}"
    )

    if args.user is None:
        print("\n⚠️ 未提供 --user，5/6 号账户使用占位符，请替换成自己的 Token 账户。")
    else:
        print("\n已根据 --user 计算 ATA，可直接用于构造 swap 指令。")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
