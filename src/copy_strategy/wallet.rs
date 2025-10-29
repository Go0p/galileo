use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use serde_json::json;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_config::RpcTokenAccountsFilter;
use solana_client::rpc_request::RpcRequest;
use solana_client::rpc_response::{Response as RpcResponse, RpcKeyedAccount};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{debug, error, info, warn};
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;

use crate::config::{CopySourceKind, CopyWalletConfig, LanderSettings};
use crate::engine::{
    ComputeUnitPriceMode, DispatchStrategy, EngineIdentity, MultiLegInstructions,
    SwapInstructionsVariant, TransactionBuilder, TxVariantPlanner,
};
use crate::lander::{Deadline, LanderFactory, LanderStack};
use crate::monitoring::events;
use crate::network::IpAllocator;
use crate::txs::jupiter::types::JUPITER_V6_PROGRAM_ID;

use super::constants::MAX_SEEN_SIGNATURES;
use super::grpc::{YellowstoneTransactionClient, parse_transaction_update};
use super::transaction::filter_transaction;
use super::transaction::{
    CachedTokenAccount, RouteContext, TransactionLoadedAddresses, TransactionTokenBalances,
    apply_replacements, build_create_ata_instruction, collect_instruction_signers,
    decode_versioned_transaction, derive_associated_token_address, extract_compute_unit_limit,
    instructions_from_message, lookup_addresses, message_required_signatures,
    resolve_lookup_accounts, split_compute_budget,
};

pub(crate) struct CopyWalletRunner {
    wallet: CopyWalletConfig,
    wallet_pubkey: Pubkey,
    rpc_client: Arc<RpcClient>,
    tx_builder: TransactionBuilder,
    identity: EngineIdentity,
    lander_stack: Arc<LanderStack>,
    planner: TxVariantPlanner,
    landing_timeout: Duration,
    dispatch_strategy: DispatchStrategy,
    compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    dry_run: bool,
    seen_signatures: tokio::sync::Mutex<SeenSignatures>,
    intermediate_mints: Arc<HashSet<Pubkey>>,
    owned_token_accounts: tokio::sync::RwLock<HashMap<Pubkey, CachedTokenAccount>>,
}

struct ReplacementPlan {
    mapping: HashMap<Pubkey, Pubkey>,
    destination: AtaAssignment,
}

#[derive(Clone, Copy)]
struct AtaAssignment {
    mint: Pubkey,
    token_program: Pubkey,
    account: Pubkey,
    existed: bool,
}

impl CopyWalletRunner {
    pub async fn new(
        wallet: CopyWalletConfig,
        rpc_client: Arc<RpcClient>,
        tx_builder: TransactionBuilder,
        identity: EngineIdentity,
        ip_allocator: Arc<IpAllocator>,
        compute_unit_price_mode: Option<ComputeUnitPriceMode>,
        lander_factory: LanderFactory,
        lander_settings: LanderSettings,
        intermediate_mints: Arc<HashSet<Pubkey>>,
        landing_timeout: Duration,
        dispatch_strategy: DispatchStrategy,
        dry_run: bool,
    ) -> Result<Self> {
        let wallet_pubkey = Pubkey::from_str(wallet.address.trim())
            .map_err(|err| anyhow!("wallet address `{}` 解析失败: {err}", wallet.address))?;

        let enable_landers = if wallet.source.enable_landers.is_empty() {
            vec!["rpc".to_string()]
        } else {
            wallet
                .source
                .enable_landers
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let lander_stack = Arc::new(
            lander_factory
                .build_stack(
                    &lander_settings,
                    &enable_landers,
                    &["rpc"],
                    lander_settings.max_retries.unwrap_or(0),
                    ip_allocator,
                )
                .map_err(|err| anyhow!(err))?,
        );

        let owned_token_accounts =
            Self::load_existing_token_mints(&rpc_client, &identity.pubkey).await?;

        Ok(Self {
            wallet,
            wallet_pubkey,
            rpc_client,
            tx_builder,
            identity,
            lander_stack,
            planner: TxVariantPlanner::new(),
            landing_timeout,
            dispatch_strategy,
            compute_unit_price_mode,
            dry_run,
            seen_signatures: tokio::sync::Mutex::new(SeenSignatures::new(MAX_SEEN_SIGNATURES)),
            intermediate_mints,
            owned_token_accounts: tokio::sync::RwLock::new(owned_token_accounts),
        })
    }

    pub async fn run(self) -> Result<()> {
        match self.wallet.source.kind {
            CopySourceKind::Rpc => self.run_rpc().await,
            CopySourceKind::Grpc => self.run_grpc().await,
        }
    }

    async fn run_grpc(self) -> Result<()> {
        let grpc = &self.wallet.source.grpc;
        let endpoint = if !grpc.yellowstone_grpc_url.trim().is_empty() {
            grpc.yellowstone_grpc_url.trim().to_string()
        } else {
            return Err(anyhow!(
                "wallet {} 未配置 yellowstone_grpc_url",
                self.wallet.address
            ));
        };

        let token = grpc
            .yellowstone_grpc_token
            .trim()
            .parse::<AsciiMetadataValue>()
            .ok();

        let mut client = YellowstoneTransactionClient::connect(endpoint.clone(), token).await?;

        let mut include_ids: HashSet<Pubkey> = HashSet::new();
        if grpc.include_program_ids.is_empty() {
            include_ids.insert(JUPITER_V6_PROGRAM_ID);
        } else {
            for value in &grpc.include_program_ids {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    include_ids.insert(JUPITER_V6_PROGRAM_ID);
                } else if let Ok(pk) = Pubkey::from_str(trimmed) {
                    include_ids.insert(pk);
                }
            }
        }
        let exclude_ids: HashSet<Pubkey> = grpc
            .exclude_program_ids
            .iter()
            .filter_map(|value| Pubkey::from_str(value.trim()).ok())
            .collect();

        info!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            endpoint = %endpoint,
            include = ?include_ids,
            exclude = ?exclude_ids,
            "Yellowstone gRPC 订阅启动"
        );

        let mut stream = client
            .subscribe_transactions(self.wallet_pubkey)
            .await
            .context("订阅 Yellowstone gRPC 失败")?;

        while let Some(update) = stream.next().await.transpose()? {
            if let Some(info) = parse_transaction_update(&update) {
                let signature = match Signature::try_from(info.signature.as_slice()) {
                    Ok(sig) => sig,
                    Err(err) => {
                        warn!(
                            target: "strategy::copy",
                            wallet = %self.wallet_pubkey,
                            error = %err,
                            "签名解析失败，跳过"
                        );
                        continue;
                    }
                };
                if !self.should_process(&signature).await {
                    continue;
                }

                if let Some(proto_tx) = info.transaction.as_ref() {
                    let meta = match info.meta.as_ref() {
                        Some(meta) => meta,
                        None => {
                            debug!(
                                target: "strategy::copy",
                                wallet = %self.wallet_pubkey,
                                signature = %signature,
                                "Yellowstone meta 缺失，无法复原账户"
                            );
                            continue;
                        }
                    };
                    let loaded_addresses = match TransactionLoadedAddresses::try_from(meta) {
                        Ok(addresses) => addresses,
                        Err(err) => {
                            warn!(
                                target: "strategy::copy",
                                wallet = %self.wallet_pubkey,
                                signature = %signature,
                                error = %err,
                                "解析 Yellowstone loaded addresses 失败"
                            );
                            continue;
                        }
                    };
                    let token_balances = match TransactionTokenBalances::try_from(meta) {
                        Ok(balances) => balances,
                        Err(err) => {
                            warn!(
                                target: "strategy::copy",
                                wallet = %self.wallet_pubkey,
                                signature = %signature,
                                error = %err,
                                "解析 Yellowstone token balance 失败"
                            );
                            continue;
                        }
                    };
                    let versioned = decode_versioned_transaction(proto_tx)?;
                    if !filter_transaction(
                        &versioned,
                        &include_ids,
                        &exclude_ids,
                        Some(&loaded_addresses),
                    ) {
                        continue;
                    }
                    if let Err(err) = self
                        .replay_transaction(
                            &signature,
                            versioned,
                            Some(&token_balances),
                            Some(&loaded_addresses),
                            grpc.fanout_count.max(1),
                        )
                        .await
                    {
                        error!(
                            target: "strategy::copy",
                            wallet = %self.wallet_pubkey,
                            signature = %signature,
                            error = %err,
                            "复制交易失败"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_rpc(self) -> Result<()> {
        warn!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            "RPC mode 尚未实现，跳过"
        );
        Ok(())
    }

    async fn should_process(&self, signature: &Signature) -> bool {
        let mut guard = self.seen_signatures.lock().await;
        guard.insert(*signature)
    }

    async fn resolve_user_token_account(
        &self,
        mint: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<(Pubkey, bool)> {
        let ata = derive_associated_token_address(&self.identity.pubkey, mint, token_program)?;
        {
            let cache = self.owned_token_accounts.read().await;
            if let Some(entry) = cache.get(mint) {
                if entry.account == ata && entry.token_program == *token_program {
                    return Ok((ata, true));
                }
            }
        }
        let mut cache = self.owned_token_accounts.write().await;
        cache.insert(
            *mint,
            CachedTokenAccount {
                account: ata,
                token_program: *token_program,
            },
        );
        Ok((ata, false))
    }

    async fn replay_transaction(
        &self,
        signature: &Signature,
        transaction: VersionedTransaction,
        token_balances: Option<&TransactionTokenBalances>,
        loaded_addresses: Option<&TransactionLoadedAddresses>,
        fanout_count: u32,
    ) -> Result<()> {
        events::copy_transaction_captured(&self.wallet_pubkey, signature, fanout_count);

        let required_signers = message_required_signatures(&transaction.message);
        if required_signers > 1 {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                required_signers,
                "交易需要多个签名，直接跳过复制"
            );
            return Ok(());
        }

        let lookups = resolve_lookup_accounts(&self.rpc_client, &transaction.message).await?;
        let (instructions, account_keys) =
            instructions_from_message(&transaction.message, &lookups, loaded_addresses)
                .context("指令解析失败")?;
        let original_signers = collect_instruction_signers(&instructions);
        debug!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            signature = %signature,
            required_signers,
            original_signers = ?original_signers,
            "解析原始指令完成"
        );

        let mut route_ctx = match instructions.iter().find_map(RouteContext::from_instruction) {
            Some(ctx) => ctx,
            None => {
                debug!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    "未找到可复制的 Jupiter Route 指令"
                );
                return Ok(());
            }
        };

        route_ctx
            .populate_from_balances(&account_keys, token_balances)
            .context("填充 route token 信息失败")?;

        let plan = self
            .build_replacement_plan(&route_ctx, &account_keys, token_balances)
            .await?;

        let mut compute_budget_instructions = Vec::new();
        let mut jupiter_instructions = Vec::new();
        for ix in &instructions {
            if ix.program_id == super::constants::COMPUTE_BUDGET_PROGRAM_ID {
                compute_budget_instructions.push(ix.clone());
            } else if ix.program_id == crate::txs::jupiter::types::JUPITER_V6_PROGRAM_ID {
                jupiter_instructions.push(ix.clone());
            }
        }

        if jupiter_instructions.is_empty() {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                "Jupiter 指令为空，跳过复制"
            );
            return Ok(());
        }

        let original_accounts_snapshot = describe_jupiter_accounts(&jupiter_instructions);
        debug!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            signature = %signature,
            accounts = ?original_accounts_snapshot,
            "Jupiter 指令原始账户"
        );

        debug!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            signature = %signature,
            replacements = ?plan.mapping,
            "账户替换表已生成"
        );
        apply_replacements(&mut jupiter_instructions, &plan.mapping);

        let replaced_accounts_snapshot = describe_jupiter_accounts(&jupiter_instructions);
        debug!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            signature = %signature,
            accounts = ?replaced_accounts_snapshot,
            "Jupiter 指令替换后账户"
        );

        let mut patched_instructions =
            Vec::with_capacity(compute_budget_instructions.len() + jupiter_instructions.len() + 1);
        patched_instructions.extend(compute_budget_instructions);

        if !plan.destination.existed
            && !self.intermediate_mints.contains(&plan.destination.mint)
            && plan.destination.mint != Pubkey::default()
            && plan.destination.token_program != Pubkey::default()
        {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                destination_mint = %plan.destination.mint,
                ata = %plan.destination.account,
                "目标 ATA 未在缓存中，准备创建"
            );

            let ix = build_create_ata_instruction(
                &self.identity.pubkey,
                &self.identity.pubkey,
                &plan.destination.mint,
                &plan.destination.token_program,
            )?;
            patched_instructions.push(ix);

            let mut cache = self.owned_token_accounts.write().await;
            cache.insert(
                plan.destination.mint,
                CachedTokenAccount {
                    account: plan.destination.account,
                    token_program: plan.destination.token_program,
                },
            );
        }

        patched_instructions.extend(jupiter_instructions);
        let patched_signers = collect_instruction_signers(&patched_instructions);
        let identity_key = self.identity.pubkey;
        if !patched_signers.contains(&identity_key) {
            warn!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                identity = %identity_key,
                patched_signers = ?patched_signers,
                "替换后的指令 signer 不包含身份密钥，可能导致签名不足"
            );
        }
        debug!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            signature = %signature,
            identity = %identity_key,
            patched_signers = ?patched_signers,
            destination_exists = plan.destination.existed,
            "指令替换与 ATA 处理完成"
        );

        let (compute_budget, main_instructions) =
            split_compute_budget(&patched_instructions, self.compute_unit_price_mode.as_ref());
        let compute_unit_limit =
            extract_compute_unit_limit(&compute_budget).unwrap_or(crate::engine::FALLBACK_CU_LIMIT);

        let mut bundle = MultiLegInstructions::new(
            compute_budget,
            main_instructions,
            lookup_addresses(&transaction.message),
            lookups.clone(),
            None,
            compute_unit_limit,
        );
        bundle.dedup_lookup_tables();

        let final_sequence = bundle.flatten_instructions();
        let variant = SwapInstructionsVariant::MultiLeg(bundle);
        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &variant, final_sequence, 0)
            .await
            .map_err(|err| anyhow!("构建复制交易失败: {err}"))?;

        let mut layout = self.lander_stack.variant_layout(self.dispatch_strategy);
        if fanout_count > 1 {
            let factor = fanout_count as usize;
            for count in &mut layout {
                *count = count.saturating_mul(factor).max(1);
            }
        }

        let plan = self
            .planner
            .plan(self.dispatch_strategy, &prepared, &layout);

        if self.dry_run {
            let variants: usize = (0..layout.len())
                .map(|idx| plan.variants_for_lander(idx).len())
                .sum();
            info!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                variants,
                "dry-run 模式，跳过落地提交"
            );
            return Ok(());
        }

        let deadline = Deadline::from_instant(Instant::now() + self.landing_timeout);
        match self.lander_stack.submit_plan(&plan, deadline, "copy").await {
            Ok(receipt) => {
                events::copy_transaction_dispatched(&self.wallet_pubkey, signature, 0);
                info!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    lander = receipt.lander,
                    endpoint = %receipt.endpoint,
                    slot = receipt.slot,
                    "复制交易提交成功"
                );
            }
            Err(err) => {
                warn!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    error = %err,
                    "复制交易提交失败"
                );
            }
        }

        Ok(())
    }

    async fn build_replacement_plan(
        &self,
        route: &RouteContext,
        account_keys: &[Pubkey],
        balances: Option<&TransactionTokenBalances>,
    ) -> Result<ReplacementPlan> {
        let mut mapping = HashMap::new();
        mapping.insert(route.authority, self.identity.pubkey);

        let mut destination_assignment: Option<AtaAssignment> = None;

        if let Some(balances) = balances {
            for entry in balances.entries() {
                if entry.owner != Some(route.authority) {
                    continue;
                }
                let Some(old_account) = account_keys.get(entry.account_index) else {
                    warn!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        index = entry.account_index,
                        "token balance account index 越界，跳过"
                    );
                    continue;
                };
                if *old_account == route.authority {
                    continue;
                }
                let Some(token_program) = entry.token_program else {
                    warn!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        account = %old_account,
                        "token balance 缺少 token program，跳过替换"
                    );
                    continue;
                };
                let expected = match derive_associated_token_address(
                    &route.authority,
                    &entry.mint,
                    &token_program,
                ) {
                    Ok(ata) => ata,
                    Err(err) => {
                        warn!(
                            target: "strategy::copy",
                            wallet = %self.wallet_pubkey,
                            mint = %entry.mint,
                            token_program = %token_program,
                            error = %err,
                            "派生 copy 钱包 ATA 失败，跳过匹配"
                        );
                        continue;
                    }
                };

                if expected != *old_account {
                    continue;
                }

                let (replacement, existed) = self
                    .resolve_user_token_account(&entry.mint, &token_program)
                    .await?;
                mapping.entry(*old_account).or_insert(replacement);

                if *old_account == route.destination_ata {
                    destination_assignment.get_or_insert(AtaAssignment {
                        mint: entry.mint,
                        token_program,
                        account: replacement,
                        existed,
                    });
                }
            }
        }

        let destination = destination_assignment.unwrap_or(AtaAssignment {
            mint: Pubkey::default(),
            token_program: Pubkey::default(),
            account: Pubkey::default(),
            existed: true,
        });

        Ok(ReplacementPlan {
            mapping,
            destination,
        })
    }

    async fn load_existing_token_mints(
        rpc_client: &Arc<RpcClient>,
        owner: &Pubkey,
    ) -> Result<HashMap<Pubkey, CachedTokenAccount>> {
        let mut cached = HashMap::new();
        let program_ids = [
            Pubkey::new_from_array(spl_token::id().to_bytes()),
            Pubkey::new_from_array(spl_token_2022::id().to_bytes()),
        ];
        for program_id in program_ids {
            let filter = RpcTokenAccountsFilter::ProgramId(program_id.to_string());
            let config = RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::JsonParsed),
                commitment: Some(rpc_client.commitment()),
                data_slice: None,
                min_context_slot: None,
            };
            let params = json!([owner.to_string(), filter, config]);
            let response_accounts: RpcResponse<Vec<RpcKeyedAccount>> = rpc_client
                .send(RpcRequest::GetTokenAccountsByOwner, params)
                .await
                .map_err(|err| anyhow!("预检 token accounts 失败: {err}"))?;

            for keyed in response_accounts.value {
                let account_pubkey = Pubkey::from_str(&keyed.pubkey).map_err(|err| anyhow!(err))?;
                if let UiAccountData::Json(parsed) = &keyed.account.data {
                    let owner_str = parsed
                        .parsed
                        .get("info")
                        .and_then(|info| info.get("owner"))
                        .and_then(|owner| owner.as_str());
                    if owner_str.map(Pubkey::from_str).transpose()? != Some(*owner) {
                        continue;
                    }
                    if let Some(mint_str) = parsed
                        .parsed
                        .get("info")
                        .and_then(|info| info.get("mint"))
                        .and_then(|mint| mint.as_str())
                    {
                        let mint = Pubkey::from_str(mint_str).map_err(|err| anyhow!(err))?;
                        let token_program = Pubkey::from_str(keyed.account.owner.as_str())
                            .map_err(|err| anyhow!(err))?;
                        cached
                            .entry(mint)
                            .and_modify(|entry: &mut CachedTokenAccount| {
                                if entry.token_program != token_program {
                                    entry.token_program = token_program;
                                }
                                entry.account = account_pubkey;
                            })
                            .or_insert(CachedTokenAccount {
                                account: account_pubkey,
                                token_program,
                            });
                    }
                }
            }
        }

        Ok(cached)
    }
}

struct SeenSignatures {
    max: usize,
    deque: VecDeque<Signature>,
    set: HashSet<Signature>,
}

impl SeenSignatures {
    fn new(max: usize) -> Self {
        Self {
            max,
            deque: VecDeque::with_capacity(max),
            set: HashSet::with_capacity(max),
        }
    }

    fn insert(&mut self, signature: Signature) -> bool {
        if self.set.contains(&signature) {
            return false;
        }

        if self.deque.len() >= self.max {
            if let Some(old) = self.deque.pop_front() {
                self.set.remove(&old);
            }
        }

        self.deque.push_back(signature);
        self.set.insert(signature);
        true
    }
}

fn describe_jupiter_accounts(instructions: &[Instruction]) -> Vec<String> {
    let mut lines = Vec::new();

    for ix in instructions {
        if ix.program_id != JUPITER_V6_PROGRAM_ID {
            continue;
        }
        lines.push(ix.program_id.to_string());

        for (idx, meta) in ix.accounts.iter().enumerate() {
            let mut flags = Vec::new();
            if meta.is_writable {
                flags.push("Writable");
            }
            if meta.is_signer {
                flags.push("Signer");
            }
            let flag_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" {}", flags.join(" "))
            };
            lines.push(format!("#{} {}{}", idx + 1, meta.pubkey, flag_str));
        }
    }

    if lines.is_empty() {
        lines.push("no_jupiter_instructions".to_string());
    }

    lines
}
