#!/usr/bin/env python3
"""
从本地 RPC (默认为 http://127.0.0.1:8899) 解析 Obric V2 池子，输出 Swap2 所需账户。

用法：
    python obric_v2_decode.py <trading-pair> --user <user-pubkey> --direction x2y

脚本会：
  1. 读取 trading pair 账户原始数据，自动对齐并提取内嵌的各类 Pubkey。
  2. 通过 jsonParsed 查询判别 mint / vault / oracle / price feed。
  3. 以 Obric Swap2 指令顺序输出账户，并给出用户侧的源/目标 Token ATA。

依赖：
  - 仅使用标准库，无需额外安装。
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import sys
import typing
import urllib.error
import urllib.request

RPC_DEFAULT = "http://127.0.0.1:8899"
OBRIC_PROGRAM_ID = "obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y"
TOKEN_PROGRAM_V1 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
TOKEN_PROGRAM_2022 = "TokenzQd4NVYcJ2VfRkximhKcZzsNTszSXgVdDTdL"
ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
SYSVAR_INSTRUCTIONS = "Sysvar1nstructions1111111111111111111111111"

# 计算过的 "account:SSTradingPair" discriminator
SSTRADING_PAIR_DISCRIMINATOR = bytes.fromhex("3bde0fec62665ae0")

# Pyth v2 program id (主网)
PYTH_V2_PROGRAM_IDS = {
    "Fs2X9M7wrp7YjgJvDkXsp1p8Dd1zv9VHtV6nQWmvMRdq",
    "FsLevCLxwJhi3F7S3w7Dnk3a1JpN96CBrum1BgqsSVqP",  # 兼容老地址
    "Minimox7jqQmMpF6Z34DTNwE9iJyNkruzvvYQRaHpAP",  # Obric 默认引用的 Pyth Aggregator
}

SPL_TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
SYSVAR_INSTRUCTIONS_ID = "Sysvar1nstructions1111111111111111111111111"
OBRIC_PROGRAM_ID_STR = "obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y"


class RpcError(RuntimeError):
    pass


def rpc_request(rpc_url: str, method: str, params: typing.Any) -> typing.Any:
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
    result = json.loads(body.decode("utf-8"))
    if "error" in result:
        raise RpcError(f"{method} 调用失败: {result['error']}")
    return result.get("result")


def assemble_accounts(
    data: dict[str, typing.Any],
    order: str,
) -> list[tuple[str, str, bool]]:
    user = data["user"]
    reserves = data["reserves"]
    oracles = data["oracles"]
    feeds = data["price_feeds"]
    ui = data.get("ui_accounts", {})

    if order == "swap2":
        return [
            ("trading_pair", data["trading_pair"], False),
            ("second_reference_oracle", oracles["second_reference_oracle"], False),
            ("third_reference_oracle", oracles["third_reference_oracle"], False),
            ("reserve_x", reserves["reserve_x"], True),
            ("reserve_y", reserves["reserve_y"], True),
            ("user_source_token", user["source_token_account"], True),
            ("user_destination_token", user["destination_token_account"], True),
            ("reference_oracle", oracles["reference_oracle"], False),
            ("x_price_feed", feeds["x_price_feed"], False),
            ("y_price_feed", feeds["y_price_feed"], False),
            ("swap_authority", reserves["swap_authority"], False),
            ("token_program", data["token_program"], False),
        ]

    # 默认使用 swap (旧版 UI) 顺序
    protocol_fee = ui.get("protocol_fee", feeds["x_price_feed"])
    return [
        ("trading_pair", data["trading_pair"], True),
        ("mint_x", ui.get("mint_x", data["mints"]["mint_x"]), False),
        ("mint_y", ui.get("mint_y", data["mints"]["mint_y"]), False),
        ("reserve_x", reserves["reserve_x"], True),
        ("reserve_y", reserves["reserve_y"], True),
        ("user_token_account_x", user["source_token_account"], True),
        ("user_token_account_y", user["destination_token_account"], True),
        ("protocol_fee", protocol_fee, True),
        ("x_price_feed", feeds["x_price_feed"], True),
        ("y_price_feed", feeds["y_price_feed"], True),
        ("user_authority", user["authority"], True),
        ("token_program", data["token_program"], False),
    ]
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
    result = json.loads(body.decode("utf-8"))
    if "error" in result:
        raise RpcError(f"{method} 调用失败: {result['error']}")
    return result["result"]


def b58encode(data: bytes) -> str:
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    num = int.from_bytes(data, "big")
    encoded = ""
    while num > 0:
        num, rem = divmod(num, 58)
        encoded = alphabet[rem] + encoded
    prefix = 0
    for byte in data:
        if byte == 0:
            prefix += 1
        else:
            break
    return "1" * prefix + (encoded or "1")


def b58decode(data: str) -> bytes:
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    num = 0
    for char in data:
        num = num * 58 + alphabet.index(char)
    raw = num.to_bytes(32, "big")
    pad = 0
    for char in data:
        if char == "1":
            pad += 1
        else:
            break
    return b"\x00" * pad + raw[len(raw) - (32) :]


# ---- Ed25519 curve helpers (摘自 tessera_v_decode.py) ----

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
ED25519_P = 2**255 - 19
ED25519_D = (-121665 * pow(121666, -1, ED25519_P)) % ED25519_P


def _recover_x(y: int, sign: int) -> int:
    y2 = (y * y) % ED25519_P
    u = (y2 - 1) % ED25519_P
    v = (ED25519_D * y2 + 1) % ED25519_P
    x2 = (u * pow(v, ED25519_P - 2, ED25519_P)) % ED25519_P
    x = pow(x2, (ED25519_P + 3) // 8, ED25519_P)
    if (x * x - x2) % ED25519_P != 0:
        x = (x * pow(2, (ED25519_P - 1) // 4, ED25519_P)) % ED25519_P
    if x & 1 != sign:
        x = (-x) % ED25519_P
    return x


def is_on_curve(point: bytes) -> bool:
    if len(point) != 32:
        return False
    y = int.from_bytes(point, "little") & ((1 << 255) - 1)
    sign = point[31] >> 7
    x = _recover_x(y, sign)
    return (ED25519_P - ((x * x + y * y - 1) % ED25519_P)) % ED25519_P == (
        ED25519_D * x * x * y * y
    ) % ED25519_P


def create_program_address(
    seeds: typing.Iterable[bytes],
    program_id: bytes,
) -> bytes:
    acc = b"ProgramDerivedAddress"
    for seed in seeds:
        if len(seed) > 32:
            raise ValueError("种子长度大于 32 字节")
        acc += seed
    acc += program_id
    derived = base64.b16decode(hashlib.sha256(acc).hexdigest().encode("ascii"), casefold=True)
    if is_on_curve(derived):
        raise ValueError("无效 PDA：落在曲线上")
    return derived


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


def find_ata(owner: str, mint: str, token_program: str = TOKEN_PROGRAM_V1) -> str:
    owner_bytes = b58decode(owner)
    mint_bytes = b58decode(mint)
    token_prog_bytes = b58decode(token_program)
    assoc_bytes = b58decode(ASSOCIATED_TOKEN_PROGRAM_ID)
    addr, _ = find_program_address(
        (owner_bytes, token_prog_bytes, mint_bytes),
        assoc_bytes,
    )
    return addr


# ---- 账户缓存 ----


class AccountCache:
    def __init__(self, rpc_url: str) -> None:
        self.rpc_url = rpc_url
        self._cache: dict[str, typing.Any] = {}

    def fetch_many(self, pubkeys: typing.Iterable[str]) -> None:
        to_fetch = [pk for pk in pubkeys if pk not in self._cache]
        while to_fetch:
            chunk = to_fetch[:100]
            to_fetch = to_fetch[100:]
            result = rpc_request(
                self.rpc_url,
                "getMultipleAccounts",
                [
                    chunk,
                    {"encoding": "jsonParsed", "commitment": "confirmed"},
                ],
            )
            values = result.get("value") if result else None
            if values is None:
                values = [None] * len(chunk)
            for pk, value in zip(chunk, values):
                self._cache[pk] = value

    def get(self, pubkey: str) -> typing.Optional[typing.Any]:
        if pubkey in self._cache:
            return self._cache[pubkey]
        result = rpc_request(
            self.rpc_url,
            "getAccountInfo",
            [
                pubkey,
                {"encoding": "jsonParsed", "commitment": "confirmed"},
            ],
        )
        value = result.get("value")
        self._cache[pubkey] = value
        return value


# ---- trading pair 数据解析 ----


class ParsedEntry(typing.NamedTuple):
    index: int
    offset: int
    pubkey: str
    info: typing.Optional[typing.Any]


def extract_pubkeys_from_body(
    body: bytes,
    cache: AccountCache,
) -> list[ParsedEntry]:
    offset = find_double_feed_offset(body)
    if offset is not None:
        return build_entries_from_offset(body, cache, offset)

    offsets: list[tuple[int, list[str]]] = []
    sample_pool: set[str] = set()
    for candidate in range(32):
        keys: list[str] = []
        idx = 0
        while True:
            start = candidate + idx * 32
            if start + 32 > len(body):
                break
            keys.append(b58encode(body[start : start + 32]))
            idx += 1
        if keys:
            sample_pool.update(keys[:8])
            offsets.append((candidate, keys))

    cache.fetch_many(list(sample_pool))

    best_offset: typing.Optional[int] = None
    best_hits = -1
    best_keys: list[str] = []
    for candidate, keys in offsets:
        hits = sum(1 for key in keys[:8] if cache.get(key) is not None)
        if hits > best_hits:
            best_hits = hits
            best_offset = candidate
            best_keys = keys

    if best_offset is None or best_hits <= 0:
        raise RuntimeError("无法对齐交易对账户")

    cache.fetch_many(best_keys)
    return build_entries_from_offset_with_keys(body, cache, best_offset, best_keys)


def classify_entries(entries: list[ParsedEntry]) -> dict[str, list[ParsedEntry]]:
    groups: dict[str, list[ParsedEntry]] = {
        "price_feed": [],
        "reserves": [],
        "others": [],
    }
    for entry in entries:
        info = entry.info or {}
        owner = info.get("owner")

        if entry.pubkey == SYSVAR_INSTRUCTIONS_ID:
            continue

        if owner in PYTH_V2_PROGRAM_IDS:
            groups["price_feed"].append(entry)
            continue

        if owner == SPL_TOKEN_PROGRAM_ID:
            groups["reserves"].append(entry)
            continue

        groups["others"].append(entry)
    return groups


def pick_reserves(
    token_accounts: list[ParsedEntry],
) -> tuple[ParsedEntry, ParsedEntry, dict[str, typing.Any]]:
    per_mint: dict[str, list[tuple[int, ParsedEntry]]] = {}
    for entry in token_accounts:
        parsed = entry.info["data"]["parsed"]["info"]
        mint = parsed["mint"]
        amount_raw = parsed["tokenAmount"]["amount"]
        try:
            amount = int(amount_raw)
        except ValueError:  # pragma: no cover
            amount = 0
        per_mint.setdefault(mint, []).append((amount, entry))

    if len(per_mint) < 2:
        raise RuntimeError("token 账户不足，无法判定池子金库")

    mint_meta: dict[str, typing.Any] = {}
    reserves: list[tuple[int, ParsedEntry, str]] = []
    for mint, candidates in per_mint.items():
        candidates.sort(key=lambda item: item[0], reverse=True)
        amount, reserve_entry = candidates[0]
        reserves.append((reserve_entry.index, reserve_entry, mint))
        mint_meta[mint] = {
            "reserve_amount": amount,
            "decimals": reserve_entry.info["data"]["parsed"]["info"]["tokenAmount"]["decimals"],
        }
    if len(reserves) < 2:
        raise RuntimeError("未能找到两条金库账户")
    reserves.sort(key=lambda item: item[0])
    (_, reserve_x, mint_x), (_, reserve_y, mint_y) = reserves[:2]
    return reserve_x, reserve_y, {
        "mint_x": mint_x,
        "mint_y": mint_y,
        "meta": mint_meta,
    }


def resolve_oracles(
    entries: list[ParsedEntry],
    assigned: set[str],
) -> tuple[str, str, str]:
    candidates = [e for e in entries if e.pubkey not in assigned]
    candidates.sort(key=lambda e: e.index)
    if len(candidates) < 3:
        raise RuntimeError("剩余可用账户不足 3 个，无法识别 reference/second/third oracle")
    reference = candidates[0].pubkey
    second = candidates[1].pubkey
    third = candidates[2].pubkey
    return reference, second, third


def resolve_price_feeds(
    price_entries: list[ParsedEntry],
    assigned: set[str],
) -> tuple[str, str]:
    candidates = [e for e in price_entries if e.pubkey not in assigned]
    candidates.sort(key=lambda e: e.index)
    if len(candidates) >= 2:
        return candidates[0].pubkey, candidates[1].pubkey

    if len(candidates) == 1:
        feed = candidates[0].pubkey
        return feed, feed

    # fallback: 统计未分配账户中重复度最高的地址，通常对应同一 oracle/feed 占位
    import collections

    counter = collections.Counter()
    for entry in candidates:
        counter[entry.pubkey] += 1
    if not counter:
        # 退化到 entries 全量统计
        for entry in price_entries:
            if entry.pubkey in assigned:
                continue
            counter[entry.pubkey] += 1
    most_common = [pk for pk, _freq in counter.most_common(2) if pk not in assigned]
    if len(most_common) < 2:
        raise RuntimeError("无法定位到 X/Y price feed")
    return most_common[0], most_common[1]


def find_candidate_before_double_feed(
    body: bytes,
    feed_pubkey: str,
    exclude: set[str],
) -> typing.Optional[str]:
    if feed_pubkey in exclude:
        exclude = set(exclude)
    feed_bytes = b58decode(feed_pubkey)
    block = feed_bytes * 2
    end = len(body) - 96
    i = 0
    while i <= end:
        if body[i + 32 : i + 96] == block:
            candidate = b58encode(body[i : i + 32])
            if candidate != feed_pubkey and candidate not in exclude:
                return candidate
            i += 32
        else:
            i += 1
    return None


def find_double_feed_offset(body: bytes) -> typing.Optional[int]:
    for idx in range(len(body) - 64):
        if body[idx : idx + 32] == body[idx + 32 : idx + 64]:
            return idx % 32
    return None


def build_entries_from_offset(
    body: bytes,
    cache: AccountCache,
    offset: int,
) -> list[ParsedEntry]:
    keys: list[str] = []
    idx = 0
    while True:
        start = offset + idx * 32
        if start + 32 > len(body):
            break
        keys.append(b58encode(body[start : start + 32]))
        idx += 1
    cache.fetch_many(keys)
    return build_entries_from_offset_with_keys(body, cache, offset, keys)


def build_entries_from_offset_with_keys(
    body: bytes,
    cache: AccountCache,
    offset: int,
    keys: list[str],
) -> list[ParsedEntry]:
    entries: list[ParsedEntry] = []
    for idx, pubkey in enumerate(keys):
        start = offset + idx * 32
        info = cache.get(pubkey)
        entries.append(ParsedEntry(idx, start, pubkey, info))
    return entries


def build_swap_accounts(
    trading_pair: str,
    rpc_url: str,
    cache: AccountCache,
    user: typing.Optional[str],
    direction: str,
) -> dict[str, typing.Any]:
    raw_resp = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            trading_pair,
            {"encoding": "base64", "commitment": "confirmed"},
        ],
    )
    raw_value = raw_resp.get("value")
    if not raw_value:
        raise RuntimeError(f"trading pair {trading_pair} 数据为空")
    data_raw = raw_value.get("data")
    if not isinstance(data_raw, list) or not data_raw:
        raise RuntimeError("trading pair 原始数据格式异常")
    raw = base64.b64decode(data_raw[0])
    if not raw.startswith(SSTRADING_PAIR_DISCRIMINATOR):
        raise RuntimeError("账户 discriminator 不匹配 SSTradingPair")
    body = raw[len(SSTRADING_PAIR_DISCRIMINATOR) :]
    entries = extract_pubkeys_from_body(body, cache)

    groups = classify_entries(entries)
    reserve_x_entry, reserve_y_entry, mint_meta = pick_reserves(groups["reserves"])

    mint_x = mint_meta["mint_x"]
    mint_y = mint_meta["mint_y"]
    reserve_x = reserve_x_entry.pubkey
    reserve_y = reserve_y_entry.pubkey

    # 读取 token program
    token_program = TOKEN_PROGRAM_V1
    for entry in entries:
        if entry.pubkey in (TOKEN_PROGRAM_V1, TOKEN_PROGRAM_2022):
            token_program = entry.pubkey
            break

    swap_authority = reserve_x_entry.info["data"]["parsed"]["info"]["owner"]
    if swap_authority != reserve_y_entry.info["data"]["parsed"]["info"]["owner"]:
        raise RuntimeError("两侧金库 owner 不一致，疑似数据异常")

    assigned = {
        reserve_x,
        reserve_y,
        mint_x,
        mint_y,
        token_program,
    }

    x_price_feed, y_price_feed = resolve_price_feeds(groups["price_feed"], assigned)
    assigned.update({x_price_feed, y_price_feed})

    oracle_candidates = [
        entry
        for entry in groups["others"]
        if entry.pubkey not in assigned
        and (entry.info or {}).get("owner") != OBRIC_PROGRAM_ID_STR
    ]

    reference_oracle, second_reference_oracle, third_reference_oracle = resolve_oracles(
        oracle_candidates,
        assigned,
    )
    assigned.update({reference_oracle, second_reference_oracle, third_reference_oracle})

    mint_sslp_x = None
    for entry in entries:
        info = entry.info or {}
        if info.get("owner") == OBRIC_PROGRAM_ID and entry.pubkey != trading_pair:
            mint_sslp_x = entry.pubkey
            break
    mint_sslp_x = mint_sslp_x or mint_x
    assigned.add(mint_sslp_x)

    protocol_fee = x_price_feed
    mint_sslp_y = find_candidate_before_double_feed(body, x_price_feed, assigned)
    if mint_sslp_y:
        assigned.add(mint_sslp_y)
    else:
        mint_sslp_y = mint_y

    # 用户 token 账户
    user_source = "<user-source-token>"
    user_destination = "<user-destination-token>"
    if user:
        if direction == "x2y":
            source_mint, dest_mint = mint_x, mint_y
        else:
            source_mint, dest_mint = mint_y, mint_x
        try:
            user_source = find_ata(user, source_mint, token_program)
            user_destination = find_ata(user, dest_mint, token_program)
        except Exception as exc:  # pragma: no cover - unlikely
            raise RuntimeError(f"计算用户 ATA 失败: {exc}") from exc

    return {
        "program_id": OBRIC_PROGRAM_ID,
        "trading_pair": trading_pair,
        "mints": {
            "mint_x": mint_x,
            "mint_y": mint_y,
        },
        "reserves": {
            "reserve_x": reserve_x,
            "reserve_y": reserve_y,
            "swap_authority": swap_authority,
        },
        "oracles": {
            "reference_oracle": reference_oracle,
            "second_reference_oracle": second_reference_oracle,
            "third_reference_oracle": third_reference_oracle,
        },
        "price_feeds": {
            "x_price_feed": x_price_feed,
            "y_price_feed": y_price_feed,
        },
        "ui_accounts": {
            "mint_x": mint_sslp_x,
            "mint_y": mint_sslp_y,
            "protocol_fee": protocol_fee,
        },
        "token_program": token_program,
        "user": {
            "authority": user or "<user-authority>",
            "source_token_account": user_source,
            "destination_token_account": user_destination,
        },
    }


def main(argv: typing.Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="解析 Obric V2 Swap2 所需账户")
    parser.add_argument("trading_pair", help="Trading pair 账户地址")
    parser.add_argument(
        "--rpc",
        default=RPC_DEFAULT,
        help="RPC 端点 (默认: %(default)s)",
    )
    parser.add_argument(
        "--user",
        help="可选：用户签名者 Pubkey，用于计算源/目标 ATA",
    )
    parser.add_argument(
        "--direction",
        choices=("x2y", "y2x"),
        default="x2y",
        help="用户 swap 方向，决定 source/destination ATA (默认: %(default)s)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="以 JSON 形式输出详细结构",
    )
    parser.add_argument(
        "--order",
        choices=("swap", "swap2"),
        default="swap",
        help="账户输出顺序：旧版 swap 或新版 swap2 (默认: %(default)s)",
    )
    args = parser.parse_args(argv)

    cache = AccountCache(args.rpc)
    try:
        result = build_swap_accounts(
            args.trading_pair,
            args.rpc,
            cache,
            args.user,
            args.direction,
        )
    except RpcError as exc:
        print(f"RPC 失败: {exc}", file=sys.stderr)
        return 1
    except Exception as exc:
        print(f"解析失败: {exc}", file=sys.stderr)
        return 1

    accounts = assemble_accounts(result, args.order)

    if args.json:
        payload = dict(result)
        payload["order"] = args.order
        payload["accounts"] = accounts
        print(json.dumps(payload, indent=2, ensure_ascii=False))
        return 0

    print(f"Obric V2 {args.order.upper()} 账户列表 (按指令顺序):")
    for name, pubkey, writable in accounts:
        flag = " (W)" if writable else ""
        print(f"  - {name}{flag}: {pubkey}")
    print()
    print("关键派生信息：")
    print(f"  mint_x (pool share): {result['ui_accounts']['mint_x']}")
    print(f"  mint_y (pool share): {result['ui_accounts']['mint_y']}")
    print(f"  mint_x: {result['mints']['mint_x']}")
    print(f"  mint_y: {result['mints']['mint_y']}")
    print(f"  protocol_fee: {result['ui_accounts']['protocol_fee']}")
    print(f"  swap_authority: {result['reserves']['swap_authority']}")
    print(f"  token_program: {result['token_program']}")
    print()
    print("用户账户：")
    print(f"  authority: {result['user']['authority']}")
    print(f"  source ATA: {result['user']['source_token_account']}")
    print(f"  destination ATA: {result['user']['destination_token_account']}")
    print()
    print("如需 JSON 输出请添加 --json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
