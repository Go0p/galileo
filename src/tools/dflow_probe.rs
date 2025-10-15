use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize, to_vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub fn run(template_path: &Path, instruction_index: usize) -> Result<()> {
    let data = fs::read_to_string(template_path)
        .with_context(|| format!("读取样本交易文件失败: {}", template_path.display()))?;

    let json: Value = serde_json::from_str(&data).context("样本交易 JSON 解析失败")?;
    let instructions = json["result"]["transaction"]["message"]["instructions"]
        .as_array()
        .context("样本交易缺少 instructions 数组")?;

    let instruction = instructions.get(instruction_index).with_context(|| {
        format!(
            "instruction index {} 超出范围（总计 {}）",
            instruction_index,
            instructions.len()
        )
    })?;

    let encoded = instruction["data"]
        .as_str()
        .context("instruction 缺少 base58 编码 data 字段")?;

    let raw = decode_base58(encoded)?;
    println!("total bytes: {}", raw.len());
    if raw.len() <= 8 {
        bail!("样本数据长度不足，无法跳过前 8 字节 discriminant");
    }
    let (_disc, rest) = raw.split_at(8);
    println!("expected len: {}", rest.len());

    let params = build_sample_params();
    let borsh_encoded = to_vec(&params).context("Borsh 编码失败")?;
    report_diff("borsh", &borsh_encoded, rest);

    let bincode_encoded = bincode::serde::encode_to_vec(
        &params,
        bincode::config::standard()
            .with_fixed_int_encoding()
            .with_little_endian(),
    )
    .context("Bincode 编码失败")?;
    report_diff("bincode", &bincode_encoded, rest);

    Ok(())
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

fn report_diff(label: &str, encoded: &[u8], expected: &[u8]) {
    println!("{label} len {}", encoded.len());
    if encoded == expected {
        println!("{label} matches exactly");
        return;
    }
    println!("{label} differs");
    for (idx, (a, b)) in encoded.iter().zip(expected.iter()).enumerate() {
        if a != b {
            println!("{label} diff at {idx}: {:#04x} vs {:#04x}", a, b);
        }
    }
    if encoded.len() != expected.len() {
        println!(
            "{label} length differs {} vs {}",
            encoded.len(),
            expected.len()
        );
    }
}

fn decode_base58(data: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut num = num_bigint::BigUint::from(0u32);
    let base = num_bigint::BigUint::from(58u32);
    for byte in data.bytes() {
        let value = ALPHABET
            .iter()
            .position(|&ch| ch == byte)
            .with_context(|| format!("无效的 base58 字符: {}", byte as char))?;
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
