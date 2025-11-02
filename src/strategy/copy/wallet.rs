use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use bincode::serde::decode_from_slice;
use futures::StreamExt;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_system_interface::instruction::SystemInstruction;
use tracing::{debug, error, info, warn};
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;

use crate::config::{
    CopyDispatchConfig, CopyDispatchMode, CopySourceKind, CopyWalletConfig, LanderSettings,
};
use crate::engine::assembly::decorators::{
    ComputeBudgetDecorator, GuardBudgetDecorator, TipDecorator,
};
use crate::engine::assembly::{AssemblyContext, DecoratorChain, InstructionBundle};
use crate::engine::{
    ComputeUnitPriceMode, DispatchStrategy, EngineIdentity, MultiLegInstructions,
    SwapInstructionsVariant, TransactionBuilder, TxVariantPlanner,
};
use crate::instructions::jupiter::types::JUPITER_V6_PROGRAM_ID;
use crate::lander::{Deadline, LanderFactory, LanderStack};
use crate::monitoring::events;
use crate::network::IpAllocator;
use crate::wallet::WalletStateManager;
use flume::{self, Receiver, Sender, TrySendError};
use parking_lot::Mutex as ParkingMutex;

use super::constants::{MAX_SEEN_SIGNATURES, SYSTEM_PROGRAM_ID};
use super::grpc::{YellowstoneTransactionClient, parse_transaction_update};
use super::transaction::filter_transaction;
use super::transaction::{
    RouteContext, TransactionLoadedAddresses, TransactionTokenBalances, apply_replacements,
    build_create_ata_instruction, collect_instruction_signers, decode_versioned_transaction,
    derive_associated_token_address, extract_compute_unit_limit, instructions_from_message,
    lookup_addresses, message_required_signatures, read_route_in_amount,
    read_route_quoted_out_amount, resolve_lookup_accounts, split_compute_budget,
    update_route_in_amount, update_route_quoted_out_amount,
};
pub(crate) struct CopyWalletRunner {
    wallet: CopyWalletConfig,
    dispatch: CopyDispatchConfig,
    wallet_pubkey: Pubkey,
    rpc_client: Arc<RpcClient>,
    tx_builder: TransactionBuilder,
    identity: EngineIdentity,
    lander_stack: Arc<LanderStack>,
    planner: TxVariantPlanner,
    landing_timeout: Duration,
    dispatch_strategy: DispatchStrategy,
    compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    cu_limit_multiplier: f64,
    dry_run: bool,
    seen_signatures: tokio::sync::Mutex<SeenSignatures>,
    wallet_state: Arc<WalletStateManager>,
}

struct CopyTask {
    signature: Signature,
    transaction: VersionedTransaction,
    token_balances: Option<TransactionTokenBalances>,
    loaded_addresses: Option<TransactionLoadedAddresses>,
}

struct CopyTaskQueue {
    sender: ParkingMutex<Option<Sender<CopyTask>>>,
    receiver: Receiver<CopyTask>,
    wallet: Pubkey,
}

const BASE_GUARD_LAMPORTS: u64 = 5_000;
const TEMP_WALLET_TIP_DEDUCTION_LAMPORTS: u64 = LAMPORTS_PER_SOL / 1_000;

impl CopyTaskQueue {
    fn new(capacity: usize, wallet: Pubkey) -> Self {
        let (sender, receiver) = flume::bounded(capacity);
        let queue = Self {
            sender: ParkingMutex::new(Some(sender)),
            receiver,
            wallet,
        };
        queue.record_depth();
        queue
    }

    async fn push(&self, task: CopyTask) -> bool {
        let sender_opt = {
            let guard = self.sender.lock();
            guard.clone()
        };

        let Some(sender) = sender_opt else {
            self.record_depth();
            return true;
        };

        let mut dropped = false;
        match sender.try_send(task) {
            Ok(()) => {}
            Err(TrySendError::Full(task)) => {
                dropped = true;
                // 丢弃最旧任务，再尝试发送。
                let _ = self.receiver.try_recv();
                self.record_drop();
                if let Err(err) = sender.send_async(task).await {
                    debug!(
                        target: "strategy::copy",
                        error = %err,
                        "复制任务队列在发送时被关闭"
                    );
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                debug!(
                    target: "strategy::copy",
                    "复制任务队列已关闭，忽略新任务"
                );
                dropped = true;
            }
        }
        self.record_depth();
        dropped
    }

    async fn pop(&self) -> Option<CopyTask> {
        let result = self.receiver.recv_async().await;
        self.record_depth();
        match result {
            Ok(task) => Some(task),
            Err(_) => None,
        }
    }

    fn close(&self) {
        {
            let mut guard = self.sender.lock();
            guard.take();
        }
        self.record_depth();
    }

    fn record_depth(&self) {
        events::copy_queue_depth(&self.wallet, self.receiver.len());
    }

    fn record_drop(&self) {
        events::copy_queue_task_dropped(&self.wallet);
    }
}

struct ReplacementPlan {
    mapping: HashMap<Pubkey, Pubkey>,
    pending_atas: Vec<AtaAssignment>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct AtaAssignment {
    mint: Pubkey,
    token_program: Pubkey,
    account: Pubkey,
    existed: bool,
}

#[derive(Clone, Copy, Debug)]
struct BaseMintInfo {
    mint: Pubkey,
    token_program: Pubkey,
    pre_amount: u64,
    post_amount: u64,
}

impl BaseMintInfo {
    fn delta(&self) -> i128 {
        self.post_amount as i128 - self.pre_amount as i128
    }
}

enum AmountAdjustment {
    NotNeeded,
    Applied {
        original_in: u64,
        adjusted_in: u64,
        original_out: u64,
        adjusted_out: u64,
        mint: Pubkey,
        available: u64,
    },
    Skip,
}

impl CopyWalletRunner {
    pub async fn new(
        wallet: CopyWalletConfig,
        dispatch: CopyDispatchConfig,
        rpc_client: Arc<RpcClient>,
        tx_builder: TransactionBuilder,
        identity: EngineIdentity,
        ip_allocator: Arc<IpAllocator>,
        compute_unit_price_mode: Option<ComputeUnitPriceMode>,
        lander_factory: LanderFactory,
        lander_settings: LanderSettings,
        landing_timeout: Duration,
        dispatch_strategy: DispatchStrategy,
        wallet_refresh_interval: Option<Duration>,
        dry_run: bool,
    ) -> Result<Self> {
        let wallet_pubkey = Pubkey::from_str(wallet.address.trim())
            .map_err(|err| anyhow!("wallet address `{}` 解析失败: {err}", wallet.address))?;

        let enable_landers = if dry_run {
            if wallet.source.enable_landers.is_empty() {
                vec!["rpc".to_string()]
            } else {
                wallet
                    .source
                    .enable_landers
                    .iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
        } else if wallet.source.enable_landers.is_empty() {
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

        let mut lander_stack = lander_factory
            .build_stack(
                &lander_settings,
                &enable_landers,
                &["rpc"],
                lander_settings.max_retries.unwrap_or(0),
                ip_allocator,
            )
            .map_err(|err| anyhow!(err))?;
        if dry_run {
            lander_stack = lander_stack.into_rpc_only();
        }
        let lander_stack = Arc::new(lander_stack);

        let cu_limit_multiplier = if wallet.cu_limit_multiplier <= 0.0 {
            warn!(
                target: "strategy::copy",
                wallet = %wallet_pubkey,
                configured = wallet.cu_limit_multiplier,
                "cu_limit_multiplier 必须为正数，回退为 1.0"
            );
            1.0
        } else {
            wallet.cu_limit_multiplier
        };

        let wallet_state =
            WalletStateManager::new(rpc_client.clone(), identity.pubkey, wallet_refresh_interval)
                .await
                .map_err(|err| anyhow!("初始化钱包状态失败: {err}"))?;

        Ok(Self {
            wallet,
            dispatch,
            wallet_pubkey,
            rpc_client,
            tx_builder,
            identity,
            lander_stack,
            planner: TxVariantPlanner::new(),
            landing_timeout,
            dispatch_strategy,
            compute_unit_price_mode,
            cu_limit_multiplier,
            dry_run,
            seen_signatures: tokio::sync::Mutex::new(SeenSignatures::new(MAX_SEEN_SIGNATURES)),
            wallet_state,
        })
    }

    pub async fn run(self) -> Result<()> {
        match self.wallet.source.kind {
            CopySourceKind::Rpc => self.run_rpc().await,
            CopySourceKind::Grpc => self.run_grpc().await,
        }
    }

    async fn run_grpc(self) -> Result<()> {
        let runner = Arc::new(self);
        CopyWalletRunner::run_grpc_internal(runner).await
    }

    async fn run_grpc_internal(self: Arc<Self>) -> Result<()> {
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

        let fanout_count = self.dispatch.fanout_count.max(1);
        let replay_interval = Duration::from_millis(self.dispatch.replay_interval_ms);
        let max_inflight = usize::try_from(self.dispatch.max_inflight.max(1)).unwrap_or(1);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_inflight));

        let mut last_replay_at: Option<Instant> = None;

        let queue_resources = if matches!(self.dispatch.mode, CopyDispatchMode::Queued) {
            let capacity = usize::try_from(self.dispatch.queue_capacity.max(1))
                .unwrap_or(1)
                .max(1);
            let queue = Arc::new(CopyTaskQueue::new(capacity, self.wallet_pubkey));
            let worker_count = usize::try_from(self.dispatch.queue_worker_count.max(1))
                .unwrap_or(1)
                .max(1);
            events::copy_queue_workers(&self.wallet_pubkey, worker_count);

            let queue_interval = Duration::from_millis(self.dispatch.queue_send_interval_ms);
            let mut handles = Vec::with_capacity(worker_count);

            for _ in 0..worker_count {
                let runner = Arc::clone(&self);
                let semaphore = Arc::clone(&semaphore);
                let worker_queue = Arc::clone(&queue);
                let handle = tokio::spawn(async move {
                    let mut last_sent_at: Option<Instant> = None;
                    while let Some(task) = worker_queue.pop().await {
                        if queue_interval != Duration::ZERO {
                            let now = Instant::now();
                            if let Some(prev) = last_sent_at {
                                let target = prev + queue_interval;
                                if now < target {
                                    tokio::time::sleep(target - now).await;
                                }
                            }
                            last_sent_at = Some(Instant::now());
                        }

                        CopyWalletRunner::spawn_replay_task(
                            Arc::clone(&runner),
                            Arc::clone(&semaphore),
                            task,
                            fanout_count,
                        );
                    }
                });
                handles.push(handle);
            }
            Some((queue, handles))
        } else {
            None
        };

        let task_queue = queue_resources.as_ref().map(|(queue, _)| Arc::clone(queue));

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
                    if replay_interval != Duration::ZERO {
                        let now = Instant::now();
                        if let Some(prev) = last_replay_at {
                            let target = prev + replay_interval;
                            if now < target {
                                tokio::time::sleep(target - now).await;
                            }
                        }
                        last_replay_at = Some(Instant::now());
                    }

                    let task = CopyTask {
                        signature,
                        transaction: versioned,
                        token_balances: Some(token_balances),
                        loaded_addresses: Some(loaded_addresses),
                    };

                    if let Some(queue) = task_queue.as_ref() {
                        let dropped = queue.push(task).await;
                        if dropped {
                            warn!(
                                target: "strategy::copy",
                                wallet = %self.wallet_pubkey,
                                signature = %signature,
                                "copy 队列已满，已丢弃最旧任务以保留最新交易"
                            );
                        }
                    } else {
                        CopyWalletRunner::spawn_replay_task(
                            Arc::clone(&self),
                            Arc::clone(&semaphore),
                            task,
                            fanout_count,
                        );
                    }
                }
            }
        }

        if let Some((queue, handles)) = queue_resources {
            queue.close();
            for handle in handles {
                let _ = handle.await;
            }
        }

        Ok(())
    }

    fn spawn_replay_task(
        runner: Arc<Self>,
        semaphore: Arc<tokio::sync::Semaphore>,
        task: CopyTask,
        fanout_count: u32,
    ) {
        tokio::spawn(async move {
            let permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(err) => {
                    error!(
                        target: "strategy::copy",
                        wallet = %runner.wallet_pubkey,
                        error = %err,
                        "获取 copy 并发信号量失败"
                    );
                    return;
                }
            };
            let _permit = permit;

            let CopyTask {
                signature,
                transaction,
                token_balances,
                loaded_addresses,
            } = task;

            if let Err(err) = runner
                .replay_transaction(
                    &signature,
                    transaction,
                    token_balances.as_ref(),
                    loaded_addresses.as_ref(),
                    fanout_count.max(1),
                )
                .await
            {
                error!(
                    target: "strategy::copy",
                    wallet = %runner.wallet_pubkey,
                    signature = %signature,
                    error = %err,
                    "复制交易失败"
                );
            }
        });
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
        if let Some(account) = self.wallet_state.get_account(mint) {
            if account.account == ata && account.token_program == *token_program {
                return Ok((ata, true));
            }
        }
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

        let route_ctx = match instructions.iter().find_map(RouteContext::from_instruction) {
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

        let Some(token_balances) = token_balances else {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                "缺少 pre token balance 数据，跳过复制"
            );
            return Ok(());
        };

        let replacement = self
            .build_replacement_plan(&route_ctx, &account_keys, token_balances)
            .await?;

        let mut compute_budget_instructions = Vec::new();
        let mut jupiter_instructions = Vec::new();
        for ix in &instructions {
            if ix.program_id == super::constants::COMPUTE_BUDGET_PROGRAM_ID {
                compute_budget_instructions.push(ix.clone());
            } else if ix.program_id == crate::instructions::jupiter::types::JUPITER_V6_PROGRAM_ID {
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

        match self
            .adjust_route_amounts(
                &route_ctx.authority,
                token_balances,
                &mut jupiter_instructions,
            )
            .await?
        {
            AmountAdjustment::Skip => {
                debug!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    "base mint 余额不足，已跳过复制"
                );
                return Ok(());
            }
            AmountAdjustment::Applied {
                original_in,
                adjusted_in,
                original_out,
                adjusted_out,
                mint,
                available,
            } => {
                debug!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    mint = %mint,
                    original_in,
                    adjusted_in,
                    original_out,
                    adjusted_out,
                    available,
                    "已根据身份余额调整 route 金额"
                );
            }
            AmountAdjustment::NotNeeded => {}
        }

        let base_mint = self
            .detect_base_mint(&route_ctx.authority, token_balances)
            .map(|info| info.mint);

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
            replacements = ?replacement.mapping,
            "账户替换表已生成"
        );
        apply_replacements(&mut jupiter_instructions, &replacement.mapping);

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

        let mut scheduled_accounts = HashSet::new();
        for assignment in &replacement.pending_atas {
            if assignment.existed
                || assignment.mint == Pubkey::default()
                || assignment.token_program == Pubkey::default()
            {
                continue;
            }
            if !scheduled_accounts.insert(assignment.account) {
                continue;
            }

            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                mint = %assignment.mint,
                ata = %assignment.account,
                "检测到缺失 ATA，准备创建"
            );

            let ix = build_create_ata_instruction(
                &self.identity.pubkey,
                &self.identity.pubkey,
                &assignment.mint,
                &assignment.token_program,
            )?;
            patched_instructions.push(ix);
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
            "指令替换与 ATA 处理完成"
        );

        let (compute_budget, main_instructions, sampled_price) =
            split_compute_budget(&patched_instructions, self.compute_unit_price_mode.as_ref());
        let raw_compute_unit_limit =
            extract_compute_unit_limit(&compute_budget).unwrap_or(crate::engine::FALLBACK_CU_LIMIT);
        let compute_unit_limit = self.scale_compute_unit_limit(raw_compute_unit_limit);

        let transfer_info = Self::detect_temp_wallet_transfer(&instructions, &route_ctx.authority);
        let mut tip_lamports = 0;
        if let Some((temp_wallet, raw_lamports)) = transfer_info {
            match raw_lamports.checked_sub(TEMP_WALLET_TIP_DEDUCTION_LAMPORTS) {
                Some(adjusted) if adjusted > 0 => {
                    tip_lamports = adjusted;
                    debug!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        signature = %signature,
                        temp_wallet = %temp_wallet,
                        raw_lamports,
                        tip_lamports,
                        deduction = TEMP_WALLET_TIP_DEDUCTION_LAMPORTS,
                        "已解析临时钱包转账金额并用于 Jito tip"
                    );
                }
                Some(_) => {
                    debug!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        signature = %signature,
                        temp_wallet = %temp_wallet,
                        raw_lamports,
                        deduction = TEMP_WALLET_TIP_DEDUCTION_LAMPORTS,
                        "临时钱包转账金额扣除 0.001 SOL 后为 0，跳过 tip"
                    );
                }
                None => {
                    debug!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        signature = %signature,
                        temp_wallet = %temp_wallet,
                        raw_lamports,
                        deduction = TEMP_WALLET_TIP_DEDUCTION_LAMPORTS,
                        "临时钱包转账金额不足 0.001 SOL，跳过 tip"
                    );
                }
            }
        }

        let mut jito_tip_plan = self.lander_stack.draw_jito_tip_plan();
        if tip_lamports > 0 {
            if let Some(plan) = jito_tip_plan.as_mut() {
                plan.lamports = tip_lamports;
            } else {
                debug!(
                    target: "strategy::copy",
                    wallet = %self.wallet_pubkey,
                    signature = %signature,
                    tip_lamports,
                    "检测到临时钱包转账，但未启用 Jito tip plan，tip 指令将被跳过"
                );
                tip_lamports = 0;
            }
        } else {
            jito_tip_plan = None;
        }

        let mut multi_leg = MultiLegInstructions::new(
            compute_budget.clone(),
            main_instructions.clone(),
            lookup_addresses(&transaction.message),
            lookups.clone(),
            None,
            raw_compute_unit_limit,
        );
        multi_leg.dedup_lookup_tables();

        let mut initial_instructions = compute_budget.clone();
        initial_instructions.extend(main_instructions.clone());
        let mut bundle = InstructionBundle::from_instructions(initial_instructions);
        bundle.set_lookup_tables(
            multi_leg.address_lookup_table_addresses.clone(),
            multi_leg.resolved_lookup_tables.clone(),
        );

        let mut variant = SwapInstructionsVariant::MultiLeg(multi_leg);

        let mut assembly_ctx = AssemblyContext::new(&self.identity);
        assembly_ctx.base_mint = base_mint.as_ref();
        assembly_ctx.compute_unit_limit = compute_unit_limit;
        assembly_ctx.compute_unit_price = sampled_price;
        assembly_ctx.guard_required = BASE_GUARD_LAMPORTS;
        assembly_ctx.tip_lamports = tip_lamports;
        assembly_ctx.jito_tip_budget = tip_lamports;
        assembly_ctx.jito_tip_plan = jito_tip_plan.clone();
        assembly_ctx.variant = Some(&mut variant);

        let mut decorators = DecoratorChain::new();
        decorators.register(ComputeBudgetDecorator);
        decorators.register(TipDecorator);
        decorators.register(GuardBudgetDecorator);

        if let Err(err) = decorators.apply_all(&mut bundle, &mut assembly_ctx).await {
            return Err(anyhow!("复制交易装配失败: {err}"));
        }

        if let SwapInstructionsVariant::MultiLeg(inner) = &mut variant {
            bundle.set_lookup_tables(
                inner.address_lookup_table_addresses.clone(),
                inner.resolved_lookup_tables.clone(),
            );
        }

        let final_sequence = bundle.flatten();
        let prepared = self
            .tx_builder
            .build_with_sequence(
                &self.identity,
                &variant,
                final_sequence,
                tip_lamports,
                jito_tip_plan.clone(),
            )
            .await
            .map_err(|err| anyhow!("构建复制交易失败: {err}"))?;

        let mut layout = self.lander_stack.variant_layout(self.dispatch_strategy);
        if fanout_count > 1 {
            let factor = fanout_count as usize;
            for count in &mut layout {
                *count = count.saturating_mul(factor).max(1);
            }
        }

        let dispatch_plan = self
            .planner
            .plan(self.dispatch_strategy, &prepared, &layout);

        if self.dry_run {
            let variants: usize = (0..layout.len())
                .map(|idx| dispatch_plan.variants_for_lander(idx).len())
                .sum();
            info!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                variants,
                "dry-run 模式：复制交易将提交至覆盖的 RPC 端点"
            );
        }

        let deadline = Deadline::from_instant(Instant::now() + self.landing_timeout);
        match self
            .lander_stack
            .submit_plan(&dispatch_plan, deadline, "copy")
            .await
        {
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
        balances: &TransactionTokenBalances,
    ) -> Result<ReplacementPlan> {
        let mut mapping = HashMap::new();
        mapping.insert(route.authority, self.identity.pubkey);

        let mut pending_atas = Vec::new();

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

            if !existed {
                if !pending_atas
                    .iter()
                    .any(|item: &AtaAssignment| item.account == replacement)
                {
                    pending_atas.push(AtaAssignment {
                        mint: entry.mint,
                        token_program,
                        account: replacement,
                        existed,
                    });
                }
            }
        }

        Ok(ReplacementPlan {
            mapping,
            pending_atas,
        })
    }

    fn detect_base_mint(
        &self,
        authority: &Pubkey,
        balances: &TransactionTokenBalances,
    ) -> Option<BaseMintInfo> {
        balances
            .entries()
            .filter(|entry| entry.owner == Some(*authority))
            .filter_map(|entry| {
                let pre = entry.pre_amount.unwrap_or(0);
                let post = entry.post_amount.unwrap_or(0);
                if post > pre {
                    let token_program = entry.token_program.unwrap_or_else(|| spl_token::id());
                    Some(BaseMintInfo {
                        mint: entry.mint,
                        token_program,
                        pre_amount: pre,
                        post_amount: post,
                    })
                } else {
                    None
                }
            })
            .max_by_key(|info| info.delta())
    }

    fn scale_compute_unit_limit(&self, raw: u32) -> u32 {
        if self.cu_limit_multiplier == 1.0 {
            return raw.max(1);
        }
        let scaled = ((raw as f64) * self.cu_limit_multiplier).round();
        let scaled = scaled.max(1.0).min(u32::MAX as f64);
        let scaled_u32 = scaled as u32;
        if scaled_u32 != raw {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                original = raw,
                scaled = scaled_u32,
                multiplier = self.cu_limit_multiplier,
                "compute unit limit 已按系数调整"
            );
        }
        scaled_u32.max(1)
    }

    async fn adjust_route_amounts(
        &self,
        authority: &Pubkey,
        token_balances: &TransactionTokenBalances,
        instructions: &mut [Instruction],
    ) -> Result<AmountAdjustment> {
        let Some(base) = self.detect_base_mint(authority, token_balances) else {
            return Ok(AmountAdjustment::NotNeeded);
        };

        let (base_ata, existed) = self
            .resolve_user_token_account(&base.mint, &base.token_program)
            .await?;
        if !existed {
            debug!(
                target: "strategy::copy",
                mint = %base.mint,
                "身份缺少 base mint ATA，跳过复制"
            );
            return Ok(AmountAdjustment::Skip);
        }

        let available = self
            .wallet_state
            .get_account(&base.mint)
            .and_then(|account| {
                if account.account == base_ata {
                    account.balance
                } else {
                    None
                }
            })
            .unwrap_or(0);

        if available == 0 {
            debug!(
                target: "strategy::copy",
                mint = %base.mint,
                "base mint 可用余额为 0，跳过复制"
            );
            return Ok(AmountAdjustment::Skip);
        }

        let mut target_in: Option<u64> = None;
        let mut target_out: Option<u64> = None;
        let mut original_in: Option<u64> = None;
        let mut original_out: Option<u64> = None;
        let mut updated = false;
        let mut encountered = false;

        for instruction in instructions.iter_mut() {
            if instruction.program_id != JUPITER_V6_PROGRAM_ID {
                continue;
            }

            let kind = crate::instructions::jupiter::parser::classify(&instruction.data);
            let current_in = match read_route_in_amount(kind, &instruction.data) {
                Some(value) => value,
                None => continue,
            };
            let current_out = match read_route_quoted_out_amount(kind, &instruction.data) {
                Some(value) => value,
                None => continue,
            };

            encountered = true;
            if target_in.is_none() {
                if current_out < current_in {
                    debug!(
                        target: "strategy::copy",
                        mint = %base.mint,
                        current_in,
                        current_out,
                        "copy 指令净收益为负，跳过复制"
                    );
                    return Ok(AmountAdjustment::Skip);
                }

                let profit = current_out - current_in;
                let capped_in = current_in.min(available);
                if capped_in == 0 {
                    return Ok(AmountAdjustment::Skip);
                }
                let desired_out = match capped_in.checked_add(profit) {
                    Some(value) => value,
                    None => {
                        debug!(
                            target: "strategy::copy",
                            mint = %base.mint,
                            profit,
                            "base mint 裁剪导致金额溢出，跳过复制"
                        );
                        return Ok(AmountAdjustment::Skip);
                    }
                };
                target_in = Some(capped_in);
                target_out = Some(desired_out);
                original_in = Some(current_in);
                original_out = Some(current_out);
            }

            let desired_in = target_in.expect("target amount must exist");
            let desired_out = target_out.expect("target out must exist");

            if desired_in != current_in {
                if let Err(err) = update_route_in_amount(kind, &mut instruction.data, desired_in) {
                    warn!(
                        target: "strategy::copy",
                        mint = %base.mint,
                        error = %err,
                        "调整 route 指令金额失败，跳过此次复制"
                    );
                    return Ok(AmountAdjustment::Skip);
                }
                updated = true;
            }
            if desired_out != current_out {
                if let Err(err) =
                    update_route_quoted_out_amount(kind, &mut instruction.data, desired_out)
                {
                    warn!(
                        target: "strategy::copy",
                        mint = %base.mint,
                        error = %err,
                        "调整 route quoted_out_amount 失败，跳过此次复制"
                    );
                    return Ok(AmountAdjustment::Skip);
                }
                updated = true;
            }
        }

        if !encountered {
            return Ok(AmountAdjustment::NotNeeded);
        }
        if !updated {
            return Ok(AmountAdjustment::NotNeeded);
        }

        if let Some(adjusted_in) = target_in {
            if adjusted_in == 0 {
                return Ok(AmountAdjustment::Skip);
            }
            return Ok(AmountAdjustment::Applied {
                original_in: original_in.unwrap_or(adjusted_in),
                adjusted_in,
                original_out: original_out.unwrap_or(target_out.unwrap_or(adjusted_in)),
                adjusted_out: target_out.unwrap_or(adjusted_in),
                mint: base.mint,
                available,
            });
        }

        Ok(AmountAdjustment::NotNeeded)
    }

    fn detect_temp_wallet_transfer(
        instructions: &[Instruction],
        authority: &Pubkey,
    ) -> Option<(Pubkey, u64)> {
        instructions
            .iter()
            .filter(|ix| ix.program_id == SYSTEM_PROGRAM_ID)
            .filter_map(|ix| {
                if ix.accounts.len() < 2 {
                    return None;
                }
                if ix.accounts.first()?.pubkey != *authority {
                    return None;
                }
                let destination = ix.accounts.get(1)?.pubkey;
                let lamports = Self::decode_system_transfer_lamports(&ix.data)?;
                Some((destination, lamports))
            })
            .max_by_key(|(_, lamports)| *lamports)
    }

    fn decode_system_transfer_lamports(data: &[u8]) -> Option<u64> {
        let config = bincode::config::standard()
            .with_fixed_int_encoding()
            .with_little_endian();
        let (instruction, _) = decode_from_slice::<SystemInstruction, _>(data, config).ok()?;
        match instruction {
            SystemInstruction::Transfer { lamports }
            | SystemInstruction::TransferWithSeed { lamports, .. } => Some(lamports),
            _ => None,
        }
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
