#!/usr/bin/env python3
"""
输入 Saros 池子（swap state）地址，解析一次 Swap 指令所需的账户。

能力：
 1. 读取池子原始数据，解析 token_a / token_b / fee_account 等字段；
 2. 根据 bump seed 计算 authority PDA；
 3. 根据方向（a2b / b2a）输出账户顺序；
 4. 可选 `--user` 计算用户侧 ATA，便于直接构造交易；
 5. 支持 JSON 与人类可读两种输出。

账户顺序与合约 `Swap` 指令完全一致：
 0 swap_state
 1 authority
 2 user_transfer_authority
 3 user_source
 4 pool_source
 5 pool_destination
 6 user_destination
 7 pool_mint
 8 fee_account
 9 token_program
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
SAROS_PROGRAM_ID = "SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
TOKEN_PROGRAM_2022 = "TokenzQdSbnjHr1P1a9wYFwS6gkU1GGzLRtXju6Rjt92"

POOL_LAYOUT = {
    "version": 0x00,
    "is_initialized": 0x01,
    "bump_seed": 0x02,
    "token_program": 0x08,
    "token_a": 0x28,
    "token_b": 0x48,
    "pool_mint": 0x68,
    "fee_account": 0x88,
    "token_a_mint": 0xA8,
    "token_b_mint": 0xC8,
    "token_a_deposit": 0xE8,
    "token_b_deposit": 0xF0,
    "token_a_fees": 0xF8,
    "token_b_fees": 0x100,
    "fees": 0x108,  # 8 × u64
    "curve_type": 0x130,
    "curve_parameters": 0x138,
}

FEE_FIELD_NAMES = (
    "trade_fee_numerator",
    "trade_fee_denominator",
    "owner_trade_fee_numerator",
    "owner_trade_fee_denominator",
    "owner_withdraw_fee_numerator",
    "owner_withdraw_fee_denominator",
    "host_fee_numerator",
    "host_fee_denominator",
)

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
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        rpc_url,
        data=body,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req) as resp:
            raw = resp.read()
    except urllib.error.URLError as exc:  # pragma: no cover - 网络环境依赖
        raise RpcError(f"RPC 请求失败: {exc}") from exc
    data = json.loads(raw)
    if "error" in data:
        raise RpcError(f"{method} 调用失败: {data['error']}")
    return data.get("result")


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


def b58decode(data: str) -> bytes:
    num = 0
    for ch in data:
        num = num * 58 + _B58_INDEX[ch]
    pad = len(data) - len(data.lstrip("1"))
    if num == 0:
        raw = b""
    else:
        raw = num.to_bytes((num.bit_length() + 7) // 8, "big")
    raw = b"\x00" * pad + raw
    return raw.rjust(32, b"\x00")


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


def read_pubkey(raw: bytes, offset: int) -> str:
    return b58encode(raw[offset : offset + 32])


def read_u64(raw: bytes, offset: int) -> int:
    return int.from_bytes(raw[offset : offset + 8], "little")


def fetch_pool_state(rpc_url: str, pool: str) -> bytes:
    result = rpc_request(
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
    value = result.get("value") if result else None
    if not value:
        raise RpcError(f"池子 {pool} 不存在或为空")
    data_field = value.get("data")
    if not data_field or not isinstance(data_field, list):
        raise RpcError(f"池子 {pool} 数据解析失败")
    data_b64, _encoding = data_field
    raw = base64.b64decode(data_b64)
    if len(raw) < POOL_LAYOUT["curve_parameters"]:
        raise RpcError(f"池子 {pool} 数据长度异常: {len(raw)}")
    return raw


def fetch_mint_info(
    rpc_url: str,
    mint: str,
) -> tuple[str, int]:
    result = rpc_request(
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
    value = result.get("value") if result else None
    if not value:
        raise RpcError(f"mint {mint} 不存在")
    owner = value.get("owner")
    parsed = value.get("data", {}).get("parsed", {})
    decimals = parsed.get("info", {}).get("decimals")
    if decimals is None:
        raise RpcError(f"mint {mint} 缺少 decimals 字段")
    return owner, int(decimals)


def fetch_token_account_owner(
    rpc_url: str,
    token_account: str,
) -> str:
    result = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            token_account,
            {
                "encoding": "jsonParsed",
                "commitment": "confirmed",
            },
        ],
    )
    value = result.get("value") if result else None
    if not value:
        raise RpcError(f"SPL Token 账户 {token_account} 不存在")
    owner = value.get("owner")
    if not owner:
        raise RpcError(f"SPL Token 账户 {token_account} 缺少 owner 字段")
    return owner


def parse_pool(raw: bytes) -> dict[str, typing.Any]:
    fields = {
        "version": raw[POOL_LAYOUT["version"]],
        "is_initialized": raw[POOL_LAYOUT["is_initialized"]] == 1,
        "bump_seed": raw[POOL_LAYOUT["bump_seed"]],
        "token_program": read_pubkey(raw, POOL_LAYOUT["token_program"]),
        "token_a": read_pubkey(raw, POOL_LAYOUT["token_a"]),
        "token_b": read_pubkey(raw, POOL_LAYOUT["token_b"]),
        "pool_mint": read_pubkey(raw, POOL_LAYOUT["pool_mint"]),
        "fee_account": read_pubkey(raw, POOL_LAYOUT["fee_account"]),
        "token_a_mint": read_pubkey(raw, POOL_LAYOUT["token_a_mint"]),
        "token_b_mint": read_pubkey(raw, POOL_LAYOUT["token_b_mint"]),
        "token_a_deposit": read_u64(raw, POOL_LAYOUT["token_a_deposit"]),
        "token_b_deposit": read_u64(raw, POOL_LAYOUT["token_b_deposit"]),
        "token_a_fees": read_u64(raw, POOL_LAYOUT["token_a_fees"]),
        "token_b_fees": read_u64(raw, POOL_LAYOUT["token_b_fees"]),
        "curve_type": raw[POOL_LAYOUT["curve_type"]],
    }
    fees = {}
    for idx, name in enumerate(FEE_FIELD_NAMES):
        offset = POOL_LAYOUT["fees"] + idx * 8
        fees[name] = read_u64(raw, offset)
    fields["fees"] = fees
    return fields


def resolve_accounts(
    rpc_url: str,
    pool: str,
    user: str | None,
    direction: str,
) -> dict[str, typing.Any]:
    raw = fetch_pool_state(rpc_url, pool)
    pool_state = parse_pool(raw)
    if pool_state["version"] != 1 or not pool_state["is_initialized"]:
        raise RpcError("池子未初始化或版本不匹配")

    authority, bump = find_program_address(
        (b58decode(pool),),
        b58decode(SAROS_PROGRAM_ID),
    )
    if bump != pool_state["bump_seed"]:
        raise RpcError("PDA bump 与池子记录不一致")

    token_program_recorded = pool_state["token_program"]
    token_program_a = fetch_token_account_owner(rpc_url, pool_state["token_a"])
    token_program_b = fetch_token_account_owner(rpc_url, pool_state["token_b"])
    if token_program_a != token_program_b:
        raise RpcError(
            f"token_a/token_b 所属程序不一致: {token_program_a} vs {token_program_b}"
        )
    token_program = token_program_a

    mint_a_owner, mint_a_decimals = fetch_mint_info(rpc_url, pool_state["token_a_mint"])
    mint_b_owner, mint_b_decimals = fetch_mint_info(rpc_url, pool_state["token_b_mint"])
    if mint_a_owner != token_program or mint_b_owner != token_program:
        raise RpcError("mint 所属 Token Program 与金库 owner 不一致")

    user_transfer_authority = user or "<user-transfer-authority>"
    if direction == "a2b":
        user_source_label = "user_token_a"
        user_destination_label = "user_token_b"
        pool_source = pool_state["token_a"]
        pool_destination = pool_state["token_b"]
        source_mint = pool_state["token_a_mint"]
        destination_mint = pool_state["token_b_mint"]
    elif direction == "b2a":
        user_source_label = "user_token_b"
        user_destination_label = "user_token_a"
        pool_source = pool_state["token_b"]
        pool_destination = pool_state["token_a"]
        source_mint = pool_state["token_b_mint"]
        destination_mint = pool_state["token_a_mint"]
    else:
        raise ValueError(f"未知方向: {direction}")

    if user:
        try:
            if token_program in (TOKEN_PROGRAM_V1, TOKEN_PROGRAM_2022):
                user_source, _ = find_ata(user, source_mint, token_program)
                user_destination, _ = find_ata(user, destination_mint, token_program)
            else:
                raise ValueError(
                    f"暂不支持 token program {token_program} 的 ATA 计算"
                )
        except Exception as exc:  # pragma: no cover - 仅在输入异常时触发
            raise RpcError(f"计算用户 ATA 失败: {exc}") from exc
    else:
        user_source = f"<{user_source_label}>"
        user_destination = f"<{user_destination_label}>"

    accounts = [
        ("swap_state", pool),
        ("authority", authority),
        ("user_transfer_authority", user_transfer_authority),
        ("user_source", user_source),
        ("pool_source", pool_source),
        ("pool_destination", pool_destination),
        ("user_destination", user_destination),
        ("pool_mint", pool_state["pool_mint"]),
        ("fee_account", pool_state["fee_account"]),
        ("token_program", token_program),
    ]

    metadata = {
        "program_id": SAROS_PROGRAM_ID,
        "pool_state": pool,
        "token_program": token_program,
        "token_program_recorded": token_program_recorded,
        "authority": authority,
        "bump_seed": pool_state["bump_seed"],
        "direction": direction,
        "mints": {
            "token_a": {
                "mint": pool_state["token_a_mint"],
                "decimals": mint_a_decimals,
            },
            "token_b": {
                "mint": pool_state["token_b_mint"],
                "decimals": mint_b_decimals,
            },
        },
        "fees": pool_state["fees"],
        "curve": {
            "type": pool_state["curve_type"],
        },
        "accounts": {name: value for name, value in accounts},
        "ordered_accounts": accounts,
    }
    return metadata


def print_human_readable(report: dict[str, typing.Any]) -> None:
    print("=== Saros Swap 账户 ===")
    print(f"Program ID        : {report['program_id']}")
    print(f"Pool State        : {report['pool_state']}")
    print(f"Token Program     : {report['token_program']}")
    if report["token_program"] != report["token_program_recorded"]:
        print(
            f"  (swap state 内记录为 {report['token_program_recorded']}, 已以实际金库 owner 为准)"
        )
    print(f"Authority (bump={report['bump_seed']}): {report['authority']}")
    print(f"Swap Direction    : {report['direction']}")
    print("\n账户顺序：")
    for idx, (name, value) in enumerate(report["ordered_accounts"]):
        print(f"{idx:2d}. {name:<24} {value}")
    print("\nMint 信息：")
    for side in ("token_a", "token_b"):

        mint_info = report["mints"][side]
        print(f"  {side:<7} mint={mint_info['mint']} decimals={mint_info['decimals']}")
    print("\nFees:")
    for key, value in report["fees"].items():
        print(f"  {key:<32} {value}")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="解析 Saros 池子的 swap 账户"
    )
    parser.add_argument("pool", help="Saros 池子（swap state）地址")
    parser.add_argument(
        "--rpc",
        default=RPC_DEFAULT,
        help="RPC 节点 (默认: %(default)s)",
    )
    parser.add_argument(
        "--user",
        help="可选：用户签名者，若提供则计算 ATA",
    )
    parser.add_argument(
        "--direction",
        choices=("a2b", "b2a"),
        default="a2b",
        help="交换方向，默认 token_a -> token_b",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="以 JSON 格式输出",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        report = resolve_accounts(args.rpc, args.pool, args.user, args.direction)
    except (RpcError, ValueError, RuntimeError) as exc:
        print(f"解析失败: {exc}", file=sys.stderr)
        return 1

    if args.json:
        json.dump(report, sys.stdout, ensure_ascii=False, indent=2)
        print()
        return 0

    print_human_readable(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
