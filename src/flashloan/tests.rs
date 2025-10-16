use std::str::FromStr;
use std::sync::Arc;

use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::config::{FlashloanMarginfiConfig, WalletConfig, WarpOrUnwrapSolConfig};
use crate::engine::{EngineIdentity, SwapOpportunity};
use crate::strategy::types::TradePair;

use super::{FlashloanManager, FlashloanProtocol, MarginfiFlashloan};

fn make_identity() -> EngineIdentity {
    let signer = Keypair::new();
    let private_key =
        serde_json::to_string(&signer.to_bytes().to_vec()).expect("serialize keypair");
    let wallet = WalletConfig {
        private_key,
        min_sol_balance: String::new(),
        warp_or_unwrap_sol: WarpOrUnwrapSolConfig {
            wrap_and_unwrap_sol: false,
            compute_unit_price_micro_lamports: 0,
            skip_user_accounts_rpc_calls: true,
        },
    };
    EngineIdentity::from_wallet(&wallet).expect("build engine identity")
}

fn make_instruction(tag: u8) -> Instruction {
    Instruction {
        program_id: Pubkey::new_unique(),
        accounts: vec![],
        data: vec![tag],
    }
}

fn sample_swap_response() -> crate::api::SwapInstructionsResponse {
    crate::api::SwapInstructionsResponse {
        raw: Value::Null,
        token_ledger_instruction: None,
        compute_budget_instructions: vec![make_instruction(1)],
        setup_instructions: vec![make_instruction(2)],
        swap_instruction: make_instruction(3),
        cleanup_instruction: Some(make_instruction(4)),
        other_instructions: vec![make_instruction(5)],
        address_lookup_table_addresses: vec![],
        prioritization_fee_lamports: 0,
        compute_unit_limit: 0,
        prioritization_type: None,
        dynamic_slippage_report: None,
        simulation_error: None,
    }
}

fn sample_opportunity() -> SwapOpportunity {
    SwapOpportunity {
        pair: TradePair {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        },
        amount_in: 1_000,
        profit_lamports: 0,
        tip_lamports: 0,
        merged_quote: Value::Null,
    }
}

#[tokio::test]
async fn marginfi_wraps_instructions() {
    let identity = make_identity();
    let marginfi_account = Keypair::new().pubkey();
    let config = FlashloanMarginfiConfig {
        enable: true,
        prefer_wallet_balance: false,
        marginfi_account: None,
    };
    let rpc = Arc::new(RpcClient::new_mock("mock://marginfi".to_string()));
    let mut manager = FlashloanManager::new(&config, rpc, None);
    manager.marginfi = Some(MarginfiFlashloan::new(marginfi_account));
    assert!(manager.is_enabled());

    let response = sample_swap_response();
    let opportunity = sample_opportunity();
    let outcome = manager
        .assemble(&identity, &opportunity, &response)
        .await
        .expect("assemble instructions");
    assert!(outcome.metadata.is_some());
    let metadata = outcome.metadata.as_ref().unwrap();
    assert_eq!(metadata.protocol, FlashloanProtocol::Marginfi);
    assert_eq!(
        metadata.mint,
        Pubkey::from_str(&opportunity.pair.input_mint).unwrap()
    );
    assert_eq!(metadata.inner_instruction_count, 4);

    // compute budgets + begin + borrow + body(4) + repay + end
    assert_eq!(outcome.instructions.len(), 9);
    // compute-budget instruction remains first
    assert_eq!(outcome.instructions[0].data, vec![1]);
    // marginfi program id inserted
    assert_eq!(
        outcome.instructions[1].program_id,
        *super::marginfi::PROGRAM_ID
    );
}

#[tokio::test]
async fn disabled_flashloan_passthrough() {
    let identity = make_identity();
    let response = sample_swap_response();
    let opportunity = sample_opportunity();
    let config = FlashloanMarginfiConfig {
        enable: false,
        prefer_wallet_balance: false,
        marginfi_account: None,
    };
    let rpc = Arc::new(RpcClient::new_mock("mock://marginfi".to_string()));
    let manager = FlashloanManager::new(&config, rpc, None);
    assert!(!manager.is_enabled());
    let outcome = manager
        .assemble(&identity, &opportunity, &response)
        .await
        .expect("assemble instructions");
    assert!(outcome.metadata.is_none());
    assert_eq!(outcome.instructions.len(), 5); // 1 compute + 1 setup + swap + other + cleanup
}
