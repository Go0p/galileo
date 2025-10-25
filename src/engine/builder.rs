use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use yellowstone_grpc_proto::geyser::CommitmentLevel;

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
    lookup_cache: LookupTableCache,
}

impl TransactionBuilder {
    pub fn new(rpc: Arc<RpcClient>, config: BuilderConfig) -> Self {
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
            lookup_cache: LookupTableCache::default(),
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn build(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_internal(identity, instructions, None, tip_lamports)
            .await
    }

    pub async fn build_with_sequence(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        sequence: Vec<Instruction>,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_internal(identity, instructions, Some(sequence), tip_lamports)
            .await
    }

    async fn build_internal(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsVariant,
        override_sequence: Option<Vec<Instruction>>,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        let lookup_accounts = if let Some(resolved) = Self::resolved_tables(instructions) {
            resolved
        } else {
            self.load_lookup_tables(instructions.address_lookup_table_addresses())
                .await?
        };

        let snapshot = if let Some(meta) = instructions.blockhash_with_metadata() {
            BlockhashSnapshot {
                blockhash: meta.blockhash,
                slot: None,
                last_valid_block_height: Some(meta.last_valid_block_height),
            }
        } else {
            self.latest_blockhash().await?
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
        addresses: &[solana_sdk::pubkey::Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        self.lookup_cache.resolve(&self.rpc, addresses).await
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

    async fn latest_blockhash(&self) -> EngineResult<BlockhashSnapshot> {
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
        self.rpc_latest_blockhash().await
    }

    async fn rpc_latest_blockhash(&self) -> EngineResult<BlockhashSnapshot> {
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .map_err(EngineError::Rpc)?;
        Ok(BlockhashSnapshot {
            blockhash,
            slot: None,
            last_valid_block_height: None,
        })
    }
}

#[derive(Clone, Default)]
struct LookupTableCache {
    inner: Arc<RwLock<HashMap<Pubkey, AddressLookupTableAccount>>>,
}

impl LookupTableCache {
    async fn resolve(
        &self,
        rpc: &Arc<RpcClient>,
        addresses: &[Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

        let mut missing = Vec::new();
        {
            let guard = self.inner.read().await;
            for address in addresses {
                if !guard.contains_key(address) {
                    missing.push(*address);
                }
            }
        }

        if !missing.is_empty() {
            let fetched = Self::fetch_many(rpc, &missing).await?;
            let mut guard = self.inner.write().await;
            for account in fetched {
                guard.insert(account.key, account);
            }
        }

        let guard = self.inner.read().await;
        let mut resolved = Vec::with_capacity(addresses.len());
        for address in addresses {
            match guard.get(address) {
                Some(account) => resolved.push(account.clone()),
                None => warn!(
                    target: "engine::builder",
                    address = %address,
                    "ALT 缓存缺失，略过该表"
                ),
            }
        }

        Ok(resolved)
    }

    async fn fetch_many(
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
                                if let Some(table) = Self::deserialize(address, account) {
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
                                if let Some(table) = Self::deserialize(address, account) {
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

    fn deserialize(address: &Pubkey, account: Account) -> Option<AddressLookupTableAccount> {
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
