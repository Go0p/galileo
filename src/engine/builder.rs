use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{debug, warn};

use crate::api::jupiter::SwapInstructionsResponse;

use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;

#[derive(Clone)]
pub struct BuilderConfig {
    pub memo: Option<String>,
}

impl BuilderConfig {
    pub fn new(memo: Option<String>) -> Self {
        Self { memo }
    }
}

#[derive(Clone)]
pub struct PreparedTransaction {
    pub transaction: VersionedTransaction,
    pub blockhash: Hash,
    pub slot: u64,
    pub signer: Arc<Keypair>,
    pub tip_lamports: u64,
}

#[derive(Clone)]
pub struct TransactionBuilder {
    rpc: Arc<RpcClient>,
    config: BuilderConfig,
}

impl TransactionBuilder {
    pub fn new(rpc: Arc<RpcClient>, config: BuilderConfig) -> Self {
        Self { rpc, config }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn build(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsResponse,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_internal(identity, instructions, None, tip_lamports)
            .await
    }

    pub async fn build_with_sequence(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsResponse,
        sequence: Vec<Instruction>,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        self.build_internal(identity, instructions, Some(sequence), tip_lamports)
            .await
    }

    async fn build_internal(
        &self,
        identity: &EngineIdentity,
        instructions: &SwapInstructionsResponse,
        override_sequence: Option<Vec<Instruction>>,
        tip_lamports: u64,
    ) -> EngineResult<PreparedTransaction> {
        let lookup_accounts = if instructions.resolved_lookup_tables.is_empty() {
            self.load_lookup_tables(&instructions.address_lookup_table_addresses)
                .await?
        } else {
            instructions.resolved_lookup_tables.clone()
        };

        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .map_err(EngineError::Rpc)?;

        let slot = self.rpc.get_slot().await.map_err(EngineError::Rpc)?;

        let mut ix = match override_sequence {
            Some(sequence) => sequence,
            None => instructions.flatten_instructions(),
        };

        if let Some(memo) = &self.config.memo {
            ix.push(build_memo_instruction(memo));
        }

        if tip_lamports > 0 {
            debug!(
                target: "engine::builder",
                tip_lamports,
                "tip 尚未实现专用指令，未来接入 lander 扩展"
            );
        }

        let message = compile_message(&identity.pubkey, &ix, &lookup_accounts, blockhash)?;
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
        })
    }

    async fn load_lookup_tables(
        &self,
        addresses: &[solana_sdk::pubkey::Pubkey],
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        let mut tables = Vec::new();
        for address in addresses {
            match self.rpc.get_account(address).await {
                Ok(account) => match AddressLookupTable::deserialize(&account.data) {
                    Ok(table) => {
                        tables.push(AddressLookupTableAccount {
                            key: *address,
                            addresses: table.addresses.into_owned(),
                        });
                    }
                    Err(err) => {
                        warn!(
                            target: "engine::builder",
                            address = %address,
                            error = %err,
                            "反序列化 ALT 失败"
                        );
                    }
                },
                Err(err) => {
                    warn!(
                        target: "engine::builder",
                        address = %address,
                        error = %err,
                        "拉取 ALT 账户失败"
                    );
                }
            }
        }
        Ok(tables)
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
