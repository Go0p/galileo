#!/usr/bin/env python3

"""Decode a base64-encoded Solana transaction inside a DFlow intent.json.

This script focuses on the `openTransaction` field produced by the reverser flow.
It prints a readable breakdown of the transaction header, accounts, and the
individual instructions with best-effort decoding for well-known programs.
"""

from __future__ import annotations

import argparse
import base64
import json
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, List, Sequence


# Alphabet used for Solana's base58 public keys.
BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


# Common program / mint aliases to make the output easier to scan.
KNOWN_ALIASES = {
    "11111111111111111111111111111111": "System Program",
    "ComputeBudget111111111111111111111111111111": "Compute Budget Program",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA": "SPL Token Program",
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL": "Associated Token Program",
    "SysvarRent111111111111111111111111111111111": "Sysvar: Rent",
    "So11111111111111111111111111111111111111112": "Wrapped SOL Mint",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": "USDC Mint",
    "DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH": "DFlow v1 Program",
}

# Token metadata used when rendering human-friendly amounts.
TOKEN_METADATA = {
    "So11111111111111111111111111111111111111112": ("SOL", 9, "Wrapped SOL"),
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": ("USDC", 6, "USD Coin"),
}


@dataclass
class AccountMeta:
    index: int
    address: str
    is_signer: bool
    is_writable: bool

    @property
    def alias(self) -> str | None:
        return KNOWN_ALIASES.get(self.address)


@dataclass
class Instruction:
    program_index: int
    account_indices: List[int]
    data: bytes


@dataclass
class AddressLookupInfo:
    table_address: str
    writable_indexes: List[int]
    readonly_indexes: List[int]


def base58_encode(data: bytes) -> str:
    """Simple base58 encoder (no checksum) suitable for Solana public keys."""
    if not data:
        return ""

    # Count the number of leading zero bytes to preserve them as '1' characters.
    zero_prefix = 0
    for byte in data:
        if byte == 0:
            zero_prefix += 1
        else:
            break

    # Convert the bytes into a big integer and perform base58 conversion.
    value = int.from_bytes(data, "big")
    if value == 0:
        return "1" * max(zero_prefix, 1)

    encoded = ""
    while value > 0:
        value, remainder = divmod(value, 58)
        encoded = BASE58_ALPHABET[remainder] + encoded

    return "1" * zero_prefix + encoded


def read_shortvec(data: bytes, offset: int) -> tuple[int, int]:
    """Parse Solana's short vector encoding (little-endian base-128 varint)."""
    result = 0
    shift = 0

    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        if byte & 0x80:
            shift += 7
            continue
        break

    return result, offset


def decode_transaction(raw: bytes) -> tuple[List[AccountMeta], str, List[Instruction], List[bytes], dict[str, Any]]:
    """Decode a Solana transaction (legacy or v0) into accounts and instructions."""
    offset = 0
    sig_count = raw[offset]
    offset += 1
    signatures = []
    for _ in range(sig_count):
        signatures.append(raw[offset : offset + 64])
        offset += 64

    message_version: int | None = None
    if offset < len(raw) and raw[offset] & 0x80:
        message_version = raw[offset] & 0x7F
        offset += 1

    header = raw[offset : offset + 3]
    if len(header) != 3:
        raise ValueError("transaction header truncated")
    num_required_signers, num_readonly_signed, num_readonly_unsigned = header
    offset += 3

    account_count, offset = read_shortvec(raw, offset)
    accounts: List[str] = []
    for _ in range(account_count):
        accounts.append(base58_encode(raw[offset : offset + 32]))
        offset += 32

    recent_blockhash = base58_encode(raw[offset : offset + 32])
    offset += 32

    # Derive account metadata (signer + writable flags).
    metas: List[AccountMeta] = []
    for idx, address in enumerate(accounts):
        is_signer = idx < num_required_signers
        if is_signer:
            writable = idx < num_required_signers - num_readonly_signed
        else:
            unsigned_idx = idx - num_required_signers
            unsigned_writable = account_count - num_required_signers - num_readonly_unsigned
            writable = unsigned_idx < unsigned_writable

        metas.append(AccountMeta(index=idx, address=address, is_signer=is_signer, is_writable=writable))

    # Parse the instruction list.
    instructions: List[Instruction] = []
    inst_count, offset = read_shortvec(raw, offset)
    for _ in range(inst_count):
        program_index = raw[offset]
        offset += 1
        account_len, offset = read_shortvec(raw, offset)
        account_indices = list(raw[offset : offset + account_len])
        offset += account_len
        data_len, offset = read_shortvec(raw, offset)
        data = raw[offset : offset + data_len]
        offset += data_len
        instructions.append(Instruction(program_index, account_indices, data))

    lookup_infos: List[AddressLookupInfo] = []
    if message_version is not None:
        lookup_count, offset = read_shortvec(raw, offset)
        for _ in range(lookup_count):
            table_address = base58_encode(raw[offset : offset + 32])
            offset += 32
            writable_len, offset = read_shortvec(raw, offset)
            writable_indexes = list(raw[offset : offset + writable_len])
            offset += writable_len
            readonly_len, offset = read_shortvec(raw, offset)
            readonly_indexes = list(raw[offset : offset + readonly_len])
            offset += readonly_len
            lookup_infos.append(
                AddressLookupInfo(
                    table_address=table_address,
                    writable_indexes=writable_indexes,
                    readonly_indexes=readonly_indexes,
                )
            )

        # Append placeholder metas for looked-up accounts to keep indexing intact.
        for lookup in lookup_infos:
            for idx in lookup.writable_indexes:
                label = f"lookup[{lookup.table_address}].w[{idx}]"
                metas.append(AccountMeta(index=len(metas), address=label, is_signer=False, is_writable=True))
            for idx in lookup.readonly_indexes:
                label = f"lookup[{lookup.table_address}].r[{idx}]"
                metas.append(AccountMeta(index=len(metas), address=label, is_signer=False, is_writable=False))

    context = {
        "message_version": message_version,
        "address_table_lookups": lookup_infos,
    }
    return metas, recent_blockhash, instructions, signatures, context


def lamports_to_sol(lamports: int) -> float:
    return lamports / 1_000_000_000


def describe_compute_budget(data: bytes) -> str:
    if not data:
        return "ComputeBudget::Unknown"

    tag = data[0]
    if tag == 2 and len(data) >= 5:
        units = struct.unpack_from("<I", data, 1)[0]
        return f"ComputeBudget::SetComputeUnitLimit units={units}"
    if tag == 3 and len(data) >= 9:
        micro_lamports = struct.unpack_from("<Q", data, 1)[0]
        return f"ComputeBudget::SetComputeUnitPrice micro_lamports={micro_lamports}"
    return f"ComputeBudget::Raw tag={tag} data={data.hex()}"


def describe_associated_token(program_accounts: Sequence[AccountMeta], inst: Instruction) -> str:
    fields = [
        ("payer", 0),
        ("ata", 1),
        ("owner", 2),
        ("mint", 3),
        ("system_program", 4),
        ("token_program", 5),
    ]
    lines = ["AssociatedTokenAccount::Create"]
    for label, idx in fields:
        if idx < len(inst.account_indices):
            account = program_accounts[inst.account_indices[idx]]
            alias = f" ({account.alias})" if account.alias else ""
            lines.append(f"    {label}: [{account.index}] {account.address}{alias}")
    if len(inst.data) > 0:
        lines.append(f"    raw_data: {inst.data.hex()}")
    return "\n".join(lines)


def describe_system(program_accounts: Sequence[AccountMeta], inst: Instruction) -> str:
    if len(inst.data) < 4:
        return f"SystemProgram::Unknown data={inst.data.hex()}"

    tag = struct.unpack_from("<I", inst.data, 0)[0]
    if tag == 2 and len(inst.data) >= 12 and len(inst.account_indices) >= 2:
        lamports = struct.unpack_from("<Q", inst.data, 4)[0]
        src = program_accounts[inst.account_indices[0]]
        dst = program_accounts[inst.account_indices[1]]
        return (
            "SystemProgram::Transfer\n"
            f"    from: [{src.index}] {src.address}\n"
            f"    to:   [{dst.index}] {dst.address}\n"
            f"    amount: {lamports} lamports ({lamports_to_sol(lamports):.9f} SOL)"
        )

    return f"SystemProgram::Raw tag={tag} data={inst.data.hex()}"


def describe_token(program_accounts: Sequence[AccountMeta], inst: Instruction) -> str:
    if not inst.data:
        return "TokenProgram::Unknown (empty data)"

    tag = inst.data[0]
    if tag == 17 and inst.account_indices:
        account = program_accounts[inst.account_indices[0]]
        return f"SPLToken::SyncNative account=[{account.index}] {account.address}"
    if tag == 9 and len(inst.account_indices) >= 3:
        source = program_accounts[inst.account_indices[0]]
        destination = program_accounts[inst.account_indices[1]]
        authority = program_accounts[inst.account_indices[2]]
        lines = [
            "SPLToken::CloseAccount",
            f"    account:     [{source.index}] {source.address}",
            f"    destination: [{destination.index}] {destination.address}",
            f"    authority:   [{authority.index}] {authority.address}",
        ]
        return "\n".join(lines)
    if tag == 3 and len(inst.data) >= 9 and len(inst.account_indices) >= 2:
        amount = struct.unpack_from("<Q", inst.data, 1)[0]
        src = program_accounts[inst.account_indices[0]]
        dst = program_accounts[inst.account_indices[1]]
        return (
            "SPLToken::Transfer\n"
            f"    from: [{src.index}] {src.address}\n"
            f"    to:   [{dst.index}] {dst.address}\n"
            f"    amount: {amount}"
        )
    return f"SPLToken::Raw tag={tag} data={inst.data.hex()}"


def describe_dflow(program_accounts: Sequence[AccountMeta], inst: Instruction, mint_info: dict[str, str]) -> str:
    data = inst.data
    lines = ["DFlow::Invoke"]
    if len(data) >= 32:
        discriminator = data[:8].hex()
        amount_in = struct.unpack_from("<Q", data, 8)[0]
        min_out = struct.unpack_from("<Q", data, 16)[0]
        fee_budget = struct.unpack_from("<Q", data, 24)[0]
        lines.append(f"    discriminator: 0x{discriminator}")
        input_mint = mint_info.get("input_mint")
        output_mint = mint_info.get("output_mint")
        if input_mint and input_mint in TOKEN_METADATA:
            symbol, decimals, description = TOKEN_METADATA[input_mint]
            human = amount_in / (10 ** decimals) if decimals else amount_in
            extra = f" {description}" if description else ""
            lines.append(f"    amount_in: {amount_in} ({human:.{decimals}f} {symbol}{extra})")
        else:
            lines.append(f"    amount_in: {amount_in}")
        if output_mint and output_mint in TOKEN_METADATA:
            symbol, decimals, description = TOKEN_METADATA[output_mint]
            human = min_out / (10 ** decimals)
            extra = f" {description}" if description else ""
            lines.append(f"    min_out: {min_out} ({human:.{decimals}f} {symbol}{extra})")
        else:
            lines.append(f"    min_out: {min_out}")
        lines.append(f"    fee_budget: {fee_budget} lamports ({lamports_to_sol(fee_budget):.9f} SOL)")

        remaining = data[32:]
        if remaining:
            lines.append(f"    remaining_data: {remaining.hex()}")
    else:
        lines.append(f"    raw_data: {data.hex()}")

    lines.append("    accounts:")
    for idx in inst.account_indices:
        account = program_accounts[idx]
        alias = f" ({account.alias})" if account.alias else ""
        role = "signer, writable" if account.is_signer and account.is_writable else (
            "signer, readonly" if account.is_signer else ("writable" if account.is_writable else "readonly")
        )
        lines.append(f"        [{account.index}] {account.address}{alias}  ({role})")
    return "\n".join(lines)


def describe_instruction(metas: Sequence[AccountMeta], inst: Instruction, mint_info: dict[str, str]) -> str:
    program = metas[inst.program_index]
    program_alias = program.alias or ""
    header = f"[{inst.program_index}] {program.address}"
    if program_alias:
        header += f" ({program_alias})"

    if program.address == "ComputeBudget111111111111111111111111111111":
        body = describe_compute_budget(inst.data)
    elif program.address == "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL":
        body = describe_associated_token(metas, inst)
    elif program.address == "11111111111111111111111111111111":
        body = describe_system(metas, inst)
    elif program.address == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA":
        body = describe_token(metas, inst)
    elif program.address == "DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH":
        body = describe_dflow(metas, inst, mint_info)
    else:
        accounts = ", ".join(f"[{idx}]" for idx in inst.account_indices)
        body = f"Raw instruction data={inst.data.hex()} accounts={accounts}"

    return f"{header}\n{body}"


def print_summary(
    metas: Sequence[AccountMeta],
    blockhash: str,
    instructions: Sequence[Instruction],
    signatures: Sequence[bytes],
    mint_info: dict[str, str],
    context: dict[str, Any] | None = None,
) -> None:
    context = context or {}
    message_version = context.get("message_version")
    lookup_infos: Sequence[AddressLookupInfo] = context.get("address_table_lookups", [])
    print("=== Signatures ===")
    if not signatures:
        print("  (none)")
    else:
        for idx, sig in enumerate(signatures):
            readable = sig.hex()
            if set(sig) == {0}:
                readable += " (all zeroes, need to be signed)"
            print(f"  [{idx}] {readable}")

    print("\n=== Recent Blockhash ===")
    if message_version is not None:
        print(f"  Message version: v{message_version}")
    print(f"  {blockhash}\n")

    print("=== Accounts ===")
    for meta in metas:
        flags: List[str] = []
        if meta.is_signer:
            flags.append("signer")
        flags.append("writable" if meta.is_writable else "readonly")
        alias = f" ({meta.alias})" if meta.alias else ""
        print(f"  [{meta.index}] {meta.address}{alias}  ({', '.join(flags)})")

    if lookup_infos:
        print("\n=== Address Table Lookups ===")
        for idx, lookup in enumerate(lookup_infos):
            print(f"  Lookup {idx}: table={lookup.table_address}")
            if lookup.writable_indexes:
                print(f"    writable indexes: {lookup.writable_indexes}")
            if lookup.readonly_indexes:
                print(f"    readonly indexes: {lookup.readonly_indexes}")

    print("\n=== Instructions ===")
    for inst_index, inst in enumerate(instructions):
        print(f"\nInstruction {inst_index}:")
        print(describe_instruction(metas, inst, mint_info))


def main() -> None:
    parser = argparse.ArgumentParser(description="Decode DFlow openTransaction payloads.")
    parser.add_argument(
        "intent_path",
        nargs="?",
        default="./intent.json",
        help="Path to the intent.json file (default: reverser/dflow/intent.json)",
    )
    args = parser.parse_args()

    intent_path = Path(args.intent_path)
    if not intent_path.exists():
        raise SystemExit(f"intent file not found: {intent_path}")

    intent = json.loads(intent_path.read_text(encoding="utf-8"))
    try:
        raw_tx = base64.b64decode(intent["openTransaction"])
    except KeyError as exc:
        raise SystemExit("openTransaction field is missing") from exc

    account_metas, blockhash, instructions, signatures, context = decode_transaction(raw_tx)

    # Pass mint metadata down for program-specific decoding.
    mint_info = {
        "input_mint": intent.get("inputMint", ""),
        "output_mint": intent.get("outputMint", ""),
    }

    print_summary(account_metas, blockhash, instructions, signatures, mint_info, context)

    # Provide a quick summary of the high-level routing amounts if available.
    input_mint = mint_info.get("input_mint", "")
    output_mint = mint_info.get("output_mint", "")
    if input_mint in TOKEN_METADATA or output_mint in TOKEN_METADATA:
        print("\n=== Intent Amounts ===")
        if input_mint:
            amount = int(intent.get("inAmount", 0))
            symbol, decimals, label = TOKEN_METADATA.get(input_mint, ("tokens", 0, ""))
            value = amount / (10 ** decimals) if decimals else amount
            extra = f" {label}" if label else ""
            print(f"  inAmount: {amount} ({value:.{decimals}f} {symbol}{extra})")
        if output_mint:
            amount = int(intent.get("minOutAmount", 0))
            symbol, decimals, label = TOKEN_METADATA.get(output_mint, ("tokens", 0, ""))
            value = amount / (10 ** decimals) if decimals else amount
            extra = f" {label}" if label else ""
            print(f"  minOutAmount: {amount} ({value:.{decimals}f} {symbol}{extra})")


if __name__ == "__main__":
    main()
