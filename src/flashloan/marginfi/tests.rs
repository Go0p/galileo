use std::sync::Arc;

use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::config::{AutoUnwrapConfig, FlashloanMarginfiConfig, WalletConfig};
use crate::engine::{EngineIdentity, SwapInstructionsVariant, SwapOpportunity};
use crate::flashloan::FlashloanProtocol;
use crate::strategy::types::TradePair;

use super::{
    MarginfiAccountRegistry, MarginfiFlashloanManager, MarginfiFlashloanPreparation, PROGRAM_ID,
    build_close_instruction,
};

fn make_identity() -> EngineIdentity {
    let signer = Keypair::new();
    let private_key =
        serde_json::to_string(&signer.to_bytes().to_vec()).expect("serialize keypair");
    let wallet = WalletConfig {
        private_key,
        auto_unwrap: AutoUnwrapConfig::default(),
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

fn sample_swap_response() -> crate::api::jupiter::SwapInstructionsResponse {
    crate::api::jupiter::SwapInstructionsResponse {
        raw: Value::Null,
        token_ledger_instruction: None,
        compute_budget_instructions: vec![make_instruction(1)],
        setup_instructions: vec![make_instruction(2)],
        swap_instruction: make_instruction(3),
        cleanup_instruction: Some(make_instruction(4)),
        other_instructions: vec![make_instruction(5)],
        address_lookup_table_addresses: vec![],
        resolved_lookup_tables: Vec::new(),
        prioritization_fee_lamports: 0,
        compute_unit_limit: 0,
        prioritization_type: None,
        dynamic_slippage_report: None,
        simulation_error: None,
    }
}

fn sample_opportunity() -> SwapOpportunity {
    let pair = TradePair::try_new(
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    )
    .expect("valid trade pair");

    SwapOpportunity {
        pair,
        amount_in: 1_000,
        profit_lamports: 0,
        tip_lamports: 0,
        merged_quote: None,
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
    let registry = MarginfiAccountRegistry::new(None);
    let mut manager = MarginfiFlashloanManager::new(&config, rpc, registry);
    manager.adopt_preparation(MarginfiFlashloanPreparation {
        account: marginfi_account,
        created: false,
    });
    assert!(manager.is_enabled());

    let response = SwapInstructionsVariant::Jupiter(sample_swap_response());
    let opportunity = sample_opportunity();
    let outcome = manager
        .assemble(&identity, &opportunity, &response)
        .await
        .expect("assemble instructions");
    assert!(outcome.metadata.is_some());
    let metadata = outcome.metadata.as_ref().unwrap();
    assert_eq!(metadata.protocol, FlashloanProtocol::Marginfi);
    assert_eq!(metadata.mint, opportunity.pair.input_pubkey);
    assert_eq!(metadata.inner_instruction_count, 4);

    // compute budgets + begin + borrow + body(4) + repay + end
    assert_eq!(outcome.instructions.len(), 9);
    // compute-budget instruction remains first
    assert_eq!(outcome.instructions[0].data, vec![1]);
    // marginfi program id inserted
    assert_eq!(outcome.instructions[1].program_id, *PROGRAM_ID);
}

#[tokio::test]
async fn disabled_flashloan_passthrough() {
    let identity = make_identity();
    let response = SwapInstructionsVariant::Jupiter(sample_swap_response());
    let opportunity = sample_opportunity();
    let config = FlashloanMarginfiConfig {
        enable: false,
        prefer_wallet_balance: false,
        marginfi_account: None,
    };
    let rpc = Arc::new(RpcClient::new_mock("mock://marginfi".to_string()));
    let registry = MarginfiAccountRegistry::new(None);
    let manager = MarginfiFlashloanManager::new(&config, rpc, registry);
    assert!(!manager.is_enabled());
    let outcome = manager
        .assemble(&identity, &opportunity, &response)
        .await
        .expect("assemble instructions");
    assert!(outcome.metadata.is_none());
    assert_eq!(outcome.instructions.len(), 5); // 1 compute + 1 setup + swap + other + cleanup
}

#[test]
fn marginfi_close_instruction_layout() {
    let marginfi_account = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let fee_payer = Pubkey::new_unique();

    let ix = build_close_instruction(marginfi_account, &authority, &fee_payer)
        .expect("build close instruction");

    assert_eq!(ix.program_id, *PROGRAM_ID);
    assert_eq!(ix.data, super::CLOSE_ACCOUNT_DISCRIMINATOR.to_vec());

    assert_eq!(ix.accounts.len(), 3);
    let account_meta = &ix.accounts[0];
    assert_eq!(account_meta.pubkey, marginfi_account);
    assert!(account_meta.is_writable);
    assert!(!account_meta.is_signer);

    let authority_meta = &ix.accounts[1];
    assert_eq!(authority_meta.pubkey, authority);
    assert!(!authority_meta.is_writable);
    assert!(authority_meta.is_signer);

    let fee_payer_meta = &ix.accounts[2];
    assert_eq!(fee_payer_meta.pubkey, fee_payer);
    assert!(fee_payer_meta.is_writable);
    assert!(fee_payer_meta.is_signer);
}
