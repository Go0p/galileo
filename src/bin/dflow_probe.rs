use std::fs;

use bincode;
use borsh::{BorshDeserialize, BorshSerialize, to_vec};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
struct SwapParams {
    actions: Vec<Action>,
    quoted_out_amount: u64,
    slippage_bps: u16,
    platform_fee_bps: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
enum Action {
    DFlowDynamicRouteV1(DFlowDynamicRouteV1Options) = 31,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
struct DFlowDynamicRouteV1Options {
    candidate_actions: Vec<DynamicRouteV1CandidateAction>,
    amount: u64,
    orchestrator_flags: OrchestratorFlags,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
enum DynamicRouteV1CandidateAction {
    TesseraV(TesseraVDynamicRouteV1Options) = 2,
    HumidiFi(HumidiFiDynamicRouteV1Options) = 3,
    SolFiV2(SolFiV2DynamicRouteV1Options) = 4,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Default)]
struct TesseraVDynamicRouteV1Options;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
struct HumidiFiDynamicRouteV1Options {
    swap_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Default)]
struct SolFiV2DynamicRouteV1Options;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Default, Clone, Copy)]
struct OrchestratorFlags {
    flags: u8,
}

fn main() {
    let path = "copy_tx/template_tx.json";
    let data = fs::read_to_string(path).expect("read template");
    let json: serde_json::Value = serde_json::from_str(&data).expect("parse json");
    let first = json["result"]["transaction"]["message"]["instructions"][2]["data"]
        .as_str()
        .expect("data str");
    let raw = decode_base58(first).expect("decode base58");
    println!("total bytes: {}", raw.len());
    if raw.len() <= 8 {
        panic!("not enough data");
    }
    let (_disc, rest) = raw.split_at(8);
    println!("expected len: {}", rest.len());

    let params = build_sample_params();
    compare("borsh", &to_vec(&params).expect("encode"), rest);
    let bincode_encoded = bincode::serde::encode_to_vec(
        &params,
        bincode::config::standard()
            .with_fixed_int_encoding()
            .with_little_endian(),
    )
    .expect("bincode encode");
    compare("bincode", &bincode_encoded, rest);
}

fn build_sample_params() -> SwapParams {
    SwapParams {
        actions: vec![Action::DFlowDynamicRouteV1(DFlowDynamicRouteV1Options {
            candidate_actions: vec![
                DynamicRouteV1CandidateAction::TesseraV(TesseraVDynamicRouteV1Options),
                DynamicRouteV1CandidateAction::HumidiFi(HumidiFiDynamicRouteV1Options {
                    swap_id: 3217692516193821715,
                }),
                DynamicRouteV1CandidateAction::HumidiFi(HumidiFiDynamicRouteV1Options {
                    swap_id: 14838073818726933429,
                }),
                DynamicRouteV1CandidateAction::HumidiFi(HumidiFiDynamicRouteV1Options {
                    swap_id: 18390110071270421531,
                }),
                DynamicRouteV1CandidateAction::HumidiFi(HumidiFiDynamicRouteV1Options {
                    swap_id: 5243714052430215353,
                }),
                DynamicRouteV1CandidateAction::SolFiV2(SolFiV2DynamicRouteV1Options),
            ],
            amount: 24_000_000_000,
            orchestrator_flags: OrchestratorFlags { flags: 0 },
        })],
        quoted_out_amount: 0,
        slippage_bps: 0,
        platform_fee_bps: 0,
    }
}

fn compare(label: &str, encoded: &[u8], expected: &[u8]) {
    println!("{} len {}", label, encoded.len());
    if encoded == expected {
        println!("{} matches exactly", label);
        return;
    }
    println!("{} differs", label);
    for (idx, (a, b)) in encoded.iter().zip(expected.iter()).enumerate() {
        if a != b {
            println!("{label} diff at {idx}: {:#04x} vs {:#04x}", a, b);
        }
    }
    if encoded.len() != expected.len() {
        println!(
            "{} length differs {} vs {}",
            label,
            encoded.len(),
            expected.len()
        );
    }
}

fn decode_base58(data: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut num = num_bigint::BigUint::from(0u32);
    let base = num_bigint::BigUint::from(58u32);
    for byte in data.bytes() {
        let value = ALPHABET
            .iter()
            .position(|&ch| ch == byte)
            .ok_or_else(|| format!("invalid base58 char: {}", byte as char))?;
        let v = num_bigint::BigUint::from(value as u32);
        num = num * &base + v;
    }
    let mut out = num.to_bytes_be();
    for byte in data.bytes() {
        if byte == b'1' {
            out.insert(0, 0);
        } else {
            break;
        }
    }
    Ok(out)
}
