use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::config::wallet::parse_keypair_string;

use super::error::{EngineError, EngineResult};

#[derive(Clone)]
pub struct EngineIdentity {
    pub pubkey: Pubkey,
    fee_account: Option<String>,
    skip_user_accounts_rpc_calls: bool,
    pub signer: Arc<Keypair>,
}

impl EngineIdentity {
    pub fn from_private_key(private_key: &str) -> EngineResult<Self> {
        if private_key.trim().is_empty() {
            return Err(EngineError::InvalidConfig(
                "私钥字符串为空，请先配置并解密钱包".into(),
            ));
        }

        let keypair = parse_keypair_string(private_key.trim())
            .map_err(|err| EngineError::InvalidConfig(format!("私钥格式非法: {err}")))?;
        let signer = Arc::new(keypair);
        let pubkey = signer.pubkey();

        Ok(Self {
            pubkey,
            fee_account: None,
            skip_user_accounts_rpc_calls: false,
            signer,
        })
    }

    pub fn fee_account(&self) -> Option<&str> {
        self.fee_account.as_deref()
    }

    pub fn skip_user_accounts_rpc_calls(&self) -> bool {
        self.skip_user_accounts_rpc_calls
    }

    pub fn set_skip_user_accounts_rpc_calls(&mut self, value: bool) {
        self.skip_user_accounts_rpc_calls = value;
    }
}
