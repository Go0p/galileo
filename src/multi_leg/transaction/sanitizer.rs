use std::collections::BTreeSet;

use solana_compute_budget_interface as compute_budget;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::message::{Message, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;

/// 从交易中移除 Compute Budget Program 指令，返回新的交易副本。
pub fn strip_compute_budget(tx: &VersionedTransaction) -> VersionedTransaction {
    strip_programs(tx, [compute_budget::id()])
}

/// 依据指定的 Program ID 列表过滤交易内的指令。
pub fn strip_programs<I>(tx: &VersionedTransaction, programs: I) -> VersionedTransaction
where
    I: IntoIterator<Item = Pubkey>,
{
    let deny: BTreeSet<Pubkey> = programs.into_iter().collect();
    if deny.is_empty() {
        return tx.clone();
    }

    let mut cloned = tx.clone();
    match &mut cloned.message {
        VersionedMessage::Legacy(message) => {
            filter_legacy_instructions(message, &deny);
        }
        VersionedMessage::V0(message) => {
            filter_v0_instructions(message, &deny);
        }
    }
    cloned
}

fn filter_legacy_instructions(message: &mut Message, deny: &BTreeSet<Pubkey>) {
    message.instructions.retain(|ix| {
        let program = message
            .account_keys
            .get(ix.program_id_index as usize)
            .copied();
        program.map(|pid| !deny.contains(&pid)).unwrap_or(true)
    });
}

fn filter_v0_instructions(message: &mut V0Message, deny: &BTreeSet<Pubkey>) {
    let static_len = message.account_keys.len();
    message.instructions.retain(|ix| {
        let idx = ix.program_id_index as usize;
        if let Some(pid) = message.account_keys.get(idx) {
            return !deny.contains(pid);
        }
        if idx < static_len {
            return true;
        }
        // 程序 ID 落在 ALT 区域，缺乏查表上下文无法识别，默认保留。
        true
    });
}

#[cfg(test)]
mod tests {
    use solana_compute_budget_interface::ComputeBudgetInstruction;
    use solana_sdk::instruction::Instruction;
    use solana_sdk::message::Message;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::transaction::VersionedMessage;

    use super::*;

    #[test]
    fn strip_compute_budget_in_legacy_message() {
        let payer = Pubkey::new_unique();
        let compute = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
        let transfer = Instruction::new_with_bytes(Pubkey::new_unique(), &[7, 8, 9], vec![]);
        let message = Message::new(&[compute, transfer], Some(&payer));
        let tx = VersionedTransaction {
            signatures: vec![],
            message: VersionedMessage::Legacy(message),
        };

        let sanitized = strip_compute_budget(&tx);
        if let VersionedMessage::Legacy(message) = sanitized.message {
            assert_eq!(message.instructions.len(), 1);
        } else {
            panic!("unexpected message version")
        }
    }
}
