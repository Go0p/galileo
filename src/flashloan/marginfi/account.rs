use std::sync::Arc;

use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use tracing::{debug, info, warn};

use super::instructions::build_initialize_instruction;
use super::{ACCOUNT_HEADER_MIN_LEN, AUTHORITY_OFFSET, GROUP_ID, GROUP_OFFSET, PROGRAM_ID};
use crate::engine::EngineIdentity;

use super::super::error::{FlashloanError, FlashloanResult};

#[derive(Debug, Clone)]
pub struct MarginfiAccountEnsure {
    pub account: Pubkey,
    pub created: bool,
}

pub async fn ensure_marginfi_account(
    rpc: &Arc<RpcClient>,
    identity: &EngineIdentity,
    configured: Option<Pubkey>,
) -> FlashloanResult<MarginfiAccountEnsure> {
    if let Some(account) = configured {
        let fetched = rpc
            .get_account(&account)
            .await
            .map_err(FlashloanError::Rpc)?;
        if marginfi_account_matches_authority(&fetched, &identity.pubkey) {
            return Ok(MarginfiAccountEnsure {
                account,
                created: false,
            });
        } else {
            return Err(FlashloanError::InvalidConfigDetail(format!(
                "配置的 marginfi_account 不属于当前钱包: {account}"
            )));
        }
    }

    let existing = match find_marginfi_account_by_authority(rpc, &identity.pubkey).await {
        Ok(value) => value,
        Err(FlashloanError::Rpc(err)) => {
            warn!(
                target: "flashloan::marginfi",
                error = %err,
                "查询 marginfi account 失败，将直接尝试创建"
            );
            None
        }
        Err(other) => return Err(other),
    };

    if let Some(account) = existing {
        return Ok(MarginfiAccountEnsure {
            account,
            created: false,
        });
    }

    let account_keypair = Keypair::new();
    let ix = build_initialize_instruction(account_keypair.pubkey(), &identity.pubkey)?;

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&identity.pubkey),
        &[identity.signer.as_ref(), &account_keypair],
        blockhash,
    );

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .await
        .map_err(FlashloanError::Rpc)?;

    info!(
        target: "flashloan::marginfi",
        account = %account_keypair.pubkey(),
        signature = %signature,
        "已创建 Marginfi account"
    );

    Ok(MarginfiAccountEnsure {
        account: account_keypair.pubkey(),
        created: true,
    })
}

pub async fn find_marginfi_account_by_authority(
    rpc: &Arc<RpcClient>,
    authority: &Pubkey,
) -> FlashloanResult<Option<Pubkey>> {
    let filters = vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
        GROUP_OFFSET,
        GROUP_ID.to_bytes().to_vec(),
    ))];

    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: Some(false),
        sort_results: None,
    };

    let accounts = rpc
        .get_program_accounts_with_config(&*PROGRAM_ID, config)
        .await?;

    for (pubkey, account) in accounts {
        if marginfi_account_matches_authority(&account, authority) {
            debug!(
                target: "flashloan::marginfi",
                account = %pubkey,
                "检测到已存在的 marginfi account"
            );
            return Ok(Some(pubkey));
        }
    }

    Ok(None)
}

pub fn marginfi_account_matches_authority(account: &Account, authority: &Pubkey) -> bool {
    if account.owner != *PROGRAM_ID || account.data.len() < ACCOUNT_HEADER_MIN_LEN {
        return false;
    }
    extract_authority(&account.data)
        .map(|pubkey| pubkey == *authority)
        .unwrap_or(false)
}

fn extract_authority(data: &[u8]) -> Option<Pubkey> {
    if data.len() < ACCOUNT_HEADER_MIN_LEN {
        return None;
    }
    let mut bytes = [0u8; super::PUBKEY_BYTES];
    bytes.copy_from_slice(&data[AUTHORITY_OFFSET..AUTHORITY_OFFSET + super::PUBKEY_BYTES]);
    Some(Pubkey::new_from_array(bytes))
}
