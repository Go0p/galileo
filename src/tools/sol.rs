use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;
use spl_token::solana_program::program_pack::Pack;

use crate::engine::EngineIdentity;
use crate::instructions::wsol;

/// 将 SOL 数量转换为 lamports。
pub fn decimal_sol_to_lamports(amount: &Decimal) -> Result<u64> {
    if amount <= &Decimal::ZERO {
        bail!("金额必须大于 0");
    }
    let scaled = amount
        .checked_mul(Decimal::from(LAMPORTS_PER_SOL))
        .ok_or_else(|| anyhow!("金额超出可表示范围"))?;
    if !scaled.fract().is_zero() {
        bail!("金额最多支持 9 位小数");
    }
    scaled
        .to_u64()
        .ok_or_else(|| anyhow!("金额超过 u64 最大值"))
}

/// 发送原生 SOL。
pub async fn transfer_sol(
    rpc: &Arc<RpcClient>,
    identity: &EngineIdentity,
    recipient: &Pubkey,
    lamports: u64,
    compute_unit_lamports: u64,
) -> Result<Signature> {
    if lamports == 0 {
        bail!("转账金额必须大于 0");
    }
    let mut instructions = Vec::with_capacity(2);
    maybe_insert_compute_unit_price(&mut instructions, compute_unit_lamports);
    instructions.push(system_instruction::transfer(
        &identity.pubkey,
        recipient,
        lamports,
    ));
    send_transaction(rpc, identity, &instructions, &[]).await
}

/// 将 SOL 转换为 WSOL（创建 ATA + 转账 + sync）。
pub async fn wrap_sol(
    rpc: &Arc<RpcClient>,
    identity: &EngineIdentity,
    lamports: u64,
    compute_unit_lamports: u64,
) -> Result<Signature> {
    if lamports == 0 {
        bail!("包裹金额必须大于 0");
    }
    let cached = wsol::wrap_sequence(&identity.pubkey, lamports);
    let mut instructions: Vec<Instruction> = cached.as_ref().clone();
    maybe_insert_compute_unit_price(&mut instructions, compute_unit_lamports);
    send_transaction(rpc, identity, &instructions, &[]).await
}

/// 仅解包部分 WSOL：通过临时账户获取指定 lamports。
pub async fn partial_unwrap_wsol(
    rpc: &Arc<RpcClient>,
    identity: &EngineIdentity,
    lamports: u64,
    compute_unit_lamports: u64,
) -> Result<Signature> {
    if lamports == 0 {
        bail!("解包金额必须大于 0");
    }

    let wsol_mint = wsol::WSOL_MINT;
    let wsol_ata =
        spl_associated_token_account::get_associated_token_address(&identity.pubkey, &wsol_mint);
    let account = rpc
        .get_account(&wsol_ata)
        .await
        .map_err(|err| anyhow!("查询 WSOL 账户失败: {err}"))?;
    if account.owner != spl_token::ID {
        bail!("WSOL 关联账户不属于 SPL Token Program");
    }
    let token_account = spl_token::state::Account::unpack(&account.data)
        .map_err(|err| anyhow!("解析 WSOL 账户失败: {err}"))?;
    if token_account.owner != identity.pubkey {
        bail!("WSOL 账户所有者与当前钱包不匹配");
    }
    if token_account.amount < lamports {
        bail!("WSOL 余额不足，可用 {} lamports", token_account.amount);
    }

    let rent = rpc
        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
        .await
        .map_err(|err| anyhow!("查询 rent 失败: {err}"))?;
    let temp = Keypair::new();

    let mut instructions = Vec::with_capacity(4);
    instructions.push(system_instruction::create_account(
        &identity.pubkey,
        &temp.pubkey(),
        rent,
        spl_token::state::Account::LEN as u64,
        &spl_token::ID,
    ));
    instructions.push(
        spl_token::instruction::initialize_account3(
            &spl_token::ID,
            &temp.pubkey(),
            &wsol::WSOL_MINT,
            &identity.pubkey,
        )
        .map_err(|err| anyhow!("初始化临时 WSOL 账户失败: {err}"))?,
    );
    instructions.push(
        spl_token::instruction::transfer(
            &spl_token::ID,
            &wsol_ata,
            &temp.pubkey(),
            &identity.pubkey,
            &[],
            lamports,
        )
        .map_err(|err| anyhow!("WSOL 转账失败: {err}"))?,
    );
    instructions.push(
        spl_token::instruction::close_account(
            &spl_token::ID,
            &temp.pubkey(),
            &identity.pubkey,
            &identity.pubkey,
            &[],
        )
        .map_err(|err| anyhow!("关闭临时 WSOL 账户失败: {err}"))?,
    );

    maybe_insert_compute_unit_price(&mut instructions, compute_unit_lamports);
    let extra_signers = [&temp];
    send_transaction(rpc, identity, &instructions, &extra_signers).await
}

fn maybe_insert_compute_unit_price(
    instructions: &mut Vec<Instruction>,
    compute_unit_lamports: u64,
) {
    if compute_unit_lamports == 0 {
        return;
    }
    instructions.insert(
        0,
        ComputeBudgetInstruction::set_compute_unit_price(compute_unit_lamports),
    );
}

async fn send_transaction(
    rpc: &Arc<RpcClient>,
    identity: &EngineIdentity,
    instructions: &[Instruction],
    extra_signers: &[&Keypair],
) -> Result<Signature> {
    if instructions.is_empty() {
        bail!("交易指令不能为空");
    }
    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .map_err(|err| anyhow!("获取最新区块哈希失败: {err}"))?;
    let mut signer_refs: Vec<&dyn Signer> = Vec::with_capacity(1 + extra_signers.len());
    signer_refs.push(identity.signer.as_ref());
    for signer in extra_signers {
        signer_refs.push(*signer);
    }
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&identity.pubkey),
        &signer_refs,
        blockhash,
    );
    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .await
        .map_err(|err| anyhow!("交易提交失败: {err}"))?;
    Ok(signature)
}
