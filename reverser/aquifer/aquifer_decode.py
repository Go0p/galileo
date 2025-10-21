#!/usr/bin/env python3
"""
输入 Aquifer 池子（`dex_instance` PDA）地址，解析该 swap 指令所需的核心账户。

功能：
1. 读取 `dex_instance` 原始数据，解析 `dex_owner`、base/quote coin、oracle、做市风险账户等字段；
2. 通过 PDA 公式校验 `dex`、`dex_instance`、`coin`、`coin_managed_ta` 等账户；
3. 读取 `coin` 账户，补齐 mint、vault，并判断对应 Token Program / decimals；
4. 可选 `--user`，自动推导 base / quote ATA；默认使用占位符；
5. 支持 JSON 与人类可读两种输出格式（`--json`）。

账户顺序与 Swap 指令一致（常见 16 个账户）：
 0 `sysvar_instructions`
 1 `user_authority`（签名者 / 费支付者）
 2 `token_program_base`
 3 `mm_account`（做市 / 风控 PDA，来自 `dex_instance` 0xCC）
 4 `base_mint`
 5 `token_program_quote`
 6 `base_coin`（PDA：["coin", dex, base_mint]）
 7 `quote_mint`
 8 `dex`（PDA：["dex", dex_owner]）
 9 `dex_instance`（池子本体，PDA：["dex_instance", dex, [instance_id]]）
10 `base_oracle`
11 `quote_oracle`
12 `base_vault_authority`（PDA：["coin_managed_ta", base_coin] 的 Token 账户 owner）
13 `base_vault_token_account`（SPL Token 账户，mint=base_mint，owner=#12）
14 `quote_vault_authority`（PDA：["coin_managed_ta", quote_coin]）
15 `quote_vault_token_account`（SPL Token 账户，mint=quote_mint，owner=#14）

脚本仍会返回额外信息（例如 `sysvar_clock` 建议、可选用户 ATA 等），可按需使用。

注意：字段偏移和校验逻辑基于 `reverser/aquifer/asm` 汇编推断，如链上结构更新需同步维护。
"""
from __future__ import annotations

import argparse
import base64
import ctypes
import ctypes.util
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
_ZSTD_LIB: ctypes.CDLL | None = None


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


def _decompress_zstd(data: bytes) -> bytes:
    global _ZSTD_LIB
    if _ZSTD_LIB is None:
        lib_name = ctypes.util.find_library("zstd")
        if not lib_name:
            raise RuntimeError("缺少 libzstd，无法解压 base64+zstd 数据")
        _ZSTD_LIB = ctypes.CDLL(lib_name)
        _ZSTD_LIB.ZSTD_getFrameContentSize.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        _ZSTD_LIB.ZSTD_getFrameContentSize.restype = ctypes.c_ulonglong
        _ZSTD_LIB.ZSTD_decompress.argtypes = [
            ctypes.c_void_p,
            ctypes.c_size_t,
            ctypes.c_void_p,
            ctypes.c_size_t,
        ]
        _ZSTD_LIB.ZSTD_decompress.restype = ctypes.c_size_t
        _ZSTD_LIB.ZSTD_isError.argtypes = [ctypes.c_size_t]
        _ZSTD_LIB.ZSTD_isError.restype = ctypes.c_uint
        _ZSTD_LIB.ZSTD_getErrorName.argtypes = [ctypes.c_size_t]
        _ZSTD_LIB.ZSTD_getErrorName.restype = ctypes.c_char_p

    src_buf = (ctypes.c_char * len(data)).from_buffer_copy(data)
    src_ptr = ctypes.cast(src_buf, ctypes.c_void_p)
    content_size = _ZSTD_LIB.ZSTD_getFrameContentSize(src_ptr, len(data))
    ZSTD_CONTENTSIZE_ERROR = 2**64 - 1
    ZSTD_CONTENTSIZE_UNKNOWN = 2**64 - 2
    if content_size in (ZSTD_CONTENTSIZE_ERROR, ZSTD_CONTENTSIZE_UNKNOWN):
        content_size = max(len(data) * 32, 8192)
    dst_buf = (ctypes.c_char * content_size)()
    result_size = _ZSTD_LIB.ZSTD_decompress(
        ctypes.cast(dst_buf, ctypes.c_void_p),
        content_size,
        src_ptr,
        len(data),
    )
    if _ZSTD_LIB.ZSTD_isError(result_size):
        err_name = _ZSTD_LIB.ZSTD_getErrorName(result_size)
        raise RuntimeError(
            f"zstd 解压失败: {err_name.decode() if err_name else result_size}"
        )
    return bytes(dst_buf[: result_size])


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
    if len(raw) < 0xCC + 32:
        raise ValueError(f"dex_instance 数据长度过短: {len(raw)}")

    stored_dex = read_pubkey(raw, 0x00)
    dex_owner = read_pubkey(raw, 0x20)
    derived_dex, dex_bump = find_program_address_bytes((b"dex", dex_owner), PROGRAM_ID_BYTES)
    if derived_dex != stored_dex:
        raise ValueError("dex PDA 校验失败")

    instance_id = raw[0x41]
    base_coin_bump = raw[0x42]
    quote_coin_bump = raw[0x43]

    base_coin = read_pubkey(raw, 0x4C)
    quote_coin = read_pubkey(raw, 0x6C)
    base_oracle = read_pubkey(raw, 0x8C)
    quote_oracle = read_pubkey(raw, 0xAC)
    mm_account = read_pubkey(raw, 0xCC)
    return {
        "raw": raw,
        "dex_owner": b58encode(dex_owner),
        "dex": b58encode(derived_dex),
        "dex_bump": dex_bump,
        "instance_id": instance_id,
        "base_coin": b58encode(base_coin),
        "quote_coin": b58encode(quote_coin),
        "base_oracle": b58encode(base_oracle),
        "quote_oracle": b58encode(quote_oracle),
        "mm_account": b58encode(mm_account),
        "base_coin_bump": base_coin_bump,
        "quote_coin_bump": quote_coin_bump,
        "dex_instance_bytes": raw[0x20:0x40],  # 供 coin 验证使用
        "dex_bytes": stored_dex,
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


def fetch_account_record(rpc_url: str, address: str) -> dict[str, typing.Any]:
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
    if not value:
        raise RpcError(f"账户 {address} 数据为空")
    data_field = value.get("data")
    if not data_field:
        raise RpcError(f"账户 {address} 缺少 data 字段")
    data_b64, _encoding = data_field
    data = base64.b64decode(data_b64)
    if data.startswith(b"\x28\xb5\x2f\xfd"):
        data = _decompress_zstd(data)
    return {
        "data": data,
        "owner": value.get("owner"),
        "lamports": value.get("lamports"),
        "executable": value.get("executable", False),
    }


def decode_token_account(raw: bytes) -> dict[str, typing.Any]:
    if len(raw) < 165:
        raise ValueError(f"SPL Token 账户数据长度过短: {len(raw)}")
    mint = read_pubkey(raw, 0x00)
    owner = read_pubkey(raw, 0x20)
    amount = int.from_bytes(raw[0x40:0x48], "little")
    delegate_present = int.from_bytes(raw[0x48:0x50], "little") != 0
    close_authority_present = int.from_bytes(raw[0x78:0x80], "little") != 0
    return {
        "mint": b58encode(mint),
        "owner": b58encode(owner),
        "amount": amount,
        "has_delegate": delegate_present,
        "has_close_authority": close_authority_present,
    }


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
    instance_record = fetch_account_record(rpc_url, dex_instance)
    instance_raw = instance_record["data"]
    instance = decode_dex_instance(instance_raw)

    base_coin_record = fetch_account_record(rpc_url, instance["base_coin"])
    base_coin_raw = base_coin_record["data"]
    base_coin = decode_coin_account(
        base_coin_raw,
        coin_address=instance["base_coin"],
        dex_instance_bytes=instance_raw[0x08:0x28],
        dex_bytes=b58decode(instance["dex"]),
    )
    quote_coin_record = fetch_account_record(rpc_url, instance["quote_coin"])
    quote_coin_raw = quote_coin_record["data"]
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

    base_vault_record = fetch_account_record(rpc_url, base_coin["vault"])
    quote_vault_record = fetch_account_record(rpc_url, quote_coin["vault"])

    if base_vault_record["owner"] not in (TOKEN_PROGRAM_V1, TOKEN_PROGRAM_2022):
        raise RpcError(
            f"base vault token account {base_coin['vault']} owner 非 SPL Token Program: {base_vault_record['owner']}"
        )
    if quote_vault_record["owner"] not in (TOKEN_PROGRAM_V1, TOKEN_PROGRAM_2022):
        raise RpcError(
            f"quote vault token account {quote_coin['vault']} owner 非 SPL Token Program: {quote_vault_record['owner']}"
        )
    base_vault_account = decode_token_account(base_vault_record["data"])
    quote_vault_account = decode_token_account(quote_vault_record["data"])

    if base_vault_account["mint"] != base_coin["mint"]:
        raise RpcError(
            f"base vault token account mint 不匹配: 期望 {base_coin['mint']}, 实际 {base_vault_account['mint']}"
        )
    if quote_vault_account["mint"] != quote_coin["mint"]:
        raise RpcError(
            f"quote vault token account mint 不匹配: 期望 {quote_coin['mint']}, 实际 {quote_vault_account['mint']}"
        )

    base_vault_authority = base_vault_account["owner"]
    quote_vault_authority = quote_vault_account["owner"]

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
        ("sysvar_instructions", SYSVAR_INSTRUCTIONS),
        ("user_authority", user_authority),
        ("token_program_base", base_token_program),
        ("mm_account", instance["mm_account"]),
        ("base_mint", base_coin["mint"]),
        ("token_program_quote", quote_token_program),
        ("base_coin", instance["base_coin"]),
        ("quote_mint", quote_coin["mint"]),
        ("dex", instance["dex"]),
        ("dex_instance", dex_instance),
        ("base_oracle", instance["base_oracle"]),
        ("quote_oracle", instance["quote_oracle"]),
        ("base_vault_authority", base_vault_authority),
        ("base_vault_token_account", base_coin["vault"]),
        ("quote_vault_authority", quote_vault_authority),
        ("quote_vault_token_account", quote_coin["vault"]),
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
            "vault_authority": base_vault_authority,
            "vault_token_account": base_coin["vault"],
            "oracle": instance["base_oracle"],
            "risk": base_coin["risk"],
            "token_program": base_token_program,
            "decimals": base_decimals,
            "vault_amount": base_vault_account["amount"],
        },
        "quote": {
            "coin": instance["quote_coin"],
            "mint": quote_coin["mint"],
            "vault_authority": quote_vault_authority,
            "vault_token_account": quote_coin["vault"],
            "oracle": instance["quote_oracle"],
            "risk": quote_coin["risk"],
            "token_program": quote_token_program,
            "decimals": quote_decimals,
            "vault_amount": quote_vault_account["amount"],
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
