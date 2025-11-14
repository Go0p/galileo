use std::net::IpAddr;
use std::sync::Arc;

use anyhow::anyhow;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{debug, info, warn};
use yellowstone_grpc_proto::geyser::CommitmentLevel;

use crate::cache::AltCache;
use crate::network::{
    IpAllocator, IpBoundClientPool, IpLeaseMode, IpLeaseOutcome, IpTaskKind, RpcClientFactoryFn,
};
use crate::rpc::BlockhashSnapshot;
use crate::rpc::yellowstone::YellowstoneBlockhashClient;

use super::COMPUTE_BUDGET_PROGRAM_ID;
use super::aggregator::SwapInstructionsVariant;
use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;
use super::types::JitoTipPlan;
use crate::engine::assembly::decorators::GuardStrategy;

#[derive(Clone)]
pub struct BuilderConfig {
    pub memo: Option<String>,
    pub yellowstone: Option<YellowstoneGrpcSettings>,
}

impl BuilderConfig {
    pub fn new(memo: Option<String>) -> Self {
        Self {
            memo,
            yellowstone: None,
        }
    }

    pub fn with_yellowstone(
        mut self,
        endpoint: Option<String>,
        token: Option<String>,
        enable_grpc: bool,
    ) -> Self {
        if !enable_grpc {
            self.yellowstone = None;
            return self;
        }
        self.yellowstone = endpoint
            .and_then(|url| {
                let trimmed = url.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .map(|endpoint| YellowstoneGrpcSettings {
                endpoint,
                x_token: token.and_then(|value| {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }),
            });
        self
    }
}

#[derive(Clone)]
pub struct YellowstoneGrpcSettings {
    pub endpoint: String,
    pub x_token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PreparedTransaction {
    pub transaction: VersionedTransaction,
    pub blockhash: Hash,
    pub slot: u64,
    pub signer: Arc<Keypair>,
    pub tip_lamports: u64,
    pub prioritization_fee_lamports: u64,
    pub guard_lamports: u64,
    pub guard_strategy: GuardStrategy,
    pub compute_unit_price_micro_lamports: Option<u64>,
    pub tip_strategy_label: &'static str,
    pub compute_unit_price_strategy_label: &'static str,
    pub instructions: Vec<Instruction>,
    pub lookup_accounts: Vec<AddressLookupTableAccount>,
    pub jito_tip_plan: Option<JitoTipPlan>,
}

#[derive(Clone)]
pub struct TransactionBuilder {
    rpc: Arc<RpcClient>,
    config: BuilderConfig,
    yellowstone: Option<YellowstoneBlockhashClient>,
    alt_cache: AltCache,
    ip_allocator: Arc<IpAllocator>,
    rpc_pool: Option<Arc<IpBoundClientPool<RpcClientFactoryFn>>>,
    force_rpc_blockhash: bool,
}

impl TransactionBuilder {
    pub fn new(
        rpc: Arc<RpcClient>,
        config: BuilderConfig,
        ip_allocator: Arc<IpAllocator>,
        rpc_pool: Option<Arc<IpBoundClientPool<RpcClientFactoryFn>>>,
        alt_cache: AltCache,
        force_rpc_blockhash: bool,
    ) -> Self {
        let yellowstone =
            config
                .yellowstone
                .as_ref()
                .and_then(|settings| {
                    match YellowstoneBlockhashClient::new(
                        settings.endpoint.clone(),
                        settings.x_token.clone(),
                        CommitmentLevel::Confirmed,
                    ) {
                        Ok(client) => {
                            info!(
                                target: "engine::builder",
                                endpoint = settings.endpoint,
                                "Yellowstone gRPC blockhash 已启用"
                            );
                            Some(client)
                        }
                        Err(err) => {
                            warn!(
                                target: "engine::builder",
                                endpoint = settings.endpoint,
                                error = %err,
                                "初始化 Yellowstone gRPC 失败，将回退至 RPC blockhash"
                            );
                            None
                        }
                    }
                });
        Self {
            rpc,
            config,
            yellowstone,
            alt_cache,
            ip_allocator,
            rpc_pool,
            force_rpc_blockhash,
        }
    }

    pub async fn build_with_sequence(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        sequence: Vec<Instruction>,
        tip_lamports: u64,
        jito_tip_plan: Option<JitoTipPlan>,
    ) -> EngineResult<PreparedTransaction> {
        self.build_with_options(
            identity,
            instructions,
            Some(sequence),
            tip_lamports,
            jito_tip_plan,
        )
        .await
    }

    async fn build_with_options(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        override_sequence: Option<Vec<Instruction>>,
        tip_lamports: u64,
        jito_tip_plan: Option<JitoTipPlan>,
    ) -> EngineResult<PreparedTransaction> {
        let lease = self
            .ip_allocator
            .acquire(IpTaskKind::SwapInstruction, IpLeaseMode::Ephemeral)
            .await
            .map_err(EngineError::NetworkResource)?;
        let handle = lease.handle();
        let local_ip = Some(handle.ip());
        drop(lease);

        let rpc = self.rpc_for_ip(local_ip);
        let result = self
            .build_internal(
                identity,
                instructions,
                override_sequence,
                tip_lamports,
                jito_tip_plan.clone(),
                &rpc,
            )
            .await;

        if let Err(err) = &result {
            if let Some(outcome) = classify_builder_error(err) {
                handle.mark_outcome(outcome);
            }
        }

        drop(handle);

        result
    }

    async fn build_internal(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        override_sequence: Option<Vec<Instruction>>,
        tip_lamports: u64,
        jito_tip_plan: Option<JitoTipPlan>,
        rpc: &Arc<RpcClient>,
    ) -> EngineResult<PreparedTransaction> {
        let lookup_accounts = if let Some(resolved) = Self::resolved_tables(instructions) {
            resolved
        } else {
            self.load_lookup_tables(rpc, instructions.address_lookup_table_addresses())
                .await?
        };

        let snapshot = if !self.force_rpc_blockhash {
            if let Some(meta) = instructions.blockhash_with_metadata() {
                BlockhashSnapshot {
                    blockhash: meta.blockhash,
                    slot: None,
                    last_valid_block_height: Some(meta.last_valid_block_height),
                }
            } else {
                self.latest_blockhash(rpc).await?
            }
        } else {
            self.latest_blockhash(rpc).await?
        };
        let blockhash = snapshot.blockhash;
        let slot = snapshot
            .slot
            .or(snapshot.last_valid_block_height)
            .unwrap_or(0);

        let mut instructions = match override_sequence {
            Some(sequence) => sequence,
            None => instructions.flatten_instructions(),
        };

        if self.force_rpc_blockhash {
            Self::strip_compute_unit_price(&mut instructions);
        }

        if let Some(memo) = &self.config.memo {
            instructions.push(Self::build_memo_instruction(memo));
        }

        if tip_lamports > 0 && !self.force_rpc_blockhash {
            debug!(
                target: "engine::builder",
                tip_lamports,
                "tip 尚未实现专用指令，未来接入 lander 扩展"
            );
        }

        let message =
            compile_message(&identity.pubkey, &instructions, &lookup_accounts, blockhash)?;
        let signer_slice: Vec<_> = message
            .account_keys
            .iter()
            .take(message.header.num_required_signatures as usize)
            .collect();
        debug!(
            target: "engine::builder",
            payer = %identity.pubkey,
            required_signers = message.header.num_required_signatures,
            total_static_accounts = message.account_keys.len(),
            signers = ?signer_slice,
            "构建 VersionedMessage 成功"
        );
        let versioned = VersionedMessage::V0(message);
        let signer = identity.signer.clone();
        let tx = VersionedTransaction::try_new(versioned, &[signer.as_ref()])
            .map_err(|err| EngineError::Transaction(anyhow!(err)))?;

        Ok(PreparedTransaction {
            transaction: tx,
            blockhash,
            slot,
            signer,
            tip_lamports,
            prioritization_fee_lamports: 0,
            guard_lamports: 0,
            guard_strategy: GuardStrategy::BasePlusTipAndPrioritizationFee,
            compute_unit_price_micro_lamports: None,
            tip_strategy_label: "none",
            compute_unit_price_strategy_label: "unknown",
            instructions,
            lookup_accounts,
            jito_tip_plan,
        })
    }

    async fn load_lookup_tables(
        &self,
        rpc: &Arc<RpcClient>,
        addresses: &[Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

        self.alt_cache
            .fetch_many(rpc, addresses)
            .await
            .map_err(|err| {
                warn!(
                    target: "engine::builder",
                    error = %err,
                    count = addresses.len(),
                    "加载 ALT 失败"
                );
                EngineError::Transaction(err.into())
            })
    }
    fn build_memo_instruction(memo: &str) -> Instruction {
        Instruction {
            program_id: solana_sdk::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
            accounts: Vec::new(),
            data: memo.as_bytes().to_vec(),
        }
    }

    fn strip_compute_unit_price(instructions: &mut Vec<Instruction>) {
        instructions.retain(|ix| {
            !(ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&3))
        });
    }

    fn rpc_for_ip(&self, ip: Option<IpAddr>) -> Arc<RpcClient> {
        if let Some(ip) = ip {
            if let Some(pool) = &self.rpc_pool {
                match pool.get_or_create(ip) {
                    Ok(client) => return client,
                    Err(err) => {
                        warn!(
                            target: "engine::builder",
                            ip = %ip,
                            error = %err,
                            "构建绑定 IP 的 RPC 客户端失败，回退默认客户端"
                        );
                    }
                }
            }
        }
        Arc::clone(&self.rpc)
    }

    fn resolved_tables(
        instructions: &SwapInstructionsVariant,
    ) -> Option<Vec<AddressLookupTableAccount>> {
        let resolved = instructions.resolved_lookup_tables();
        if resolved.is_empty() {
            None
        } else {
            Some(resolved.to_vec())
        }
    }

    async fn latest_blockhash(&self, rpc: &Arc<RpcClient>) -> EngineResult<BlockhashSnapshot> {
        if let Some(client) = &self.yellowstone {
            match client.latest_blockhash().await {
                Ok(snapshot) => return Ok(snapshot),
                Err(err) => {
                    warn!(
                        target: "engine::builder",
                        error = %err,
                        "Yellowstone gRPC 获取 blockhash 失败，回退至 RPC"
                    );
                }
            }
        }
        self.rpc_latest_blockhash(rpc).await
    }

    async fn rpc_latest_blockhash(&self, rpc: &Arc<RpcClient>) -> EngineResult<BlockhashSnapshot> {
        let blockhash = rpc.get_latest_blockhash().await.map_err(EngineError::Rpc)?;
        Ok(BlockhashSnapshot {
            blockhash,
            slot: None,
            last_valid_block_height: None,
        })
    }
}

fn compile_message(
    payer: &Pubkey,
    instructions: &[Instruction],
    tables: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> EngineResult<V0Message> {
    V0Message::try_compile(payer, instructions, tables, blockhash)
        .map_err(|err| EngineError::Transaction(anyhow!(err)))
}

fn classify_builder_error(err: &EngineError) -> Option<IpLeaseOutcome> {
    match err {
        EngineError::Rpc(inner) => classify_client_error(inner),
        EngineError::Network(inner) => classify_reqwest_error(inner),
        EngineError::NetworkResource(_) => Some(IpLeaseOutcome::NetworkError),
        _ => None,
    }
}

fn classify_client_error(err: &ClientError) -> Option<IpLeaseOutcome> {
    match err.kind() {
        ClientErrorKind::Reqwest(inner) => classify_reqwest_error(inner),
        ClientErrorKind::Io(_) => Some(IpLeaseOutcome::NetworkError),
        _ => None,
    }
}

fn classify_reqwest_error(err: &reqwest::Error) -> Option<IpLeaseOutcome> {
    if err.is_timeout() {
        return Some(IpLeaseOutcome::Timeout);
    }
    if let Some(status) = err.status() {
        if status.as_u16() == 429 {
            return Some(IpLeaseOutcome::RateLimited);
        }
        if status.as_u16() == 408 || status.as_u16() == 504 {
            return Some(IpLeaseOutcome::Timeout);
        }
        if status.is_server_error() {
            return Some(IpLeaseOutcome::NetworkError);
        }
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}
