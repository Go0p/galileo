use std::collections::{BTreeSet, HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use tracing::{info, warn};

use super::{EngineError, EngineIdentity, EngineResult};
use crate::flashloan::FlashloanError;
use crate::flashloan::marginfi::{
    MarginfiAccountRegistry, MarginfiFlashloanPreparation, build_initialize_instruction,
    find_marginfi_account_by_authority, marginfi_account_matches_authority,
};
use crate::strategy::types::TradePair;

static TOKEN_PROGRAM_ID: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
        .expect("invalid SPL token program id")
});
static TOKEN_2022_PROGRAM_ID: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
        .expect("invalid SPL token2022 program id")
});
static ASSOCIATED_TOKEN_PROGRAM_ID: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")
        .expect("invalid SPL associated token program id")
});
static SYSTEM_PROGRAM_ID: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str("11111111111111111111111111111111").expect("invalid system program id")
});

const MAX_RPC_BATCH_SIZE: usize = 100;
const CREATION_BATCH_SIZE: usize = 20;

fn derive_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let seeds: [&[u8]; 3] = [owner.as_ref(), token_program.as_ref(), mint.as_ref()];
    Pubkey::find_program_address(&seeds, &ASSOCIATED_TOKEN_PROGRAM_ID).0
}

fn build_create_associated_token_account_idempotent(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let associated = derive_associated_token_address(owner, mint, token_program);
    Instruction {
        program_id: *ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(associated, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(*token_program, false),
        ],
        data: vec![1u8],
    }
}

#[derive(Debug, Clone)]
struct MintCandidate {
    mint: Pubkey,
    mint_text: String,
    ata_for_token_keg: Pubkey,
    ata_for_token_2022: Pubkey,
}

#[derive(Debug, Clone)]
struct MintAccountState {
    mint: Pubkey,
    token_program: Pubkey,
    exists: bool,
}

#[derive(Debug)]
struct MarginfiCreationPlan {
    keypair: Keypair,
}

#[derive(Debug, Clone, Default)]
pub struct PrecheckSummary {
    pub total_mints: usize,
    pub processed_mints: usize,
    pub created_accounts: usize,
}

pub struct AccountPrechecker {
    rpc: Arc<RpcClient>,
    marginfi_accounts: MarginfiAccountRegistry,
}

impl AccountPrechecker {
    pub fn new(rpc: Arc<RpcClient>, marginfi_accounts: MarginfiAccountRegistry) -> Self {
        Self {
            rpc,
            marginfi_accounts,
        }
    }

    /// 确保套利所需的所有账户均已就绪：用户 ATA 与 Marginfi 闪电贷账户。
    pub async fn ensure_accounts(
        &self,
        identity: &EngineIdentity,
        trade_pairs: &[TradePair],
        flashloan_enabled: bool,
    ) -> EngineResult<(PrecheckSummary, Option<MarginfiFlashloanPreparation>)> {
        let candidates = self.collect_candidates(identity, trade_pairs)?;
        let (states, skipped) = self.classify_account_states(&candidates).await?;

        let mut summary = PrecheckSummary {
            total_mints: candidates.len(),
            processed_mints: states.len(),
            created_accounts: 0,
        };

        if skipped > 0 {
            warn!(
                target: "engine::precheck",
                skipped,
                "部分 Mint 未通过所有权校验，已跳过预检查"
            );
        }

        let missing_atas: Vec<&MintAccountState> =
            states.iter().filter(|state| !state.exists).collect();

        let mut instructions: Vec<Instruction> = Vec::with_capacity(missing_atas.len() + 1);
        for entry in &missing_atas {
            instructions.push(build_create_associated_token_account_idempotent(
                &identity.pubkey,
                &identity.pubkey,
                &entry.mint,
                &entry.token_program,
            ));
        }

        summary.created_accounts = missing_atas.len();

        let marginfi_mints = self.collect_marginfi_mints(trade_pairs);
        let (marginfi_plan, flashloan_preparation) = if flashloan_enabled {
            self.prepare_marginfi_plan(identity, &marginfi_mints, &mut instructions)
                .await?
        } else {
            (None, None)
        };

        if instructions.is_empty() {
            if summary.processed_mints > 0 {
                info!(
                    target: "engine::precheck",
                    processed = summary.processed_mints,
                    "所有关联账户已存在，预检查无新增"
                );
            }
            if let Some(prep) = &flashloan_preparation {
                info!(
                    target: "engine::precheck",
                    account = %prep.account,
                    created = prep.created,
                    "Marginfi flashloan account 预检完成"
                );
            }
            return Ok((summary, flashloan_preparation));
        }

        self.submit_creation_transactions(identity, &instructions, marginfi_plan.as_ref())
            .await?;

        if summary.created_accounts > 0 {
            info!(
                target: "engine::precheck",
                created = summary.created_accounts,
                processed = summary.processed_mints,
                "账户预检完成，已补齐缺失的关联账户"
            );
        }

        if let Some(prep) = &flashloan_preparation {
            info!(
                target: "engine::precheck",
                account = %prep.account,
                created = prep.created,
                "Marginfi flashloan account 预检完成"
            );
        }

        Ok((summary, flashloan_preparation))
    }

    fn collect_candidates(
        &self,
        identity: &EngineIdentity,
        trade_pairs: &[TradePair],
    ) -> EngineResult<Vec<MintCandidate>> {
        let mut unique_mints: BTreeSet<String> = BTreeSet::new();
        for pair in trade_pairs {
            let input = pair.input_mint.trim();
            let output = pair.output_mint.trim();
            if !input.is_empty() {
                unique_mints.insert(input.to_string());
            }
            if !output.is_empty() {
                unique_mints.insert(output.to_string());
            }
        }

        let owner = identity.pubkey;
        let mut candidates = Vec::with_capacity(unique_mints.len());
        for mint_text in unique_mints {
            match Pubkey::from_str(&mint_text) {
                Ok(mint) => {
                    let ata_normal =
                        derive_associated_token_address(&owner, &mint, &TOKEN_PROGRAM_ID);
                    let ata_2022 =
                        derive_associated_token_address(&owner, &mint, &TOKEN_2022_PROGRAM_ID);
                    candidates.push(MintCandidate {
                        mint,
                        mint_text,
                        ata_for_token_keg: ata_normal,
                        ata_for_token_2022: ata_2022,
                    });
                }
                Err(err) => {
                    warn!(
                        target: "engine::precheck",
                        mint = %mint_text,
                        error = %err,
                        "跳过非法的 Mint 配置"
                    );
                }
            }
        }

        Ok(candidates)
    }

    fn collect_marginfi_mints(&self, trade_pairs: &[TradePair]) -> BTreeSet<Pubkey> {
        trade_pairs.iter().map(|pair| pair.input_pubkey).collect()
    }

    async fn classify_account_states(
        &self,
        candidates: &[MintCandidate],
    ) -> EngineResult<(Vec<MintAccountState>, usize)> {
        let mut lookup_keys: Vec<Pubkey> = Vec::with_capacity(candidates.len() * 3);
        let mut dedup: HashSet<Pubkey> = HashSet::with_capacity(candidates.len() * 3);
        for candidate in candidates {
            for key in [
                candidate.mint,
                candidate.ata_for_token_keg,
                candidate.ata_for_token_2022,
            ] {
                if dedup.insert(key) {
                    lookup_keys.push(key);
                }
            }
        }

        let accounts = self.fetch_accounts(&lookup_keys).await?;

        let mut states = Vec::with_capacity(candidates.len());
        let mut skipped = 0usize;
        for candidate in candidates {
            let mint_account_opt = accounts.get(&candidate.mint).and_then(|acc| acc.as_ref());
            let Some(mint_account) = mint_account_opt else {
                warn!(
                    target: "engine::precheck",
                    mint = %candidate.mint_text,
                    "Mint 账户不存在，忽略预检查"
                );
                skipped += 1;
                continue;
            };

            let (token_program, associated_account) =
                if mint_account.owner == *TOKEN_2022_PROGRAM_ID {
                    (*TOKEN_2022_PROGRAM_ID, candidate.ata_for_token_2022)
                } else if mint_account.owner == *TOKEN_PROGRAM_ID {
                    (*TOKEN_PROGRAM_ID, candidate.ata_for_token_keg)
                } else {
                    warn!(
                        target: "engine::precheck",
                        mint = %candidate.mint_text,
                        owner = %mint_account.owner,
                        "Mint 所属程序不是已知的 SPL Token Program，跳过"
                    );
                    skipped += 1;
                    continue;
                };

            let exists = accounts
                .get(&associated_account)
                .and_then(|acc| acc.as_ref())
                .is_some();

            states.push(MintAccountState {
                mint: candidate.mint,
                token_program,
                exists,
            });
        }

        Ok((states, skipped))
    }

    async fn fetch_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> EngineResult<HashMap<Pubkey, Option<Account>>> {
        let mut result = HashMap::with_capacity(pubkeys.len());
        for chunk in pubkeys.chunks(MAX_RPC_BATCH_SIZE) {
            let accounts = self
                .rpc
                .get_multiple_accounts(chunk)
                .await
                .map_err(EngineError::Rpc)?;
            for (pubkey, account) in chunk.iter().copied().zip(accounts.into_iter()) {
                result.insert(pubkey, account);
            }
        }
        Ok(result)
    }

    async fn prepare_marginfi_plan(
        &self,
        identity: &EngineIdentity,
        required_mints: &BTreeSet<Pubkey>,
        instructions: &mut Vec<Instruction>,
    ) -> EngineResult<(
        Option<MarginfiCreationPlan>,
        Option<MarginfiFlashloanPreparation>,
    )> {
        if self.marginfi_accounts.has_per_mint_accounts() {
            let mut verified: HashSet<Pubkey> = HashSet::new();
            for mint in required_mints {
                let Some(account) = self.marginfi_accounts.per_mint().get(mint) else {
                    return Err(EngineError::InvalidConfig(format!(
                        "缺少 mint {mint} 对应的 flashloan.marginfi.marginfi_accounts 配置"
                    )));
                };
                if verified.insert(*account) {
                    let context = format!("flashloan.marginfi.marginfi_accounts[{mint}]");
                    self.verify_marginfi_account(identity, *account, &context)
                        .await?;
                }
            }
            // 额外校验用户配置但当前未启用的账户，提前发现权限问题
            for (&mint, &account) in self.marginfi_accounts.per_mint() {
                if verified.insert(account) {
                    let context = format!("flashloan.marginfi.marginfi_accounts[{mint}]");
                    self.verify_marginfi_account(identity, account, &context)
                        .await?;
                }
            }
            return Ok((None, None));
        }

        if let Some(configured) = self.marginfi_accounts.default() {
            self.verify_marginfi_account(
                identity,
                configured,
                "flashloan.marginfi.marginfi_account",
            )
            .await?;
            return Ok((
                None,
                Some(MarginfiFlashloanPreparation {
                    account: configured,
                    created: false,
                }),
            ));
        }

        let lookup = find_marginfi_account_by_authority(&self.rpc, &identity.pubkey).await;
        match lookup {
            Ok(Some(account)) => Ok((
                None,
                Some(MarginfiFlashloanPreparation {
                    account,
                    created: false,
                }),
            )),
            Ok(None) => {
                let keypair = Keypair::new();
                let instruction = build_initialize_instruction(keypair.pubkey(), &identity.pubkey)
                    .map_err(EngineError::from)?;
                instructions.insert(0, instruction);
                let preparation = MarginfiFlashloanPreparation {
                    account: keypair.pubkey(),
                    created: true,
                };
                Ok((Some(MarginfiCreationPlan { keypair }), Some(preparation)))
            }
            Err(FlashloanError::Rpc(err)) => {
                warn!(
                    target: "flashloan::marginfi",
                    error = %err,
                    "查询 marginfi account 失败，将直接尝试创建"
                );
                let keypair = Keypair::new();
                let instruction = build_initialize_instruction(keypair.pubkey(), &identity.pubkey)
                    .map_err(EngineError::from)?;
                instructions.insert(0, instruction);
                let preparation = MarginfiFlashloanPreparation {
                    account: keypair.pubkey(),
                    created: true,
                };
                Ok((Some(MarginfiCreationPlan { keypair }), Some(preparation)))
            }
            Err(other) => Err(EngineError::from(other)),
        }
    }

    async fn verify_marginfi_account(
        &self,
        identity: &EngineIdentity,
        account: Pubkey,
        context: &str,
    ) -> EngineResult<()> {
        let fetched = self
            .rpc
            .get_account(&account)
            .await
            .map_err(EngineError::Rpc)?;
        if marginfi_account_matches_authority(&fetched, &identity.pubkey) {
            Ok(())
        } else {
            Err(EngineError::InvalidConfig(format!(
                "{context} ({account}) 不属于当前钱包"
            )))
        }
    }

    async fn submit_creation_transactions(
        &self,
        identity: &EngineIdentity,
        instructions: &[Instruction],
        marginfi_plan: Option<&MarginfiCreationPlan>,
    ) -> EngineResult<()> {
        if instructions.is_empty() {
            return Ok(());
        }

        for (index, chunk) in instructions.chunks(CREATION_BATCH_SIZE).enumerate() {
            let blockhash = self
                .rpc
                .get_latest_blockhash()
                .await
                .map_err(EngineError::Rpc)?;
            let mut signer_refs: Vec<&dyn Signer> = vec![identity.signer.as_ref()];
            if index == 0 {
                if let Some(plan) = marginfi_plan {
                    signer_refs.push(&plan.keypair);
                }
            }
            let tx = Transaction::new_signed_with_payer(
                chunk,
                Some(&identity.pubkey),
                &signer_refs,
                blockhash,
            );
            let signature = self
                .rpc
                .send_and_confirm_transaction(&tx)
                .await
                .map_err(EngineError::Rpc)?;
            info!(
                target: "engine::precheck",
                instructions = chunk.len(),
                %signature,
                "账户预检交易已提交"
            );
        }
        Ok(())
    }
}
