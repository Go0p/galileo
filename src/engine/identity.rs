use std::env;
use std::str::FromStr;
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
    wrap_and_unwrap_sol: bool,
    use_shared_accounts: bool,
    compute_unit_price_micro_lamports: Option<u64>,
    skip_user_accounts_rpc_calls: bool,
    pub signer: Arc<Keypair>,
}

impl EngineIdentity {
    pub fn from_wallet(wallet: &WalletConfig) -> EngineResult<Self> {
        let signer = load_keypair(wallet)?;
        let pubkey = match env::var("GALILEO_USER_PUBKEY") {
            Ok(value) => Pubkey::from_str(value.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!("环境变量 GALILEO_USER_PUBKEY 非法: {err}"))
            })?,
            Err(env::VarError::NotPresent) => signer.pubkey(),
            Err(err) => {
                return Err(EngineError::InvalidConfig(format!(
                    "读取 GALILEO_USER_PUBKEY 失败: {err}"
                )));
            }
        };

        let fee_account = env::var("GALILEO_FEE_ACCOUNT")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let wrap_and_unwrap_sol = wallet.warp_or_unwrap_sol.wrap_and_unwrap_sol;

        let compute_unit_price_micro_lamports =
            match env::var("GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS") {
                Ok(value) => {
                    let parsed = value.trim().parse::<u64>().map_err(|err| {
                        EngineError::InvalidConfig(format!(
                            "环境变量 GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS 非法: {err}"
                        ))
                    })?;
                    Some(parsed)
                }
                Err(env::VarError::NotPresent) => {
                    let configured = wallet.warp_or_unwrap_sol.compute_unit_price_micro_lamports;
                    if configured > 0 {
                        Some(configured)
                    } else {
                        None
                    }
                }
                Err(err) => {
                    return Err(EngineError::InvalidConfig(format!(
                        "读取 GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS 失败: {err}"
                    )));
                }
            };

        let use_shared_accounts = parse_env_bool("GALILEO_USE_SHARED_ACCOUNTS")?.unwrap_or(false);
        let skip_user_accounts_rpc_calls = parse_env_bool("GALILEO_SKIP_USER_ACCOUNTS_RPC_CALLS")?
            .unwrap_or(wallet.warp_or_unwrap_sol.skip_user_accounts_rpc_calls);

        Ok(Self {
            pubkey,
            fee_account,
            wrap_and_unwrap_sol,
            use_shared_accounts,
            compute_unit_price_micro_lamports,
            skip_user_accounts_rpc_calls,
            signer,
        })
    }

    pub fn fee_account(&self) -> Option<&str> {
        self.fee_account.as_deref()
    }

    pub fn wrap_and_unwrap_sol(&self) -> bool {
        self.wrap_and_unwrap_sol
    }

    pub fn use_shared_accounts(&self) -> bool {
        self.use_shared_accounts
    }

    pub fn compute_unit_price_override(&self) -> Option<u64> {
        self.compute_unit_price_micro_lamports
    }

    pub fn skip_user_accounts_rpc_calls(&self) -> bool {
        self.skip_user_accounts_rpc_calls
    }
}

fn parse_env_bool(var: &str) -> EngineResult<Option<bool>> {
    match env::var(var) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "1" | "yes" => Ok(Some(true)),
                "false" | "0" | "no" => Ok(Some(false)),
                _ => Err(EngineError::InvalidConfig(format!(
                    "{var} 环境变量期望是布尔值，实际为 {value}"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(EngineError::InvalidConfig(format!(
            "读取 {var} 失败: {err}"
        ))),
    }
}

fn load_keypair(wallet: &WalletConfig) -> EngineResult<Arc<Keypair>> {
    if let Ok(value) = env::var("GALILEO_PRIVATE_KEY") {
        if !value.trim().is_empty() {
            let keypair = parse_keypair_string(value.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!("环境变量 GALILEO_PRIVATE_KEY 非法: {err}"))
            })?;
            return Ok(Arc::new(keypair));
        }
    }

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
