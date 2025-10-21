#!/usr/bin/env python3
"""
输入 Aquifer dex_instance 账户地址，解析 swap 所需的核心账户。

脚本流程：
1. 拉取 dex_instance 数据并解析出 dex_owner / dex PDA / base & quote coin。
2. 按推导的 PDA 公式校验 dex / dex_instance / coin / coin_managed_ta。
3. 读取 coin 账户获取 mint / vault / oracle，并补充 mint 的 Token Program 与 decimals。
4. 可选接收用户地址，计算其 base/quote ATA；否则使用占位符。

输出涵盖：
- 程序 ID、dex、dex_instance、instance_id；
- base/quote coin 账户、vault、mint、oracle、风险账户；
- swap 指令常用系统账户 (clock / instructions)。

注意：字段偏移基于 `reverser/aquifer/asm` 汇编推断，如链上结构更新需同步修改。
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
PROGRAM_ID = "AQU1FRd7papthgdrwPTTq5JacJh8YtwEXaBfKU3bTz45"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
SYSVAR_CLOCK = "SysvarC1ock11111111111111111111111111111111"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
TOKEN_PROGRAM_2022 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXkq9AH7K"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BASE58_INDEX = {c: i for i, c in enumerate(BASE58_ALPHABET)}
PROGRAM_ID_BYTES = None  # 在主函数初始化


class RpcError(RuntimeError):
    """RPC 请求失败。"""


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
        num = num * 58 + BASE58_INDEX[ch]
    raw = num.to_bytes((num.bit_length() + 7) // 8, "big") if num else b""
    pad = len(data) - len(data.lstrip("1"))
    return b"\x00" * pad + raw.rjust(32, b"\x00")


def rpc_request(rpc_url: str, method: str, params: typing.List[typing.Any]) -> typing.Any:
    payload = json.dumps(
        {
            "jsonrpc": "2.0",
            "id": method,
            "method": method,
            "params": params,
        }
    ).encode("utf-8")
    req = urllib.request.Request(
        rpc_url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            body = resp.read()
    except urllib.error.URLError as exc:  # pragma: no cover - 环境依赖
        raise RpcError(f"{method} RPC 请求失败: {exc}") from exc
    doc = json.loads(body)
    if "error" in doc:
        raise RpcError(f"{method} 返回错误: {doc['error']}")
    return doc.get("result")


def is_on_curve(pubkey: bytes) -> bool:
    if len(pubkey) != 32:
        return False
    p = 2**255 - 19
    d = (-121665 * pow(121666, -1, p)) % p
    y = int.from_bytes(pubkey, "little") & ((1 << 255) - 1)
    sign = pubkey[31] >> 7
    if y >= p:
        return False
    y2 = (y * y) % p
    u = (y2 - 1) % p
    v = (d * y2 + 1) % p
    if v == 0:
        return False
    x2 = (u * pow(v, p - 2, p)) % p
    x = pow(x2, (p + 3) // 8, p)
    if (x * x - x2) % p != 0:
        x = (x * pow(2, (p - 1) // 4, p)) % p
        if (x * x - x2) % p != 0:
            return False
    if (x % 2) != sign:
        x = (-x) % p
    return not (x == 0 and sign == 1)


def create_program_address(seeds: typing.Iterable[bytes], program_id: bytes) -> bytes:
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


def find_program_address_bytes(
    seeds: typing.Iterable[bytes],
    program_id: bytes,
) -> tuple[bytes, int]:
    seeds_tuple = tuple(seeds)
    for bump in range(255, -1, -1):
        try:
            addr = create_program_address(seeds_tuple + (bytes([bump]),), program_id)
            return addr, bump
        except ValueError:
            continue
    raise RuntimeError("无法找到合法 PDA")


def find_program_address(
    seeds: typing.Iterable[bytes],
    program_id: bytes,
) -> tuple[str, int]:
    addr, bump = find_program_address_bytes(seeds, program_id)
    return b58encode(addr), bump


def find_ata(owner: str, mint: str, token_program: str) -> str:
    owner_bytes = b58decode(owner)
    mint_bytes = b58decode(mint)
    token_prog_bytes = b58decode(token_program)
    assoc_bytes = b58decode(ASSOCIATED_TOKEN_PROGRAM_ID)
    addr, _ = find_program_address_bytes(
        (owner_bytes, token_prog_bytes, mint_bytes),
        assoc_bytes,
    )
    return b58encode(addr)


def read_pubkey(data: bytes, offset: int) -> bytes:
    segment = data[offset : offset + 32]
    if len(segment) != 32:
        raise ValueError(f"偏移 {offset:#x} 解析 pubkey 失败，长度不足")
    return segment


def decode_dex_instance(raw: bytes) -> dict[str, typing.Any]:
    if len(raw) < 0xEC:
        raise ValueError(f"dex_instance 数据长度过短: {len(raw)}")
    owner = read_pubkey(raw, 0x08)
    stored_dex = read_pubkey(raw, 0x28)
    derived_dex, dex_bump = find_program_address_bytes((b"dex", owner), PROGRAM_ID_BYTES)
    if stored_dex != derived_dex:
        raise ValueError("dex PDA 校验失败")
    instance_id = raw[0x49]
    base_coin = read_pubkey(raw, 0x4C)
    quote_coin = read_pubkey(raw, 0x6C)
    base_oracle = read_pubkey(raw, 0x8C)
    quote_oracle = read_pubkey(raw, 0xAC)
    mm_account = read_pubkey(raw, 0xCC)
    return {
        "raw": raw,
        "dex_owner": b58encode(owner),
        "dex": b58encode(derived_dex),
        "dex_bump": dex_bump,
        "instance_id": instance_id,
        "base_coin": b58encode(base_coin),
        "quote_coin": b58encode(quote_coin),
        "base_oracle": b58encode(base_oracle),
        "quote_oracle": b58encode(quote_oracle),
        "mm_account": b58encode(mm_account),
        "base_coin_bump": raw[0x4A],
        "quote_coin_bump": raw[0x4B],
        "dex_instance_bytes": raw[8:40],  # 供 coin 验证使用
        "dex_bytes": derived_dex,
    }


def decode_coin_account(
    raw: bytes,
    *,
    coin_address: str,
    dex_instance_bytes: bytes,
    dex_bytes: bytes,
) -> dict[str, typing.Any]:
    if len(raw) < 0xB0:
        raise ValueError(f"coin 数据长度过短: {len(raw)}")
    if raw[0x08:0x28] != dex_instance_bytes:
        raise ValueError("coin 绑定的 dex_instance 与输入不一致")
    mint = read_pubkey(raw, 0x28)
    vault = read_pubkey(raw, 0x48)
    oracle = read_pubkey(raw, 0x68)
    risk = read_pubkey(raw, 0x88)
    decimals = raw[0xA8]
    coin_addr_bytes = b58decode(coin_address)
    derived_coin, coin_bump = find_program_address_bytes(
        (b"coin", dex_bytes, mint),
        PROGRAM_ID_BYTES,
    )
    if derived_coin != coin_addr_bytes:
        raise ValueError("coin PDA 校验失败")
    vault_expected, vault_bump = find_program_address_bytes(
        (b"coin_managed_ta", coin_addr_bytes),
        PROGRAM_ID_BYTES,
    )
    if vault_expected != vault:
        raise ValueError("coin_managed_ta PDA 校验失败")
    return {
        "mint": b58encode(mint),
        "vault": b58encode(vault),
        "oracle": b58encode(oracle),
        "risk": b58encode(risk),
        "coin_bump": coin_bump,
        "managed_ta_bump": vault_bump,
        "decimals_flag": decimals,
        "raw": raw,
    }


def fetch_account_data(rpc_url: str, address: str) -> bytes:
    resp = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            address,
            {
                "encoding": "base64",
                "commitment": "confirmed",
            },
        ],
    )
    value = resp.get("value")
    if not value or not value.get("data"):
        raise RpcError(f"账户 {address} 数据为空")
    data_b64, _encoding = value["data"]
    return base64.b64decode(data_b64)


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


def resolve_accounts(
    rpc_url: str,
    dex_instance: str,
    user: str | None,
) -> dict[str, typing.Any]:
    instance_raw = fetch_account_data(rpc_url, dex_instance)
    instance = decode_dex_instance(instance_raw)

    base_coin_raw = fetch_account_data(rpc_url, instance["base_coin"])
    base_coin = decode_coin_account(
        base_coin_raw,
        coin_address=instance["base_coin"],
        dex_instance_bytes=instance_raw[0x08:0x28],
        dex_bytes=b58decode(instance["dex"]),
    )
    quote_coin_raw = fetch_account_data(rpc_url, instance["quote_coin"])
    quote_coin = decode_coin_account(
        quote_coin_raw,
        coin_address=instance["quote_coin"],
        dex_instance_bytes=instance_raw[0x08:0x28],
        dex_bytes=b58decode(instance["dex"]),
    )

    base_token_program, base_decimals = fetch_mint_owner_and_decimals(
        rpc_url, base_coin["mint"]
    )
    quote_token_program, quote_decimals = fetch_mint_owner_and_decimals(
        rpc_url, quote_coin["mint"]
    )

    user_authority = user or "<user-authority>"
    if user:
        try:
            user_base_token = find_ata(user, base_coin["mint"], base_token_program)
        except Exception as exc:  # pragma: no cover - 需人工处理
            raise RpcError(f"计算 base ATA 失败: {exc}") from exc
        try:
            user_quote_token = find_ata(user, quote_coin["mint"], quote_token_program)
        except Exception as exc:  # pragma: no cover - 需人工处理
            raise RpcError(f"计算 quote ATA 失败: {exc}") from exc
    else:
        user_base_token = "<user-base-token-account>"
        user_quote_token = "<user-quote-token-account>"

    account_list = [
        ("dex", instance["dex"]),
        ("dex_instance", dex_instance),
        ("user_authority", user_authority),
        ("base_coin", instance["base_coin"]),
        ("quote_coin", instance["quote_coin"]),
        ("base_vault", base_coin["vault"]),
        ("quote_vault", quote_coin["vault"]),
        ("base_mint", base_coin["mint"]),
        ("quote_mint", quote_coin["mint"]),
        ("base_token_program", base_token_program),
        ("quote_token_program", quote_token_program),
        ("base_oracle", instance["base_oracle"]),
        ("quote_oracle", instance["quote_oracle"]),
        ("mm_account", instance["mm_account"]),
        ("risk_base", base_coin["risk"]),
        ("risk_quote", quote_coin["risk"]),
        ("sysvar_clock", SYSVAR_CLOCK),
        ("sysvar_instructions", SYSVAR_INSTRUCTIONS),
        ("user_base_token", user_base_token),
        ("user_quote_token", user_quote_token),
    ]

    return {
        "program_id": PROGRAM_ID,
        "dex": instance["dex"],
        "dex_instance": dex_instance,
        "dex_owner": instance["dex_owner"],
        "instance_id": instance["instance_id"],
        "base": {
            "coin": instance["base_coin"],
            "mint": base_coin["mint"],
            "vault": base_coin["vault"],
            "oracle": instance["base_oracle"],
            "risk": base_coin["risk"],
            "token_program": base_token_program,
            "decimals": base_decimals,
        },
        "quote": {
            "coin": instance["quote_coin"],
            "mint": quote_coin["mint"],
            "vault": quote_coin["vault"],
            "oracle": instance["quote_oracle"],
            "risk": quote_coin["risk"],
            "token_program": quote_token_program,
            "decimals": quote_decimals,
        },
        "mm_account": instance["mm_account"],
        "user": {
            "authority": user_authority,
            "base_token": user_base_token,
            "quote_token": user_quote_token,
        },
        "ordered_accounts": account_list,
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Aquifer swap 账户解码")
    parser.add_argument("pool", help="Aquifer dex_instance 账户地址")
    parser.add_argument("--rpc", default=RPC_DEFAULT, help="RPC endpoint，默认 127.0.0.1:8899")
    parser.add_argument("--user", help="可选：用户钱包地址，用于计算 ATA")
    args = parser.parse_args(argv)

    global PROGRAM_ID_BYTES
    PROGRAM_ID_BYTES = b58decode(PROGRAM_ID)

    try:
        result = resolve_accounts(args.rpc, args.pool, args.user)
    except Exception as exc:
        raise SystemExit(f"解析失败: {exc}") from exc

    json.dump(result, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
