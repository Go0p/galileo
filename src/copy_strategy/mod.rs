use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use futures::{StreamExt, TryStreamExt};
use rand::Rng;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_message::compiled_instruction::CompiledInstruction;
use solana_message::legacy::Message as LegacyMessage;
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::{Message as V0Message, MessageAddressTableLookup};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use yellowstone_grpc_proto::geyser::geyser_client::GeyserClient;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions, SubscribeUpdate,
    SubscribeUpdateTransactionInfo, subscribe_update,
};
use yellowstone_grpc_proto::solana::storage::confirmed_block;
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;
use yellowstone_grpc_proto::tonic::service::{Interceptor, interceptor::InterceptedService};
use yellowstone_grpc_proto::tonic::transport::{Channel, Endpoint};
use yellowstone_grpc_proto::tonic::{Request, Status};

use crate::cli::context::{
    resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::cli::strategy::{
    StrategyBackend, StrategyMode, build_http_client_pool, build_ip_allocator,
    build_rpc_client_pool,
};
use crate::config::{
    AppConfig, CopySourceKind, CopyStrategyConfig, CopyWalletConfig, IntermediumConfig,
    LanderSettings,
};
use crate::engine::{
    ComputeUnitPriceMode, DispatchStrategy, EngineIdentity, MultiLegInstructions,
    SwapInstructionsVariant, TransactionBuilder, TxVariantPlanner,
};
use crate::jupiter_parser::{PROGRAM_ID as PARSER_PROGRAM_ID, RouteKind, RouteV2Accounts};
use crate::lander::{Deadline, LanderFactory, LanderStack};
use crate::monitoring::events;
use crate::network::IpAllocator;
use crate::txs::jupiter::types::JUPITER_V6_PROGRAM_ID;

const MAX_SEEN_SIGNATURES: usize = 512;
const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");
const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

pub async fn run_copy_strategy(
    config: &AppConfig,
    _backend: &StrategyBackend<'_>,
    mode: StrategyMode,
) -> Result<()> {
    let copy_config = &config.galileo.copy_strategy;
    if !copy_config.enable {
        warn!(target: "strategy", "复制策略未启用，直接退出");
        return Ok(());
    }

    let dry_run = matches!(mode, StrategyMode::DryRun) || config.galileo.bot.dry_run;

    let resolved_rpc = resolve_rpc_client(&config.galileo.global)?;
    let rpc_client = resolved_rpc.client.clone();
    let identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let builder_config = crate::engine::BuilderConfig::new(resolve_instruction_memo(
        &config.galileo.global.instruction,
    ))
    .with_yellowstone(
        config.galileo.global.yellowstone_grpc_url.clone(),
        config.galileo.global.yellowstone_grpc_token.clone(),
        config.galileo.bot.get_block_hash_by_grpc,
    );

    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let rpc_client_pool =
        build_rpc_client_pool(resolved_rpc.endpoints.clone(), global_proxy.clone());

    let mut submission_builder = reqwest::Client::builder();
    if let Some(proxy_url) = global_proxy.as_ref() {
        let proxy = reqwest::Proxy::all(proxy_url.as_str())
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        submission_builder = submission_builder
            .proxy(proxy)
            .danger_accept_invalid_certs(true);
    }
    let submission_client = submission_builder.build()?;
    let submission_client_pool = build_http_client_pool(None, global_proxy.clone(), false, None);

    let tx_builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
    );

    let intermediate_mints = Arc::new(parse_intermediate_mints(&config.galileo.intermedium)?);

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);
    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client.clone(),
        Some(Arc::clone(&submission_client_pool)),
    );

    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);
    let dispatch_strategy = config.lander.lander.sending_strategy;

    let runner = CopyStrategyRunner {
        config: copy_config.clone(),
        rpc_client,
        tx_builder,
        identity,
        ip_allocator,
        compute_unit_price_mode,
        lander_factory,
        lander_settings: config.lander.lander.clone(),
        landing_timeout,
        dispatch_strategy,
        dry_run,
        intermediate_mints,
    };

    runner.run().await
}

fn parse_intermediate_mints(config: &IntermediumConfig) -> Result<HashSet<Pubkey>> {
    let mut set = HashSet::new();
    for mint in &config.mints {
        let trimmed = mint.trim();
        if trimmed.is_empty() {
            continue;
        }
        let pubkey = Pubkey::from_str(trimmed)
            .map_err(|err| anyhow!("intermedium.mints 中的 mint `{trimmed}` 解析失败: {err}"))?;
        set.insert(pubkey);
    }
    Ok(set)
}

struct CopyStrategyRunner {
    config: CopyStrategyConfig,
    rpc_client: Arc<RpcClient>,
    tx_builder: TransactionBuilder,
    identity: EngineIdentity,
    ip_allocator: Arc<IpAllocator>,
    compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    lander_factory: LanderFactory,
    lander_settings: LanderSettings,
    landing_timeout: Duration,
    dispatch_strategy: DispatchStrategy,
    dry_run: bool,
    intermediate_mints: Arc<HashSet<Pubkey>>,
}

impl CopyStrategyRunner {
    async fn run(self) -> Result<()> {
        if self.config.wallets.is_empty() {
            warn!(target: "strategy", "copy_strategy.wallets 为空，直接退出");
            return Ok(());
        }

        let mut tasks = futures::stream::FuturesUnordered::new();

        for wallet in self.config.wallets.clone() {
            let runner = CopyWalletRunner::new(
                wallet,
                self.rpc_client.clone(),
                self.tx_builder.clone(),
                self.identity.clone(),
                self.ip_allocator.clone(),
                self.compute_unit_price_mode.clone(),
                self.lander_factory.clone(),
                self.lander_settings.clone(),
                Arc::clone(&self.intermediate_mints),
                self.landing_timeout,
                self.dispatch_strategy,
                self.dry_run,
            )
            .await
            .with_context(|| "初始化 copy wallet runner 失败")?;
            tasks.push(tokio::spawn(runner.run()));
        }

        while let Some(result) = tasks.next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(err)) => return Err(err),
                Err(err) => {
                    if err.is_cancelled() {
                        return Err(anyhow!("copy wallet task cancelled"));
                    } else {
                        return Err(anyhow!("copy wallet task panicked: {err:?}"));
                    }
                }
            }
        }

        Ok(())
    }
}

struct CopyWalletRunner {
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
}

impl CopyWalletRunner {
    async fn new(
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
        })
    }

    async fn run(self) -> Result<()> {
        match self.wallet.source.kind {
            CopySourceKind::Rpc => self.run_rpc().await,
            CopySourceKind::Grpc => self.run_grpc().await,
        }
    }

    async fn run_rpc(self) -> Result<()> {
        warn!(
            target: "strategy::copy",
            wallet = %self.wallet_pubkey,
            "RPC mode 尚未实现，跳过"
        );
        Ok(())
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

        let include_ids: HashSet<Pubkey> = grpc
            .include_program_ids
            .iter()
            .filter_map(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Some(JUPITER_V6_PROGRAM_ID)
                } else {
                    Pubkey::from_str(trimmed).ok()
                }
            })
            .collect();
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
                    let versioned = decode_versioned_transaction(proto_tx)?;
                    let token_balances = info
                        .meta
                        .as_ref()
                        .map(TransactionTokenBalances::try_from)
                        .transpose()
                        .context("解析 Yellowstone token balance 失败")?;
                    if !filter_transaction(&versioned, &include_ids, &exclude_ids) {
                        continue;
                    }
                    if let Err(err) = self
                        .replay_transaction(
                            &signature,
                            versioned,
                            token_balances.as_ref(),
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

    async fn should_process(&self, signature: &Signature) -> bool {
        let mut guard = self.seen_signatures.lock().await;
        guard.insert(*signature)
    }

    async fn replay_transaction(
        &self,
        signature: &Signature,
        transaction: VersionedTransaction,
        token_balances: Option<&TransactionTokenBalances>,
        fanout_count: u32,
    ) -> Result<()> {
        events::copy_transaction_captured(&self.wallet_pubkey, signature, fanout_count);

        let lookups = resolve_lookup_accounts(&self.rpc_client, &transaction.message).await?;
        let (instructions, account_keys) =
            instructions_from_message(&transaction.message, &lookups).context("指令解析失败")?;

        let Some((route_index, mut route_ctx)) = instructions
            .iter()
            .enumerate()
            .find_map(|(idx, ix)| RouteContext::from_instruction(ix).map(|ctx| (idx, ctx)))
        else {
            debug!(
                target: "strategy::copy",
                wallet = %self.wallet_pubkey,
                signature = %signature,
                "未找到可复制的 Jupiter Route 指令"
            );
            return Ok(());
        };

        route_ctx
            .populate_from_balances(&account_keys, token_balances)
            .context("填充 route token 信息失败")?;
        let replacements =
            build_replacement_map(&route_ctx, &self.identity.pubkey).context("构建账户映射失败")?;
        let mut patched_instructions = instructions.clone();

        if !self
            .intermediate_mints
            .contains(&route_ctx.destination_mint)
        {
            let create_ix = build_create_ata_instruction(
                &self.identity.pubkey,
                &self.identity.pubkey,
                &route_ctx.destination_mint,
                &route_ctx.destination_token_program,
            )?;
            patched_instructions.insert(route_index, create_ix);
        }
        apply_replacements(&mut patched_instructions, &replacements);

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

        let variant = SwapInstructionsVariant::MultiLeg(bundle);
        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &variant, patched_instructions.clone(), 0)
            .await
            .map_err(|err| anyhow!("构建复制交易失败: {err}"))?;

        let layout = self.lander_stack.variant_layout(self.dispatch_strategy);
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
        for attempt in 0..fanout_count.max(1) {
            let plan_id = format!("copy-{}-{}", signature, attempt);
            match self
                .lander_stack
                .submit_plan(&plan, deadline, plan_id.as_str())
                .await
            {
                Ok(receipt) => {
                    events::copy_transaction_dispatched(&self.wallet_pubkey, signature, attempt);
                    info!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        signature = %signature,
                        attempt,
                        lander = receipt.lander,
                        endpoint = %receipt.endpoint,
                        slot = receipt.slot,
                        "复制交易提交成功"
                    );
                    break;
                }
                Err(err) => {
                    warn!(
                        target: "strategy::copy",
                        wallet = %self.wallet_pubkey,
                        signature = %signature,
                        attempt,
                        error = %err,
                        "复制交易提交失败"
                    );
                }
            }
        }

        Ok(())
    }
}

fn resolve_landing_timeout(bot: &crate::config::BotConfig) -> Duration {
    let ms = bot.landing_ms.unwrap_or(2_000).max(1);
    Duration::from_millis(ms)
}

fn derive_compute_unit_price_mode(settings: &LanderSettings) -> Option<ComputeUnitPriceMode> {
    let strategy = settings
        .compute_unit_price_strategy
        .trim()
        .to_ascii_lowercase();
    match strategy.as_str() {
        "" | "none" => None,
        "fixed" => settings
            .fixed_compute_unit_price
            .map(ComputeUnitPriceMode::Fixed),
        "random" => {
            if settings.random_compute_unit_price_range.len() >= 2 {
                let min = settings.random_compute_unit_price_range[0];
                let max = settings.random_compute_unit_price_range[1];
                Some(ComputeUnitPriceMode::Random { min, max })
            } else {
                warn!(
                    target: "strategy::copy",
                    "random compute unit price 需要提供上下限，忽略配置"
                );
                None
            }
        }
        other => {
            warn!(
                target: "strategy::copy",
                strategy = other,
                "未知的 compute_unit_price_strategy"
            );
            None
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

struct YellowstoneTransactionClient {
    client: GeyserClient<InterceptedService<Channel, TokenInterceptor>>,
}

impl YellowstoneTransactionClient {
    async fn connect(endpoint: String, token: Option<AsciiMetadataValue>) -> Result<Self> {
        let endpoint = Endpoint::from_shared(endpoint)
            .map_err(|err| anyhow!("Yellowstone endpoint 无效: {err}"))?
            .tcp_nodelay(true);
        let channel = endpoint
            .connect()
            .await
            .map_err(|err| anyhow!("连接 Yellowstone gRPC 失败: {err}"))?;
        let interceptor = TokenInterceptor { token };
        Ok(Self {
            client: GeyserClient::with_interceptor(channel, interceptor),
        })
    }

    async fn subscribe_transactions(
        &mut self,
        wallet: Pubkey,
    ) -> Result<impl futures::Stream<Item = Result<SubscribeUpdate, Status>>> {
        let mut request = SubscribeRequest::default();
        let mut tx_filter = SubscribeRequestFilterTransactions::default();
        tx_filter.account_required.push(wallet.to_string());
        request.commitment = Some(CommitmentLevel::Processed as i32);
        request.transactions.insert(wallet.to_string(), tx_filter);

        let (sender, receiver) = mpsc::channel(4);
        sender
            .send(request)
            .await
            .map_err(|_| anyhow!("发送订阅请求失败"))?;
        drop(sender);

        let response = self
            .client
            .subscribe(Request::new(ReceiverStream::new(receiver)))
            .await
            .map_err(|err| anyhow!("订阅 Yellowstone 失败: {err}"))?;

        Ok(response.into_inner().map_err(Into::into))
    }
}

#[derive(Clone)]
struct TokenInterceptor {
    token: Option<AsciiMetadataValue>,
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut request: Request<()>) -> std::result::Result<Request<()>, Status> {
        if let Some(token) = &self.token {
            request.metadata_mut().insert("x-token", token.clone());
        }
        Ok(request)
    }
}

fn parse_transaction_update(update: &SubscribeUpdate) -> Option<SubscribeUpdateTransactionInfo> {
    match &update.update_oneof {
        Some(subscribe_update::UpdateOneof::Transaction(tx)) => tx.transaction.clone(),
        _ => None,
    }
}

fn decode_versioned_transaction(tx: &confirmed_block::Transaction) -> Result<VersionedTransaction> {
    let message = tx
        .message
        .as_ref()
        .ok_or_else(|| anyhow!("交易缺少 message"))?;

    let signatures = tx
        .signatures
        .iter()
        .map(|bytes| Signature::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("签名解析失败: {err}"))?;

    let message = if message.versioned {
        VersionedMessage::V0(decode_v0_message(message)?)
    } else {
        let legacy = decode_legacy_message(message)?;
        VersionedMessage::Legacy(legacy)
    };

    Ok(VersionedTransaction {
        signatures,
        message,
    })
}

fn decode_legacy_message(message: &confirmed_block::Message) -> Result<LegacyMessage> {
    use solana_message::MessageHeader;

    let header_proto = message
        .header
        .as_ref()
        .ok_or_else(|| anyhow!("legacy message 缺少 header"))?;
    let header = MessageHeader {
        num_required_signatures: header_proto.num_required_signatures as u8,
        num_readonly_signed_accounts: header_proto.num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: header_proto.num_readonly_unsigned_accounts as u8,
    };

    let account_keys = message
        .account_keys
        .iter()
        .map(|bytes| Pubkey::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("account key 解析失败: {err}"))?;

    let recent_blockhash = {
        let bytes: [u8; 32] = message
            .recent_blockhash
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("recent blockhash 长度错误"))?;
        Hash::new_from_array(bytes)
    };

    let instructions = message
        .instructions
        .iter()
        .map(|ix| CompiledInstruction {
            program_id_index: ix.program_id_index as u8,
            accounts: ix.accounts.clone(),
            data: ix.data.clone(),
        })
        .collect();

    Ok(LegacyMessage {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    })
}

fn decode_v0_message(message: &confirmed_block::Message) -> Result<V0Message> {
    use solana_message::MessageHeader;

    let header_proto = message
        .header
        .as_ref()
        .ok_or_else(|| anyhow!("v0 message 缺少 header"))?;
    let header = MessageHeader {
        num_required_signatures: header_proto.num_required_signatures as u8,
        num_readonly_signed_accounts: header_proto.num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: header_proto.num_readonly_unsigned_accounts as u8,
    };

    let account_keys = message
        .account_keys
        .iter()
        .map(|bytes| Pubkey::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("account key 解析失败: {err}"))?;

    let recent_blockhash = {
        let bytes: [u8; 32] = message
            .recent_blockhash
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("recent blockhash 长度错误"))?;
        Hash::new_from_array(bytes)
    };

    let instructions = message
        .instructions
        .iter()
        .map(|ix| CompiledInstruction {
            program_id_index: ix.program_id_index as u8,
            accounts: ix.accounts.clone(),
            data: ix.data.clone(),
        })
        .collect();

    let mut lookups = Vec::with_capacity(message.address_table_lookups.len());
    for lookup in &message.address_table_lookups {
        let account_key = Pubkey::try_from(lookup.account_key.as_slice())
            .map_err(|err| anyhow!("lookup account 解析失败: {err}"))?;
        lookups.push(MessageAddressTableLookup {
            account_key,
            writable_indexes: lookup.writable_indexes.clone(),
            readonly_indexes: lookup.readonly_indexes.clone(),
        });
    }

    Ok(V0Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
        address_table_lookups: lookups,
    })
}

async fn resolve_lookup_accounts(
    rpc: &Arc<RpcClient>,
    message: &VersionedMessage,
) -> Result<Vec<solana_sdk::message::AddressLookupTableAccount>> {
    let Some(lookups) = message.address_table_lookups() else {
        return Ok(Vec::new());
    };
    if lookups.is_empty() {
        return Ok(Vec::new());
    }

    let keys: Vec<Pubkey> = lookups.iter().map(|lookup| lookup.account_key).collect();
    let accounts = rpc
        .get_multiple_accounts(&keys)
        .await
        .map_err(|err| anyhow!("获取 ALT 失败: {err}"))?;

    let mut resolved = Vec::with_capacity(keys.len());
    for (key, maybe_account) in keys.iter().zip(accounts.into_iter()) {
        if let Some(account) = maybe_account {
            if let Some(table) = deserialize_lookup_table(key, account) {
                resolved.push(table);
            }
        }
    }

    Ok(resolved)
}

fn deserialize_lookup_table(
    key: &Pubkey,
    account: solana_sdk::account::Account,
) -> Option<solana_sdk::message::AddressLookupTableAccount> {
    match AddressLookupTable::deserialize(&account.data) {
        Ok(table) => Some(solana_sdk::message::AddressLookupTableAccount {
            key: *key,
            addresses: table.addresses.into_owned(),
        }),
        Err(err) => {
            warn!(
                target: "strategy::copy",
                address = %key,
                error = %err,
                "反序列化 ALT 失败"
            );
            None
        }
    }
}

fn instructions_from_message(
    message: &VersionedMessage,
    lookups: &[solana_sdk::message::AddressLookupTableAccount],
) -> Result<(Vec<Instruction>, Vec<Pubkey>)> {
    let account_keys = collect_account_keys(message, lookups);

    let mut instructions = Vec::with_capacity(message.instructions().len());
    for compiled in message.instructions() {
        let program_index = compiled.program_id_index as usize;
        let program_id = *account_keys.get(program_index).ok_or_else(|| {
            anyhow!(
                "编译指令 program index {program_index} 超出 account_keys 范围 {}",
                account_keys.len()
            )
        })?;

        let mut metas = Vec::with_capacity(compiled.accounts.len());
        for &index in &compiled.accounts {
            let idx = index as usize;
            let pubkey = *account_keys
                .get(idx)
                .ok_or_else(|| anyhow!("account index {idx} 越界"))?;
            let is_signer = message.is_signer(idx);
            let is_writable = message.is_maybe_writable(idx, None);
            metas.push(AccountMeta {
                pubkey,
                is_signer,
                is_writable,
            });
        }

        instructions.push(Instruction {
            program_id,
            accounts: metas,
            data: compiled.data.clone(),
        });
    }

    Ok((instructions, account_keys))
}

fn collect_account_keys(
    message: &VersionedMessage,
    lookups: &[solana_sdk::message::AddressLookupTableAccount],
) -> Vec<Pubkey> {
    let mut account_keys: Vec<Pubkey> = message.static_account_keys().to_vec();
    if let Some(table_lookups) = message.address_table_lookups() {
        let mut lookup_map: HashMap<Pubkey, &solana_sdk::message::AddressLookupTableAccount> =
            HashMap::with_capacity(lookups.len());
        for account in lookups {
            lookup_map.insert(account.key, account);
        }

        for lookup in table_lookups {
            if let Some(account) = lookup_map.get(&lookup.account_key) {
                for idx in &lookup.writable_indexes {
                    let index = *idx as usize;
                    if let Some(address) = account.addresses.get(index) {
                        account_keys.push(*address);
                    } else {
                        warn!(
                            target: "strategy::copy",
                            lookup = %lookup.account_key,
                            index,
                            "ALT writable index 越界"
                        );
                    }
                }
                for idx in &lookup.readonly_indexes {
                    let index = *idx as usize;
                    if let Some(address) = account.addresses.get(index) {
                        account_keys.push(*address);
                    } else {
                        warn!(
                            target: "strategy::copy",
                            lookup = %lookup.account_key,
                            index,
                            "ALT readonly index 越界"
                        );
                    }
                }
            } else {
                warn!(
                    target: "strategy::copy",
                    lookup = %lookup.account_key,
                    "未找到 ALT 账户，指令可能解析失败"
                );
            }
        }
    }

    account_keys
}

fn filter_transaction(
    transaction: &VersionedTransaction,
    include: &HashSet<Pubkey>,
    exclude: &HashSet<Pubkey>,
) -> bool {
    let message = &transaction.message;
    for compiled in message.instructions() {
        if let Some(program_id) = message
            .static_account_keys()
            .get(compiled.program_id_index as usize)
        {
            if exclude.contains(program_id) {
                return false;
            }
            if include.is_empty() || include.contains(program_id) {
                return true;
            }
        }
    }
    include.is_empty()
}

#[derive(Clone, Debug)]
struct RouteContext {
    authority: Pubkey,
    source_ata: Pubkey,
    destination_ata: Pubkey,
    source_mint: Pubkey,
    destination_mint: Pubkey,
    source_token_program: Pubkey,
    destination_token_program: Pubkey,
}

impl RouteContext {
    fn from_instruction(instruction: &Instruction) -> Option<Self> {
        debug_assert_eq!(
            JUPITER_V6_PROGRAM_ID, PARSER_PROGRAM_ID,
            "Jupiter program id mismatch"
        );
        if instruction.program_id != JUPITER_V6_PROGRAM_ID {
            return None;
        }
        match crate::jupiter_parser::classify(&instruction.data) {
            RouteKind::RouteV2 | RouteKind::SharedRouteV2 | RouteKind::ExactRouteV2 => {
                let parsed = RouteV2Accounts::parse(instruction)?;
                Some(Self {
                    authority: parsed.user_transfer_authority,
                    source_ata: parsed.user_source_token_account,
                    destination_ata: parsed.user_destination_token_account,
                    source_mint: parsed.source_mint,
                    destination_mint: parsed.destination_mint,
                    source_token_program: parsed.source_token_program,
                    destination_token_program: parsed.destination_token_program,
                })
            }
            RouteKind::Route => {
                let accounts = &instruction.accounts;
                if accounts.len() < 9 {
                    return None;
                }
                Some(Self {
                    authority: accounts[1].pubkey,
                    source_ata: accounts[2].pubkey,
                    destination_ata: accounts[3].pubkey,
                    source_mint: Pubkey::default(),
                    destination_mint: accounts[5].pubkey,
                    source_token_program: accounts[0].pubkey,
                    destination_token_program: accounts[0].pubkey,
                })
            }
            RouteKind::Other => None,
        }
    }

    fn populate_from_balances(
        &mut self,
        account_keys: &[Pubkey],
        balances: Option<&TransactionTokenBalances>,
    ) -> Result<()> {
        if self.source_mint != Pubkey::default()
            && self.destination_mint != Pubkey::default()
            && self.source_token_program != Pubkey::default()
            && self.destination_token_program != Pubkey::default()
        {
            return Ok(());
        }

        let Some(balances) = balances else {
            return Err(anyhow!("Yellowstone meta 缺少 token balance 数据"));
        };

        if self.source_mint == Pubkey::default() {
            let source_index = find_account_index(account_keys, &self.source_ata)
                .ok_or_else(|| anyhow!("Route source ATA 未出现在账户列表中"))?;
            let balance = balances
                .get(source_index)
                .ok_or_else(|| anyhow!("未在 Yellowstone token balances 中找到 source ATA"))?;
            self.source_mint = balance.mint;
            if self.source_token_program == Pubkey::default() {
                if let Some(program) = balance.token_program {
                    self.source_token_program = program;
                }
            }
        }

        if let Some(dest_index) = find_account_index(account_keys, &self.destination_ata) {
            if let Some(balance) = balances.get(dest_index) {
                if self.destination_mint == Pubkey::default() {
                    self.destination_mint = balance.mint;
                }
                if self.destination_token_program == Pubkey::default() {
                    if let Some(program) = balance.token_program {
                        self.destination_token_program = program;
                    }
                }
            }
        }

        if self.source_token_program == Pubkey::default()
            || self.destination_token_program == Pubkey::default()
        {
            return Err(anyhow!("缺少 token program 信息"));
        }
        if self.source_mint == Pubkey::default() {
            return Err(anyhow!("缺少 source mint"));
        }
        if self.destination_mint == Pubkey::default() {
            return Err(anyhow!("缺少 destination mint"));
        }
        Ok(())
    }
}

fn find_account_index(account_keys: &[Pubkey], target: &Pubkey) -> Option<usize> {
    account_keys.iter().position(|key| key == target)
}

#[derive(Clone, Debug, Default)]
struct TransactionTokenBalances {
    by_index: HashMap<usize, TokenBalanceEntry>,
}

impl TransactionTokenBalances {
    fn get(&self, index: usize) -> Option<&TokenBalanceEntry> {
        self.by_index.get(&index)
    }
}

impl TryFrom<&confirmed_block::TransactionStatusMeta> for TransactionTokenBalances {
    type Error = anyhow::Error;

    fn try_from(meta: &confirmed_block::TransactionStatusMeta) -> Result<Self> {
        let mut balances = HashMap::new();
        for balance in &meta.pre_token_balances {
            if let Some(entry) = TokenBalanceEntry::parse(balance) {
                balances.entry(entry.account_index).or_insert(entry);
            }
        }
        for balance in &meta.post_token_balances {
            if let Some(entry) = TokenBalanceEntry::parse(balance) {
                balances.insert(entry.account_index, entry);
            }
        }

        Ok(Self { by_index: balances })
    }
}

#[derive(Clone, Debug)]
struct TokenBalanceEntry {
    account_index: usize,
    mint: Pubkey,
    token_program: Option<Pubkey>,
}

impl TokenBalanceEntry {
    fn parse(balance: &confirmed_block::TokenBalance) -> Option<Self> {
        if balance.mint.is_empty() {
            return None;
        }
        let mint = Pubkey::from_str(balance.mint.as_str()).ok()?;
        let token_program = if balance.program_id.is_empty() {
            None
        } else {
            Pubkey::from_str(balance.program_id.as_str()).ok()
        };

        Some(Self {
            account_index: balance.account_index as usize,
            mint,
            token_program,
        })
    }
}

fn build_replacement_map(
    route: &RouteContext,
    new_authority: &Pubkey,
) -> Result<HashMap<Pubkey, Pubkey>> {
    if route.source_mint == Pubkey::default() {
        return Err(anyhow!("缺少 source mint"));
    }
    if route.destination_mint == Pubkey::default() {
        return Err(anyhow!("缺少 destination mint"));
    }
    if route.source_token_program == Pubkey::default()
        || route.destination_token_program == Pubkey::default()
    {
        return Err(anyhow!("缺少 token program"));
    }
    let mut map = HashMap::new();
    map.insert(route.authority, *new_authority);

    let new_source_ata = derive_associated_token_address(
        new_authority,
        &route.source_mint,
        &route.source_token_program,
    )?;
    let new_destination_ata = derive_associated_token_address(
        new_authority,
        &route.destination_mint,
        &route.destination_token_program,
    )?;

    map.insert(route.source_ata, new_source_ata);
    map.insert(route.destination_ata, new_destination_ata);

    Ok(map)
}

fn to_program_pubkey(pk: &Pubkey) -> ProgramPubkey {
    ProgramPubkey::new_from_array(pk.to_bytes())
}

fn from_program_pubkey(pk: ProgramPubkey) -> Pubkey {
    Pubkey::try_from(pk.as_ref()).expect("pubkey length")
}

fn derive_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Pubkey> {
    if mint == &Pubkey::default() {
        return Err(anyhow!("mint 未初始化"));
    }
    if token_program == &Pubkey::default() {
        return Err(anyhow!("token program 未初始化"));
    }
    let owner_prog = to_program_pubkey(owner);
    let mint_prog = to_program_pubkey(mint);
    let token_prog = to_program_pubkey(token_program);
    let program_id = to_program_pubkey(&ASSOCIATED_TOKEN_PROGRAM_ID);
    let (ata, _) = ProgramPubkey::find_program_address(
        &[owner_prog.as_ref(), token_prog.as_ref(), mint_prog.as_ref()],
        &program_id,
    );
    Ok(from_program_pubkey(ata))
}

fn build_create_ata_instruction(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction> {
    let ata = derive_associated_token_address(owner, mint, token_program)?;
    Ok(Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(*token_program, false),
        ],
        data: vec![1],
    })
}

fn apply_replacements(instructions: &mut [Instruction], map: &HashMap<Pubkey, Pubkey>) {
    for instruction in instructions {
        for account in &mut instruction.accounts {
            if let Some(replacement) = map.get(&account.pubkey) {
                account.pubkey = *replacement;
            }
        }
    }
}

fn split_compute_budget(
    instructions: &[Instruction],
    price_mode: Option<&ComputeUnitPriceMode>,
) -> (Vec<Instruction>, Vec<Instruction>) {
    let mut compute_budget = Vec::new();
    let mut main = Vec::new();

    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
            compute_budget.push(ix.clone());
        } else {
            main.push(ix.clone());
        }
    }

    if let Some(price) = price_mode.and_then(sample_compute_unit_price) {
        if price > 0 {
            let mut buf = [0u8; 9];
            buf[0] = 3;
            buf[1..9].copy_from_slice(&price.to_le_bytes());
            compute_budget.push(Instruction {
                program_id: COMPUTE_BUDGET_PROGRAM_ID,
                accounts: Vec::new(),
                data: buf.to_vec(),
            });
        }
    }

    (compute_budget, main)
}

fn sample_compute_unit_price(mode: &ComputeUnitPriceMode) -> Option<u64> {
    match mode {
        ComputeUnitPriceMode::Fixed(price) => Some(*price),
        ComputeUnitPriceMode::Random { min, max } => {
            let low = (*min).min(*max);
            let high = (*min).max(*max);
            if low == high {
                Some(low)
            } else if low == 0 && high == 0 {
                None
            } else {
                let mut rng = rand::rng();
                Some(rng.random_range(low..=high))
            }
        }
    }
}

fn extract_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            if ix.data.len() >= 5 {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&ix.data[1..5]);
                return Some(u32::from_le_bytes(buf));
            }
        }
    }
    None
}

fn lookup_addresses(message: &VersionedMessage) -> Vec<Pubkey> {
    message
        .address_table_lookups()
        .map(|lookups| lookups.iter().map(|lookup| lookup.account_key).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    #[test]
    fn populate_route_context_uses_grpc_balances() {
        let authority = pk(1);
        let source_ata = pk(2);
        let destination_ata = pk(3);
        let destination_mint = pk(4);
        let source_mint = pk(5);
        let token_program = pk(6);

        let mut route = RouteContext {
            authority,
            source_ata,
            destination_ata,
            source_mint: Pubkey::default(),
            destination_mint,
            source_token_program: token_program,
            destination_token_program: token_program,
        };

        let account_keys = vec![
            token_program,
            authority,
            source_ata,
            destination_ata,
            destination_mint,
        ];

        let mut meta = confirmed_block::TransactionStatusMeta::default();
        meta.pre_token_balances.push(confirmed_block::TokenBalance {
            account_index: 2,
            mint: source_mint.to_string(),
            ui_token_amount: None,
            owner: authority.to_string(),
            program_id: token_program.to_string(),
        });

        let balances = TransactionTokenBalances::try_from(&meta).expect("token balances");
        route
            .populate_from_balances(&account_keys, Some(&balances))
            .expect("populate route context");

        assert_eq!(route.source_mint, source_mint);
        assert_eq!(route.destination_mint, destination_mint);
        assert_eq!(route.source_token_program, token_program);
        assert_eq!(route.destination_token_program, token_program);
    }

    #[test]
    fn populate_route_context_errors_without_balances() {
        let authority = pk(9);
        let source_ata = pk(10);
        let destination_ata = pk(11);
        let token_program = pk(12);
        let mut route = RouteContext {
            authority,
            source_ata,
            destination_ata,
            source_mint: Pubkey::default(),
            destination_mint: pk(13),
            source_token_program: token_program,
            destination_token_program: token_program,
        };
        let account_keys = vec![token_program, authority, source_ata, destination_ata];
        let result = route.populate_from_balances(&account_keys, None);
        assert!(result.is_err());
    }
}
