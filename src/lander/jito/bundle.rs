use std::str::FromStr;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use bincode::{config::standard, serde::decode_from_slice, serde::encode_to_vec};
use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use serde_json::{Value, json};
use solana_sdk::instruction::Instruction;
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::VersionedTransaction;
use solana_system_interface::instruction::{self as system_instruction, SystemInstruction};
use solana_system_interface::program;
use url::Url;

use crate::engine::{COMPUTE_BUDGET_PROGRAM_ID, JitoTipPlan, TxVariant};

use super::types::StrategyEndpoint;
use super::uuid::UuidTicket;
use crate::lander::error::LanderError;

static TIP_WALLETS: Lazy<Vec<Pubkey>> = Lazy::new(|| {
    [
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    ]
    .iter()
    .filter_map(|value| Pubkey::from_str(value).ok())
    .collect()
});

pub(crate) const JSONRPC_VERSION: &str = "2.0";

pub(crate) fn random_tip_wallet() -> Option<Pubkey> {
    if TIP_WALLETS.is_empty() {
        None
    } else {
        let mut rng = rand::rng();
        TIP_WALLETS.as_slice().choose(&mut rng).copied()
    }
}

pub(crate) fn encode_transaction<T: serde::Serialize>(tx: &T) -> Result<String, LanderError> {
    let bytes = encode_to_vec(tx, standard())?;
    Ok(BASE64_STANDARD.encode(bytes))
}

pub(crate) fn build_jito_transaction(
    variant: &TxVariant,
    tip: Option<(Pubkey, u64)>,
) -> Result<VersionedTransaction, LanderError> {
    let signer = variant.signer();
    let payer = signer.pubkey();
    let mut instructions = strip_compute_unit_price(variant.instructions().to_vec());

    if let Some((recipient, lamports)) = tip {
        instructions.push(system_instruction::transfer(&payer, &recipient, lamports));
    }

    let message = V0Message::try_compile(
        &payer,
        &instructions,
        variant.lookup_accounts(),
        variant.blockhash(),
    )
    .map_err(|err| LanderError::fatal(format!("构建 Jito 交易消息失败: {err:#}")))?;
    let versioned = VersionedMessage::V0(message);
    VersionedTransaction::try_new(versioned, &[signer.as_ref()])
        .map_err(|err| LanderError::fatal(format!("签名 Jito 交易失败: {err:#}")))
}

fn strip_compute_unit_price(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    instructions
        .retain(|ix| !(ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&3)));
    instructions
}

pub(crate) fn strip_tip_transfer(
    instructions: &[Instruction],
    plan: Option<&JitoTipPlan>,
    payer: &Pubkey,
) -> (Vec<Instruction>, Option<usize>) {
    let Some(plan) = plan else {
        return (instructions.to_vec(), None);
    };

    let mut sanitized = Vec::with_capacity(instructions.len());
    let mut removed_idx = None;

    for (idx, instruction) in instructions.iter().cloned().enumerate() {
        if removed_idx.is_none() && is_tip_transfer(&instruction, payer, plan) {
            removed_idx = Some(idx);
            continue;
        }
        sanitized.push(instruction);
    }

    (sanitized, removed_idx)
}

pub(crate) fn has_tip_transfer(
    instructions: &[Instruction],
    plan: Option<&JitoTipPlan>,
    payer: &Pubkey,
) -> bool {
    let Some(plan) = plan else {
        return false;
    };
    instructions
        .iter()
        .any(|instruction| is_tip_transfer(instruction, payer, plan))
}

fn is_tip_transfer(instruction: &Instruction, payer: &Pubkey, plan: &JitoTipPlan) -> bool {
    if instruction.program_id != program::ID {
        return false;
    }
    if instruction.accounts.len() < 2 {
        return false;
    }
    if instruction.accounts[0].pubkey != *payer || instruction.accounts[1].pubkey != plan.recipient
    {
        return false;
    }

    match decode_system_transfer_lamports(&instruction.data) {
        Some(value) => value == plan.lamports,
        None => false,
    }
}

fn decode_system_transfer_lamports(data: &[u8]) -> Option<u64> {
    let config = standard().with_fixed_int_encoding().with_little_endian();
    let (instruction, _) = decode_from_slice::<SystemInstruction, _>(data, config).ok()?;
    match instruction {
        SystemInstruction::Transfer { lamports }
        | SystemInstruction::TransferWithSeed { lamports, .. } => Some(lamports),
        _ => None,
    }
}

pub(crate) fn prepare_endpoint_url(
    endpoint: &StrategyEndpoint,
    ticket: Option<&UuidTicket>,
) -> Option<Url> {
    let trimmed = endpoint.url.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut url = Url::parse(trimmed).ok()?;
    if let Some(ticket) = ticket {
        url.query_pairs_mut()
            .append_pair("uuid", ticket.uuid.as_str());
    }
    Some(url)
}

pub(crate) fn build_jsonrpc_payload(txs: Vec<String>, ticket: Option<&UuidTicket>) -> Value {
    let bundle_value = Value::Array(txs.into_iter().map(Value::String).collect());
    let mut params = vec![bundle_value];

    let mut options = ticket
        .map(|t| t.options_value())
        .unwrap_or_else(|| Value::Object(Default::default()));
    if let Value::Object(ref mut map) = options {
        map.insert("encoding".to_string(), Value::String("base64".to_string()));
    }
    params.push(options);

    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": 1,
        "method": "sendBundle",
        "params": params,
    })
}
