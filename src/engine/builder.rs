use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{debug, info, warn};
use yellowstone_grpc_proto::geyser::CommitmentLevel;

use crate::cache::{Cache, InMemoryBackend};
use crate::network::{
    IpAllocator, IpBoundClientPool, IpLeaseMode, IpLeaseOutcome, IpTaskKind, RpcClientFactoryFn,
};
use crate::rpc::BlockhashSnapshot;
use crate::rpc::yellowstone::YellowstoneBlockhashClient;

use super::aggregator::SwapInstructionsVariant;
use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;

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

#[derive(Clone)]
pub struct PreparedTransaction {
    pub transaction: VersionedTransaction,
    pub blockhash: Hash,
    pub slot: u64,
    pub last_valid_block_height: Option<u64>,
    pub signer: Arc<Keypair>,
    pub tip_lamports: u64,
    pub instructions: Vec<Instruction>,
    pub lookup_accounts: Vec<AddressLookupTableAccount>,
}

#[derive(Clone)]
pub struct TransactionBuilder {
    rpc: Arc<RpcClient>,
    config: BuilderConfig,
    yellowstone: Option<YellowstoneBlockhashClient>,
    lookup_cache: Cache<InMemoryBackend<Pubkey, AddressLookupTableAccount>>,
    ip_allocator: Arc<IpAllocator>,
    rpc_pool: Option<Arc<IpBoundClientPool<RpcClientFactoryFn>>>,
}

impl TransactionBuilder {
    pub fn new(
        rpc: Arc<RpcClient>,
        config: BuilderConfig,
        ip_allocator: Arc<IpAllocator>,
        rpc_pool: Option<Arc<IpBoundClientPool<RpcClientFactoryFn>>>,
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
            lookup_cache: Cache::new(InMemoryBackend::default()),
            ip_allocator,
            rpc_pool,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn build(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_with_options(identity, instructions, None, tip_lamports)
            .await
    }

    pub async fn build_with_sequence(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        sequence: Vec<Instruction>,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_with_options(identity, instructions, Some(sequence), tip_lamports)
            .await
    }

    async fn build_with_options(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        override_sequence: Option<Vec<Instruction>>,
        tip_lamports: u64,
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
        rpc: &Arc<RpcClient>,
    ) -> EngineResult<PreparedTransaction> {
        let lookup_accounts = if let Some(resolved) = Self::resolved_tables(instructions) {
            resolved
        } else {
            self.load_lookup_tables(rpc, instructions.address_lookup_table_addresses())
                .await?
        };

        let snapshot = if let Some(meta) = instructions.blockhash_with_metadata() {
            BlockhashSnapshot {
                blockhash: meta.blockhash,
                slot: None,
                last_valid_block_height: Some(meta.last_valid_block_height),
            }
        } else {
            self.latest_blockhash(rpc).await?
        };
        let blockhash = snapshot.blockhash;
        let last_valid_block_height = snapshot.last_valid_block_height;
        let slot = snapshot.slot.or(last_valid_block_height).unwrap_or(0);

        let mut instructions = match override_sequence {
            Some(sequence) => sequence,
            None => instructions.flatten_instructions(),
        };

        if let Some(memo) = &self.config.memo {
            instructions.push(build_memo_instruction(memo));
        }

        if tip_lamports > 0 {
            debug!(
                target: "engine::builder",
                tip_lamports,
                "tip 尚未实现专用指令，未来接入 lander 扩展"
            );
        }

        let message =
            compile_message(&identity.pubkey, &instructions, &lookup_accounts, blockhash)?;
        let versioned = VersionedMessage::V0(message);
        let signer = identity.signer.clone();
        let tx = VersionedTransaction::try_new(versioned, &[signer.as_ref()])
            .map_err(|err| EngineError::Transaction(anyhow!(err)))?;

        Ok(PreparedTransaction {
            transaction: tx,
            blockhash,
            slot,
            last_valid_block_height,
            signer,
            tip_lamports,
            instructions,
            lookup_accounts,
        })
    }

    async fn load_lookup_tables(
        &self,
        rpc: &Arc<RpcClient>,
        addresses: &[solana_sdk::pubkey::Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(addresses.len());
        let mut missing = Vec::new();

        for address in addresses {
            if let Some(entry) = self.lookup_cache.get(address).await {
                result.push((*entry).clone());
            } else {
                missing.push(*address);
            }
        }

        if missing.is_empty() {
            return Ok(result);
        }

        let fetched = self.fetch_lookup_tables(rpc, &missing).await?;
        for table in &fetched {
            self.lookup_cache
                .insert_arc(table.key, Arc::new(table.clone()), None)
                .await;
        }
        result.extend(fetched);
        Ok(result)
    }

    async fn fetch_lookup_tables(
        &self,
        rpc: &Arc<RpcClient>,
        addresses: &[Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        const ALT_BATCH_LIMIT: usize = 100;
        let mut collected = Vec::new();

        for chunk in addresses.chunks(ALT_BATCH_LIMIT) {
            match rpc.get_multiple_accounts(chunk).await {
                Ok(accounts) => {
                    for (address, maybe_account) in chunk.iter().zip(accounts.into_iter()) {
                        match maybe_account {
                            Some(account) => {
                                if let Some(table) = deserialize_lookup_table(address, account) {
                                    collected.push(table);
                                }
                            }
                            None => warn!(
                                target: "engine::builder",
                                address = %address,
                                "批量拉取 ALT 返回空账户"
                            ),
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        target: "engine::builder",
                        error = %err,
                        "批量拉取 ALT 失败，回退逐条查询"
                    );
                    for address in chunk {
                        match rpc.get_account(address).await {
                            Ok(account) => {
                                if let Some(table) = deserialize_lookup_table(address, account) {
                                    collected.push(table);
                                }
                            }
                            Err(err) => warn!(
                                target: "engine::builder",
                                address = %address,
                                error = %err,
                                "拉取 ALT 账户失败"
                            ),
                        }
                    }
                }
            }
        }

        Ok(collected)
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
    payer: &solana_sdk::pubkey::Pubkey,
    instructions: &[Instruction],
    tables: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> EngineResult<V0Message> {
    V0Message::try_compile(payer, instructions, tables, blockhash)
        .map_err(|err| EngineError::Transaction(anyhow!(err)))
}

fn deserialize_lookup_table(
    address: &Pubkey,
    account: Account,
) -> Option<AddressLookupTableAccount> {
    match AddressLookupTable::deserialize(&account.data) {
        Ok(table) => Some(AddressLookupTableAccount {
            key: *address,
            addresses: table.addresses.into_owned(),
        }),
        Err(err) => {
            warn!(
                target: "engine::builder",
                address = %address,
                error = %err,
                "反序列化 ALT 失败"
            );
            None
        }
    }
}

fn build_memo_instruction(text: &str) -> Instruction {
    use solana_sdk::pubkey::Pubkey;
    let program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")
        .unwrap_or_else(|_| Pubkey::new_unique());
    Instruction {
        program_id,
        accounts: vec![],
        data: text.as_bytes().to_vec(),
    }
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
