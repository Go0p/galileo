use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;

use metrics::gauge;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use solana_system_interface::instruction as system_instruction;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::warn;

use crate::config::{LanderJitoMultiIpsSetting, LanderJitoTipsWalletConfig};
use crate::engine::TxVariant;
use crate::lander::error::LanderError;

use super::bundle::{encode_transaction, random_tip_wallet, strip_tip_transfer};
use super::tip::MIN_JITO_TIP_LAMPORTS;
use crate::instructions::guards::lighthouse::program::LIGHTHOUSE_PROGRAM_ID;

const SURCHARGE_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
const GUARD_BUFFER_LAMPORTS: u64 = 5_000;

#[derive(Clone)]
pub(crate) struct MultiIpsStrategy {
    wallet_pool: Arc<WalletPool>,
}

impl MultiIpsStrategy {
    pub fn new(config: &LanderJitoMultiIpsSetting) -> Option<Self> {
        let pool = WalletPool::new(&config.tips_wallet);
        Some(Self {
            wallet_pool: Arc::new(pool),
        })
    }

    pub async fn build_bundle(
        &self,
        variant: &TxVariant,
        base_tip_lamports: u64,
        tip_offset: i64,
    ) -> Result<Option<MultiIpsBundle>, LanderError> {
        let tip_wallet = match random_tip_wallet() {
            Some(wallet) => wallet,
            None => {
                warn!(
                    target: "lander::jito::multi_ips",
                    "tip wallet 列表为空，跳过 multi_ips bundle 构建"
                );
                return Ok(None);
            }
        };

        let adjusted_tip = adjust_tip(base_tip_lamports, tip_offset);
        if adjusted_tip < MIN_JITO_TIP_LAMPORTS {
            warn!(
                target: "lander::jito::multi_ips",
                tip = adjusted_tip,
                "multi_ips tip 太小，跳过"
            );
            return Ok(None);
        }

        let wallet = self.wallet_pool.acquire().await;
        let guard_buffer = GUARD_BUFFER_LAMPORTS;
        let deposit = adjusted_tip
            .saturating_add(guard_buffer)
            .saturating_add(SURCHARGE_LAMPORTS);
        let reclaim = SURCHARGE_LAMPORTS;

        let payer = variant.signer().pubkey();
        let (mut base_instructions, insertion_idx) =
            strip_tip_transfer(variant.instructions(), variant.jito_tip_plan(), &payer);
        let insert_at = insertion_idx.unwrap_or(base_instructions.len());
        base_instructions.insert(
            insert_at,
            system_instruction::transfer(&payer, &wallet.pubkey(), deposit),
        );
        bump_profit_guard_threshold(&mut base_instructions, guard_buffer);
        let main_tx = assemble_main_transaction(variant, base_instructions)?;

        let reclaim_instruction = if reclaim > 0 {
            Some(system_instruction::transfer(
                &wallet.pubkey(),
                &payer,
                reclaim,
            ))
        } else {
            None
        };

        let mut instructions = vec![system_instruction::transfer(
            &wallet.pubkey(),
            &tip_wallet,
            adjusted_tip,
        )];
        if let Some(ix) = reclaim_instruction {
            instructions.push(ix);
        }

        let tip_tx = build_tip_transaction(&wallet, &instructions, &variant.blockhash())?;

        let encoded_main = encode_transaction(&main_tx)?;
        let encoded_tip = encode_transaction(&tip_tx)?;

        Ok(Some(MultiIpsBundle {
            encoded_transactions: vec![encoded_main, encoded_tip],
            ephemeral_wallet: wallet.pubkey(),
            jito_tip_wallet: tip_wallet,
            tip_lamports: adjusted_tip,
            main_transaction: main_tx,
            tip_transaction: tip_tx,
        }))
    }
}

#[derive(Clone)]
pub(crate) struct MultiIpsBundle {
    pub encoded_transactions: Vec<String>,
    pub ephemeral_wallet: Pubkey,
    pub jito_tip_wallet: Pubkey,
    pub tip_lamports: u64,
    pub main_transaction: VersionedTransaction,
    pub tip_transaction: VersionedTransaction,
}

fn assemble_main_transaction(
    variant: &TxVariant,
    instructions: Vec<Instruction>,
) -> Result<VersionedTransaction, LanderError> {
    let payer = variant.signer().pubkey();
    let message = V0Message::try_compile(
        &payer,
        &instructions,
        variant.lookup_accounts(),
        variant.blockhash(),
    )
    .map_err(|err| LanderError::fatal(format!("构建 multi_ips 主交易消息失败: {err:#}")))?;
    let versioned = VersionedMessage::V0(message);
    VersionedTransaction::try_new(versioned, &[variant.signer().as_ref()])
        .map_err(|err| LanderError::fatal(format!("签名 multi_ips 主交易失败: {err:#}")))
}

fn build_tip_transaction(
    wallet: &Arc<Keypair>,
    instructions: &[Instruction],
    blockhash: &solana_sdk::hash::Hash,
) -> Result<VersionedTransaction, LanderError> {
    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&wallet.pubkey()),
        &[wallet.as_ref()],
        *blockhash,
    );
    Ok(VersionedTransaction::from(transaction))
}

fn adjust_tip(base_tip: u64, offset: i64) -> u64 {
    if offset == 0 {
        return base_tip.max(MIN_JITO_TIP_LAMPORTS);
    }

    let base = base_tip as i64;
    let adjusted = base.saturating_add(offset);
    if adjusted <= 0 {
        MIN_JITO_TIP_LAMPORTS
    } else {
        adjusted as u64
    }
}

fn bump_profit_guard_threshold(instructions: &mut [Instruction], extra: u64) {
    if extra == 0 {
        return;
    }
    for instruction in instructions.iter_mut() {
        if instruction.program_id != LIGHTHOUSE_PROGRAM_ID {
            continue;
        }
        if adjust_guard_delta(&mut instruction.data, extra) {
            break;
        }
    }
}

fn adjust_guard_delta(data: &mut Vec<u8>, extra: u64) -> bool {
    if data.first() != Some(&4) {
        return false;
    }
    // layout: [4, log_level, 1, snapshot_offset..., account_data_offset..., 6, expected_delta (i128 LE), operator]
    let expected_start = 6;
    if data.len() < expected_start + 16 {
        return false;
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&data[expected_start..expected_start + 16]);
    let mut value = i128::from_le_bytes(buf);
    value = value.saturating_add(extra as i128);
    data[expected_start..expected_start + 16].copy_from_slice(&value.to_le_bytes());
    true
}

#[derive(Clone)]
struct WalletPool {
    store: Arc<Mutex<VecDeque<Arc<Keypair>>>>,
}

impl WalletPool {
    fn new(config: &LanderJitoTipsWalletConfig) -> Self {
        let init = config.init_wallet_size.min(20_000) as usize;
        let mut initial = VecDeque::with_capacity(init);
        for _ in 0..init {
            initial.push_back(Arc::new(Keypair::new()));
        }
        let initial_len = initial.len();
        let store = Arc::new(Mutex::new(initial));
        gauge!("galileo_jito_multi_ips_wallet_pool_size").set(initial_len as f64);

        let interval_ms = config.auto_generate_interval_ms;
        let generate_count = usize::try_from(config.auto_generate_count).unwrap_or(usize::MAX);
        let refill_threshold = if config.refill_threshold == 0 {
            None
        } else {
            Some(usize::try_from(config.refill_threshold).unwrap_or(usize::MAX))
        };
        if interval_ms > 0 && generate_count > 0 {
            if let Ok(handle) = Handle::try_current() {
                let store_clone = store.clone();
                let threshold = refill_threshold;
                handle.spawn(async move {
                    let interval = Duration::from_millis(interval_ms);
                    loop {
                        sleep(interval).await;
                        let mut guard = store_clone.lock().await;
                        if let Some(limit) = threshold {
                            if guard.len() >= limit {
                                continue;
                            }
                            let missing = limit - guard.len();
                            let batch = missing.min(generate_count);
                            for _ in 0..batch {
                                guard.push_back(Arc::new(Keypair::new()));
                            }
                        } else {
                            for _ in 0..generate_count {
                                guard.push_back(Arc::new(Keypair::new()));
                            }
                        }
                        gauge!("galileo_jito_multi_ips_wallet_pool_size").set(guard.len() as f64);
                    }
                });
            } else {
                warn!(
                    target: "lander::jito::multi_ips",
                    "未检测到 Tokio runtime，无法启动钱包生成任务"
                );
            }
        }

        Self { store }
    }

    async fn acquire(&self) -> Arc<Keypair> {
        let mut guard = self.store.lock().await;
        let wallet = guard
            .pop_front()
            .unwrap_or_else(|| Arc::new(Keypair::new()));
        gauge!("galileo_jito_multi_ips_wallet_pool_size").set(guard.len() as f64);
        wallet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjust_tip_respects_minimum() {
        assert_eq!(
            adjust_tip(0, 0),
            MIN_JITO_TIP_LAMPORTS,
            "zero tip should clamp to minimum"
        );
        assert_eq!(
            adjust_tip(5000, -1),
            4999,
            "negative offset decreases value when above minimum"
        );
        assert_eq!(
            adjust_tip(MIN_JITO_TIP_LAMPORTS, -10_000),
            MIN_JITO_TIP_LAMPORTS,
            "offset below zero clamps to minimum"
        );
        assert_eq!(
            adjust_tip(10_000, 5),
            10_005,
            "positive offset increments tip"
        );
    }
}
