#!/usr/bin/env python3
"""
Decode SolFi V2 market, config, oracle, and vault token accounts directly from RPC.

Example:
    python3 solfi_v2_decode.py <MARKET_PUBKEY> [--rpc http://127.0.0.1:8899]

The script fetches:
  - The SolFi V2 market account and decodes header/config fields.
  - The linked oracle account (XOR-masked) and prints its price parameters.
  - Base/quote vault SPL token accounts (amounts + owners).

Outputs are human-readable; additionally `--json` emits a JSON document.
"""
from __future__ import annotations

import argparse
import base64
import json
import struct
from dataclasses import asdict, dataclass
from decimal import Decimal, getcontext
from pathlib import Path
from typing import Dict, List, Tuple

import subprocess

# Match Rust precision expectations when converting to human numbers.
getcontext().prec = 40

RPC_DEFAULT = "http://127.0.0.1:8899"
MARKET_ACCOUNT_SIZE = 1728
MARKET_ACCOUNT_HEADER_SIZE = 704
MARKET_CONFIG_SIZE = 1024
ORACLE_ACCOUNT_SIZE = 168
SPL_TOKEN_ACCOUNT_SIZE = 165
PRICE_MASK_BYTES = bytes([0x66, 0x11, 0xEE, 0x77, 0x88, 0x22, 0xDD, 0x44])

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58encode(data: bytes) -> str:
    if not data:
        return ""
    num = int.from_bytes(data, "big")
    enc = ""
    while num > 0:
        num, rem = divmod(num, 58)
        enc = BASE58_ALPHABET[rem] + enc
    pad = 0
    for byte in data:
        if byte == 0:
            pad += 1
        else:
            break
    return "1" * pad + enc


def rpc_request(rpc_url: str, method: str, params: List[object]) -> Dict[str, object]:
    payload = json.dumps(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }
    )
    result = subprocess.run(
        [
            "curl",
            "-sS",
            rpc_url,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            payload,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"curl failed: {result.stderr.strip() or result.stdout}")
    response = json.loads(result.stdout)
    if "error" in response:
        raise RuntimeError(response["error"])
    return response


def fetch_account_data(rpc_url: str, pubkey: str) -> Tuple[bytes, Dict[str, object]]:
    response = rpc_request(
        rpc_url,
        "getAccountInfo",
        [
            pubkey,
            {
                "encoding": "base64",
                "commitment": "processed",
            },
        ],
    )
    value = response.get("result", {}).get("value")
    if value is None:
        raise RuntimeError(f"account {pubkey} not found")
    data_b64 = value.get("data", ["", "base64"])[0]
    if not data_b64:
        raise RuntimeError(f"account {pubkey} has empty data")
    return base64.b64decode(data_b64), value


def read_pubkey(buf: bytes, offset: int) -> Tuple[str, int]:
    return b58encode(buf[offset : offset + 32]), offset + 32


def read_u64(buf: bytes, offset: int) -> Tuple[int, int]:
    return struct.unpack_from("<Q", buf, offset)[0], offset + 8


def read_i64(buf: bytes, offset: int) -> Tuple[int, int]:
    return struct.unpack_from("<q", buf, offset)[0], offset + 8


def read_u32(buf: bytes, offset: int) -> Tuple[int, int]:
    return struct.unpack_from("<I", buf, offset)[0], offset + 4


@dataclass
class MarketHeader:
    bump: int
    market_num: int
    config_version: int
    sequence_number: int
    sequence_number_prev_slot: int
    oracle_account: str
    base_mint: str
    quote_mint: str
    base_vault: str
    quote_vault: str
    base_token_program: str
    quote_token_program: str
    base_mint_decimals: int
    quote_mint_decimals: int
    config_account: str


@dataclass
class Spline:
    x: List[int]
    y: List[int]
    length: int


@dataclass
class MarketConfigV0View:
    enabled: bool
    last_price: int
    last_price_updated_ms: int
    bid_size_edge: Spline
    ask_size_edge: Spline
    time_edge: Spline
    trade_limit: Spline
    min_quote_amount_to_increment_trade_count: int
    num_buys_since_price_update: int
    num_sells_since_price_update: int
    max_trades_per_side_between_price_updates: int
    retreat_milli_bips: int
    retreat_quote_amount: int
    max_retreat_up_milli_bips: int
    max_retreat_down_milli_bips: int
    max_edge_milli_bips: int
    retreat_pct_per_slot: int
    last_known_imbalance: int
    last_balanced_slot: int


@dataclass
class OracleAccountView:
    price_decimals: int
    price_quote_atoms_per_base_atom: int
    ui_price_quote_per_base: Decimal
    price_updated_slot: int
    price_updated_ms: int
    volatility_milli_scale: int
    price_last_valid_slot: int
    bid_widener_milli_bips: int
    ask_widener_milli_bips: int


@dataclass
class TokenAccountView:
    mint: str
    owner: str
    amount_raw: int
    amount_ui: Decimal


@dataclass
class MarketDump:
    header: MarketHeader
    config: MarketConfigV0View
    oracle: OracleAccountView
    base_vault: TokenAccountView
    quote_vault: TokenAccountView


def parse_market_header(buf: bytes) -> MarketHeader:
    if len(buf) != MARKET_ACCOUNT_HEADER_SIZE:
        raise ValueError("invalid market header length")
    offset = 0
    bump, market_num, config_version = struct.unpack_from("<BBB", buf, offset)
    offset += 3
    offset += 5  # padding
    sequence_number, sequence_prev = struct.unpack_from("<QQ", buf, offset)
    offset += 16
    oracle_account, offset = read_pubkey(buf, offset)
    base_mint, offset = read_pubkey(buf, offset)
    quote_mint, offset = read_pubkey(buf, offset)
    base_vault, offset = read_pubkey(buf, offset)
    quote_vault, offset = read_pubkey(buf, offset)
    base_token_program, offset = read_pubkey(buf, offset)
    quote_token_program, offset = read_pubkey(buf, offset)
    base_mint_decimals = buf[offset]
    quote_mint_decimals = buf[offset + 1]
    offset += 8  # decimals + padding
    config_account, offset = read_pubkey(buf, offset)
    # skip padding
    return MarketHeader(
        bump=bump,
        market_num=market_num,
        config_version=config_version,
        sequence_number=sequence_number,
        sequence_number_prev_slot=sequence_prev,
        oracle_account=oracle_account,
        base_mint=base_mint,
        quote_mint=quote_mint,
        base_vault=base_vault,
        quote_vault=quote_vault,
        base_token_program=base_token_program,
        quote_token_program=quote_token_program,
        base_mint_decimals=base_mint_decimals,
        quote_mint_decimals=quote_mint_decimals,
        config_account=config_account,
    )


def parse_spline(buf: bytes, offset: int) -> Tuple[Spline, int]:
    xs = list(struct.unpack_from("<8Q", buf, offset))
    offset += 8 * 8
    ys = list(struct.unpack_from("<8Q", buf, offset))
    offset += 8 * 8
    length = struct.unpack_from("<Q", buf, offset)[0]
    offset += 8
    return Spline(x=xs, y=ys, length=length), offset


def parse_market_config(buf: bytes) -> MarketConfigV0View:
    if len(buf) != MARKET_CONFIG_SIZE:
        raise ValueError("invalid market config length")
    offset = 0
    enabled = buf[offset] == 1
    offset += 8  # enabled + padding
    last_price, offset = read_u64(buf, offset)
    last_price_updated_ms, offset = read_u64(buf, offset)
    bid, offset = parse_spline(buf, offset)
    ask, offset = parse_spline(buf, offset)
    time_edge, offset = parse_spline(buf, offset)
    trade_limit, offset = parse_spline(buf, offset)
    min_quote_amt, offset = read_u64(buf, offset)
    num_buys, offset = read_u32(buf, offset)
    num_sells, offset = read_u32(buf, offset)
    max_trades, offset = read_u32(buf, offset)
    offset += 4  # padding
    retreat_milli_bips, offset = read_i64(buf, offset)
    retreat_quote_amount, offset = read_i64(buf, offset)
    max_retreat_up, offset = read_i64(buf, offset)
    max_retreat_down, offset = read_i64(buf, offset)
    max_edge, offset = read_i64(buf, offset)
    retreat_pct_per_slot, offset = read_i64(buf, offset)
    last_known_imbalance, offset = read_i64(buf, offset)
    last_balanced_slot, offset = read_u64(buf, offset)
    return MarketConfigV0View(
        enabled=enabled,
        last_price=last_price,
        last_price_updated_ms=last_price_updated_ms,
        bid_size_edge=bid,
        ask_size_edge=ask,
        time_edge=time_edge,
        trade_limit=trade_limit,
        min_quote_amount_to_increment_trade_count=min_quote_amt,
        num_buys_since_price_update=num_buys,
        num_sells_since_price_update=num_sells,
        max_trades_per_side_between_price_updates=max_trades,
        retreat_milli_bips=retreat_milli_bips,
        retreat_quote_amount=retreat_quote_amount,
        max_retreat_up_milli_bips=max_retreat_up,
        max_retreat_down_milli_bips=max_retreat_down,
        max_edge_milli_bips=max_edge,
        retreat_pct_per_slot=retreat_pct_per_slot,
        last_known_imbalance=last_known_imbalance,
        last_balanced_slot=last_balanced_slot,
    )


def oracle_mask() -> bytearray:
    mask = bytearray(ORACLE_ACCOUNT_SIZE)
    mask[0:8] = bytes([0xFF, 0xAA, 0x55, 0xCC, 0x33, 0xF0, 0x0F, 0x99])
    mask[8:16] = PRICE_MASK_BYTES
    mask[16:24] = bytes([0xBB, 0x55, 0xAA, 0xFF, 0x00, 0x33, 0xCC, 0x66])
    mask[32:40] = bytes([0x99, 0x77, 0x11, 0xEE, 0x22, 0xDD, 0x88, 0x44])
    mask[40:48] = bytes([0x55, 0xAA, 0xFF, 0x00, 0xCC, 0x33, 0x66, 0x99])
    mask[56:60] = bytes([0x77, 0x88, 0x99, 0xAA])
    mask[60:64] = bytes([0xBB, 0xCC, 0xDD, 0xEE])
    return mask


def apply_oracle_mask(buf: bytearray) -> None:
    mask = oracle_mask()
    for i, byte in enumerate(mask):
        buf[i] ^= byte


def parse_oracle_account(buf: bytes) -> OracleAccountView:
    if len(buf) < ORACLE_ACCOUNT_SIZE:
        raise ValueError("oracle account data too small")
    data = bytearray(buf[:ORACLE_ACCOUNT_SIZE])
    apply_oracle_mask(data)
    offset = 0
    price_decimals, offset = read_i64(data, offset)
    price_quote_atoms_per_base_atom, offset = read_u64(data, offset)
    price_updated_slot, offset = read_u64(data, offset)
    price_updated_ms, offset = read_u64(data, offset)
    volatility_milli_scale, offset = read_u64(data, offset)
    price_last_valid_slot, offset = read_u64(data, offset)
    offset += 8  # padding0
    bid_widener, offset = read_u32(data, offset)
    ask_widener, offset = read_u32(data, offset)
    # Remaining padding is ignored

    price_ui = Decimal(price_quote_atoms_per_base_atom)
    if price_decimals >= 0:
        price_ui *= Decimal(10) ** price_decimals
    else:
        price_ui /= Decimal(10) ** (-price_decimals)

    return OracleAccountView(
        price_decimals=price_decimals,
        price_quote_atoms_per_base_atom=price_quote_atoms_per_base_atom,
        ui_price_quote_per_base=price_ui,
        price_updated_slot=price_updated_slot,
        price_updated_ms=price_updated_ms,
        volatility_milli_scale=volatility_milli_scale,
        price_last_valid_slot=price_last_valid_slot,
        bid_widener_milli_bips=bid_widener,
        ask_widener_milli_bips=ask_widener,
    )


def parse_token_account(buf: bytes, decimals: int) -> TokenAccountView:
    if len(buf) < SPL_TOKEN_ACCOUNT_SIZE:
        raise ValueError("token account too small")
    mint = b58encode(buf[0:32])
    owner = b58encode(buf[32:64])
    amount = struct.unpack_from("<Q", buf, 64)[0]
    amount_ui = Decimal(amount) / (Decimal(10) ** decimals)
    return TokenAccountView(
        mint=mint,
        owner=owner,
        amount_raw=amount,
        amount_ui=amount_ui,
    )


def decode_market(rpc_url: str, market_pubkey: str) -> MarketDump:
    market_data, _meta = fetch_account_data(rpc_url, market_pubkey)
    if len(market_data) < MARKET_ACCOUNT_SIZE:
        raise RuntimeError("market account data too small")
    header_bytes = market_data[:MARKET_ACCOUNT_HEADER_SIZE]
    config_bytes = market_data[
        MARKET_ACCOUNT_HEADER_SIZE : MARKET_ACCOUNT_HEADER_SIZE + MARKET_CONFIG_SIZE
    ]
    header = parse_market_header(header_bytes)
    config = parse_market_config(config_bytes)

    oracle_data, _ = fetch_account_data(rpc_url, header.oracle_account)
    oracle = parse_oracle_account(oracle_data)

    base_vault_data, _ = fetch_account_data(rpc_url, header.base_vault)
    quote_vault_data, _ = fetch_account_data(rpc_url, header.quote_vault)
    base_vault = parse_token_account(base_vault_data, header.base_mint_decimals)
    quote_vault = parse_token_account(quote_vault_data, header.quote_mint_decimals)

    return MarketDump(
        header=header,
        config=config,
        oracle=oracle,
        base_vault=base_vault,
        quote_vault=quote_vault,
    )


def print_dump(dump: MarketDump) -> None:
    header = dump.header
    config = dump.config
    oracle = dump.oracle
    print("== SolFi V2 Market ==")
    print(f"market_num              : {header.market_num}")
    print(f"config_version          : {header.config_version}")
    print(f"sequence_number         : {header.sequence_number}")
    print(f"oracle_account          : {header.oracle_account}")
    print(f"base_mint               : {header.base_mint} (decimals={header.base_mint_decimals})")
    print(f"quote_mint              : {header.quote_mint} (decimals={header.quote_mint_decimals})")
    print(f"base_vault              : {header.base_vault}")
    print(
        f"  mint/owner/amount     : {dump.base_vault.mint} / {dump.base_vault.owner} / {dump.base_vault.amount_ui}"
    )
    print(f"quote_vault             : {header.quote_vault}")
    print(
        f"  mint/owner/amount     : {dump.quote_vault.mint} / {dump.quote_vault.owner} / {dump.quote_vault.amount_ui}"
    )
    print(f"base_token_program      : {header.base_token_program}")
    print(f"quote_token_program     : {header.quote_token_program}")
    print(f"config_account          : {header.config_account}")

    print("\n-- Market Config V0 --")
    print(f"enabled                 : {config.enabled}")
    print(f"last_price              : {config.last_price}")
    print(f"last_price_updated_ms   : {config.last_price_updated_ms}")
    print(
        f"num_buys/sells          : {config.num_buys_since_price_update} / "
        f"{config.num_sells_since_price_update}"
    )
    print(
        f"max_trades_per_side     : {config.max_trades_per_side_between_price_updates}"
    )
    print(f"min_quote_amount        : {config.min_quote_amount_to_increment_trade_count}")
    print(f"retreat_milli_bips      : {config.retreat_milli_bips}")
    print(f"max_edge_milli_bips     : {config.max_edge_milli_bips}")
    print(f"last_known_imbalance    : {config.last_known_imbalance}")
    print(f"last_balanced_slot      : {config.last_balanced_slot}")

    print("\n-- Oracle --")
    print(f"price_decimals          : {oracle.price_decimals}")
    print(f"raw_price_quote/base    : {oracle.price_quote_atoms_per_base_atom}")
    print(f"ui_price_quote/base     : {oracle.ui_price_quote_per_base}")
    print(f"price_updated_slot      : {oracle.price_updated_slot}")
    print(f"price_updated_ms        : {oracle.price_updated_ms}")
    print(f"price_last_valid_slot   : {oracle.price_last_valid_slot}")
    print(f"volatility_milli_scale  : {oracle.volatility_milli_scale}")
    print(f"bid/ask widener (bps)   : {oracle.bid_widener_milli_bips} / {oracle.ask_widener_milli_bips}")


def dump_to_json(dump: MarketDump) -> str:
    def convert(obj):
        if isinstance(obj, Decimal):
            return str(obj)
        if isinstance(obj, dict):
            return {k: convert(v) for k, v in obj.items()}
        if isinstance(obj, list):
            return [convert(v) for v in obj]
        return obj

    payload = asdict(dump)
    return json.dumps(convert(payload), indent=2)


def main() -> None:
    parser = argparse.ArgumentParser(description="Decode SolFi V2 market account.")
    parser.add_argument("market", help="SolFi V2 market account pubkey")
    parser.add_argument("--rpc", default=RPC_DEFAULT, help="RPC endpoint (default: %(default)s)")
    parser.add_argument(
        "--json", action="store_true", help="Emit JSON instead of human-readable text"
    )
    args = parser.parse_args()

    dump = decode_market(args.rpc, args.market)
    if args.json:
        print(dump_to_json(dump))
    else:
        print_dump(dump)


if __name__ == "__main__":
    main()
