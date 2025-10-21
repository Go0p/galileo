#!/usr/bin/env python3
"""
输入 HumidiFi 池子地址，解析 swap 指令需要的核心账户信息。

该脚本只依赖池子配置账户自身的数据（不抓链上历史交易）：
  1. 读取配置账户并解码 base / quote mint；
  2. 通过 `getTokenAccountsByOwner` 查找对应的金库账户；
  3. 汇总 HumidiFi CPI 的账户顺序，并可选计算用户的 ATA。
"""
from __future__ import annotations

import argparse
import base64
import json
import sys
import typing
import urllib.error
import urllib.request

RPC_DEFAULT = "https://mainnet.helius-rpc.com/?api-key=39602420-7609-49be-905e-92421ddcb342"
HUMIDIFI_PROGRAM_ID = "9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"
SYSVAR_CLOCK = "SysvarC1ock11111111111111111111111111111111"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"

# 配置账户中 base / quote mint 的存储采用 4 个 64-bit 常量异或
MINT_MASKS = (
    0xFB5CE87AAE443C38,
    0x04A2178451BAC3C7,
    0x04A1178751B9C3C6,
    0x04A0178651B8C3C5,
)
QUOTE_MINT_OFFSET = 0x180
BASE_MINT_OFFSET = 0x1A0
SWAP_ID_OFFSET = 0x2B0
SWAP_ID_MASK = 0x6E9DE2B30B19F9EA

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
_B58_INDEX = {c: i for i, c in enumerate(BASE58_ALPHABET)}
ED25519_P = 2**255 - 19
ED25519_D = (-121665 * pow(121666, -1, ED25519_P)) % ED25519_P


class RpcError(RuntimeError):
    pass


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
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = resp.read()
    except urllib.error.URLError as exc:  # pragma: no cover - 环境依赖
        raise RpcError(f"{method} RPC 请求失败: {exc}") from exc
    doc = json.loads(body)
    if "error" in doc:
        raise RpcError(f"{method} 返回错误: {doc['error']}")
    return doc.get("result")


def decode_mint(data: bytes, offset: int) -> bytes:
    out = bytearray()
    for idx, mask in enumerate(MINT_MASKS):
        chunk = data[offset + idx * 8 : offset + (idx + 1) * 8]
        if len(chunk) != 8:
            raise ValueError("配置账户长度不足，无法解码 mint")
        value = int.from_bytes(chunk, "little") ^ mask
        out.extend(value.to_bytes(8, "little"))
    return bytes(out)


def decode_swap_id(data: bytes) -> int:
    chunk = data[SWAP_ID_OFFSET : SWAP_ID_OFFSET + 8]
    if len(chunk) != 8:
        raise ValueError("配置账户缺少 swap_id 字段")
    masked = int.from_bytes(chunk, "little")
    return masked ^ SWAP_ID_MASK


def get_config_data(rpc_url: str, pool: str) -> bytes:
    info = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            pool,
            {"encoding": "base64", "commitment": "confirmed"},
        ],
    )
    value = info.get("value")
    if not value or not value.get("data"):
        raise RpcError(f"池子 {pool} 数据为空")
    data_b64, _encoding = value["data"]
    return base64.b64decode(data_b64)


def fetch_vault_for_mint(
    rpc_url: str,
    pool: str,
    mint: str,
) -> dict[str, typing.Any]:
    resp = rpc_request(
        rpc_url,
        "getTokenAccountsByOwner",
        [
            pool,
            {"mint": mint},
            {"encoding": "jsonParsed", "commitment": "confirmed"},
        ],
    )
    candidates = resp.get("value") or []
    if not candidates:
        raise RpcError(f"{pool} 未找到 mint {mint} 关联的 token 账户")

    def _amount(entry: dict) -> int:
        token_info = (
            entry.get("account", {})
            .get("data", {})
            .get("parsed", {})
            .get("info", {})
            .get("tokenAmount", {})
        )
        try:
            return int(token_info.get("amount", "0"))
        except ValueError:
            return 0

    entry = max(candidates, key=_amount)
    parsed = entry["account"]["data"]["parsed"]["info"]
    token_amount = parsed["tokenAmount"]
    return {
        "address": entry["pubkey"],
        "mint": parsed["mint"],
        "decimals": int(token_amount.get("decimals")),
        "amount": token_amount.get("amount"),
        "ui_amount": token_amount.get("uiAmountString"),
        "is_native": parsed.get("isNative", False),
        "token_program": entry["account"]["owner"],
    }


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
    seeds: typing.Iterable[bytes], program_id: bytes
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
    seeds: typing.Iterable[bytes], program_id: bytes
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


def find_ata(owner: str, mint: str, token_program: str) -> tuple[str, int]:
    owner_bytes = b58decode(owner)
    mint_bytes = b58decode(mint)
    token_prog_bytes = b58decode(token_program)
    assoc_bytes = b58decode(ASSOCIATED_TOKEN_PROGRAM_ID)
    return find_program_address(
        (owner_bytes, token_prog_bytes, mint_bytes),
        assoc_bytes,
    )


def resolve_accounts(
    rpc_url: str,
    pool: str,
    user: str | None,
) -> dict[str, typing.Any]:
    data = get_config_data(rpc_url, pool)

    base_mint_raw = decode_mint(data, BASE_MINT_OFFSET)
    quote_mint_raw = decode_mint(data, QUOTE_MINT_OFFSET)
    base_mint = b58encode(base_mint_raw)
    quote_mint = b58encode(quote_mint_raw)
    last_swap_id = decode_swap_id(data)

    base_vault = fetch_vault_for_mint(rpc_url, pool, base_mint)
    quote_vault = fetch_vault_for_mint(rpc_url, pool, quote_mint)

    report: dict[str, typing.Any] = {
        "program_id": HUMIDIFI_PROGRAM_ID,
        "pool_account": pool,
        "base_mint": base_mint,
        "quote_mint": quote_mint,
        "base_vault": base_vault,
        "quote_vault": quote_vault,
        "token_program": base_vault["token_program"],
        "sysvar_clock": SYSVAR_CLOCK,
        "sysvar_instructions": SYSVAR_INSTRUCTIONS,
        "last_swap_id": last_swap_id,
    }

    if user:
        user_base_ata, _ = find_ata(user, base_mint, base_vault["token_program"])
        user_quote_ata, _ = find_ata(user, quote_mint, quote_vault["token_program"])
        report["user_authority"] = user
        report["user_base_ata"] = user_base_ata
        report["user_quote_ata"] = user_quote_ata
        report["payer"] = user
    else:
        report["user_authority"] = "<user-authority>"
        report["user_base_ata"] = "<user-base-token-account>"
        report["user_quote_ata"] = "<user-quote-token-account>"
        report["payer"] = "<payer>"

    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="解析 HumidiFi Swap 所需账户")
    parser.add_argument("pool", help="HumidiFi 池子 / 配置账户地址")
    parser.add_argument(
        "--rpc",
        default=RPC_DEFAULT,
        help="Solana JSON-RPC 终端 (默认: %(default)s)",
    )
    parser.add_argument(
        "--user",
        help="可选：用户签名者公钥，用于计算自己的 base/quote ATA",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="以 JSON 格式输出",
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

    print("=== HumidiFi Swap 账户概览 ===")
    print(f"Program ID          : {report['program_id']}")
    print(f"Pool Account        : {report['pool_account']}")
    print(f"Payer               : {report['payer']}")
    print(f"Base Mint           : {report['base_mint']}")
    print(f"Quote Mint          : {report['quote_mint']}")
    print(f"Last Swap ID        : {report['last_swap_id']}")
    print()

    print("CPI 账户顺序：")
    ordered = [
        ("payer", report["payer"]),
        ("pool_account", report["pool_account"]),
        ("pool_base_vault", report["base_vault"]["address"]),
        ("pool_quote_vault", report["quote_vault"]["address"]),
        ("user_base_token", report["user_base_ata"]),
        ("user_quote_token", report["user_quote_ata"]),
        ("sysvar_clock", report["sysvar_clock"]),
        ("token_program", report["token_program"]),
        ("sysvar_instructions", report["sysvar_instructions"]),
    ]
    for idx, (label, value) in enumerate(ordered):
        print(f"{idx:2d}. {label:<20} {value}")

    print("\nVault / Mint 信息：")
    base = report["base_vault"]
    quote = report["quote_vault"]
    print(
        f"  base  vault {base['address']} -> mint {base['mint']} "
        f"(decimals={base['decimals']}, amount={base['ui_amount']})"
    )
    print(
        f"  quote vault {quote['address']} -> mint {quote['mint']} "
        f"(decimals={quote['decimals']}, amount={quote['ui_amount']})"
    )
    print("\n注意：用户 token 账户会因签名者不同而变化，未指定 --user 时以上为占位。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
