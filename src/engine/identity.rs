use std::sync::Arc;

use bs58;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::config::WalletConfig;

use super::error::{EngineError, EngineResult};

#[derive(Clone)]
pub struct EngineIdentity {
    pub pubkey: Pubkey,
    fee_account: Option<String>,
    use_shared_accounts: bool,
    skip_user_accounts_rpc_calls: bool,
    pub signer: Arc<Keypair>,
}

impl EngineIdentity {
    pub fn from_wallet(wallet: &WalletConfig) -> EngineResult<Self> {
        let signer = load_keypair(wallet)?;
        let pubkey = signer.pubkey();

        let fee_account = None;

        let use_shared_accounts = false;
        let skip_user_accounts_rpc_calls = false;

        Ok(Self {
            pubkey,
            fee_account,
            use_shared_accounts,
            skip_user_accounts_rpc_calls,
            signer,
        })
    }

    pub fn fee_account(&self) -> Option<&str> {
        self.fee_account.as_deref()
    }

    pub fn use_shared_accounts(&self) -> bool {
        self.use_shared_accounts
    }

    pub fn skip_user_accounts_rpc_calls(&self) -> bool {
        self.skip_user_accounts_rpc_calls
    }

    pub fn set_skip_user_accounts_rpc_calls(&mut self, value: bool) {
        self.skip_user_accounts_rpc_calls = value;
    }
}

fn load_keypair(wallet: &WalletConfig) -> EngineResult<Arc<Keypair>> {
    if !wallet.private_key.trim().is_empty() {
        let keypair = parse_keypair_string(wallet.private_key.trim()).map_err(|err| {
            EngineError::InvalidConfig(format!("配置 global.wallet.private_key 非法: {err}"))
        })?;
        return Ok(Arc::new(keypair));
    }

    Err(EngineError::InvalidConfig(
        "缺少私钥配置，请提供 global.wallet.private_key 或环境变量 GALILEO_PRIVATE_KEY".into(),
    ))
}

fn parse_keypair_string(raw: &str) -> Result<Keypair, anyhow::Error> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("keypair string empty");
    }

    if trimmed.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(trimmed)?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else if trimmed.contains(',') {
        let bytes = trimmed
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else {
        let data = bs58::decode(trimmed).into_vec()?;
        Ok(Keypair::try_from(data.as_slice())?)
    }
}
