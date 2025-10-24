use once_cell::sync::Lazy;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use super::super::{
    FlashloanMetadata, FlashloanOutcome, FlashloanProtocol,
    error::{FlashloanError, FlashloanResult},
};
use super::{
    BEGIN_DISCRIMINATOR, BORROW_DISCRIMINATOR, END_DISCRIMINATOR, GROUP_ID, PROGRAM_ID,
    REPAY_DISCRIMINATOR, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID, parse_pubkey,
};
use crate::engine::EngineIdentity;

use super::compute_associated_token_address;

#[derive(Debug, Clone)]
struct MarginfiAsset {
    mint: Pubkey,
    bank: Pubkey,
    token_program: Pubkey,
    remaining_accounts: Vec<Pubkey>,
}

static MARGINFI_ASSETS: Lazy<Vec<MarginfiAsset>> = Lazy::new(|| {
    vec![
        MarginfiAsset {
            mint: parse_pubkey("So11111111111111111111111111111111111111112"),
            bank: parse_pubkey("CCKtUs6Cgwo4aaQUmBPmyoApH2gUDErxNZCAntD6LYGh"),
            token_program: *TOKEN_PROGRAM_ID,
            remaining_accounts: vec![parse_pubkey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG")],
        },
        MarginfiAsset {
            mint: parse_pubkey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            bank: parse_pubkey("2s37akK2eyBbp8DZgCm7RtsaEz8eJP3Nxd4urLHQv7yB"),
            token_program: *TOKEN_PROGRAM_ID,
            remaining_accounts: vec![parse_pubkey("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX")],
        },
    ]
});

#[derive(Debug, Clone)]
pub struct MarginfiFlashloan {
    account: Pubkey,
}

impl MarginfiFlashloan {
    pub fn new(account: Pubkey) -> Self {
        Self { account }
    }

    pub fn account(&self) -> Pubkey {
        self.account
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn wrap(
        &self,
        identity: &EngineIdentity,
        base_mint: &Pubkey,
        mut prefix: Vec<Instruction>,
        mut body: Vec<Instruction>,
        borrow_amount: u64,
    ) -> FlashloanResult<FlashloanOutcome> {
        if borrow_amount == 0 {
            prefix.append(&mut body);
            return Ok(FlashloanOutcome {
                instructions: prefix,
                metadata: None,
            });
        }

        let asset = self
            .resolve_asset(base_mint)
            .ok_or_else(|| FlashloanError::UnsupportedAsset(base_mint.to_string()))?;

        let destination = compute_associated_token_address(&identity.pubkey, &asset.mint);
        let liquidity_vault = find_liquidity_vault(&asset.bank);
        let liquidity_vault_authority = find_liquidity_vault_authority(&asset.bank);

        let borrow_ix = build_borrow(
            MarginfiBorrowAccounts {
                group: *GROUP_ID,
                marginfi_account: self.account,
                authority: identity.pubkey,
                bank: asset.bank,
                destination_token_account: destination,
                liquidity_vault_authority,
                liquidity_vault,
                token_program: asset.token_program,
            },
            borrow_amount,
        );

        let repay_ix = build_repay(
            MarginfiRepayAccounts {
                group: *GROUP_ID,
                marginfi_account: self.account,
                authority: identity.pubkey,
                bank: asset.bank,
                signer_token_account: destination,
                liquidity_vault,
                token_program: asset.token_program,
            },
            borrow_amount,
        );

        let inner_count = body.len();
        let start_index = prefix.len();
        let end_index = (start_index + inner_count + 2 + 1) as u64;

        let begin_ix = build_begin(self.account, identity.pubkey, end_index);
        let end_ix = build_end(
            self.account,
            identity.pubkey,
            asset.bank,
            &asset.remaining_accounts,
        );

        let mut instructions = Vec::with_capacity(prefix.len() + inner_count + 4);
        instructions.append(&mut prefix);
        instructions.push(begin_ix);
        instructions.push(borrow_ix);
        instructions.append(&mut body);
        instructions.push(repay_ix);
        instructions.push(end_ix);

        Ok(FlashloanOutcome {
            instructions,
            metadata: Some(FlashloanMetadata {
                protocol: FlashloanProtocol::Marginfi,
                mint: asset.mint,
                borrow_amount,
                inner_instruction_count: inner_count,
            }),
        })
    }

    fn resolve_asset(&self, mint: &Pubkey) -> Option<&'static MarginfiAsset> {
        MARGINFI_ASSETS.iter().find(|asset| asset.mint == *mint)
    }
}

pub fn build_initialize_instruction(
    marginfi_account: Pubkey,
    authority: &Pubkey,
) -> FlashloanResult<Instruction> {
    if marginfi_account == Pubkey::default() {
        return Err(FlashloanError::InvalidConfig("缺少 marginfi account"));
    }
    if *authority == Pubkey::default() {
        return Err(FlashloanError::InvalidConfig("缺少 authority 公钥"));
    }

    let mut data = Vec::with_capacity(super::ACCOUNT_INITIALIZE_DISCRIMINATOR.len());
    data.extend_from_slice(&super::ACCOUNT_INITIALIZE_DISCRIMINATOR);

    let accounts = vec![
        AccountMeta::new(*super::GROUP_ID, false),
        AccountMeta::new(marginfi_account, true),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(*SYSTEM_PROGRAM_ID, false),
    ];

    Ok(Instruction {
        program_id: *PROGRAM_ID,
        accounts,
        data,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn build_close_instruction(
    marginfi_account: Pubkey,
    authority: &Pubkey,
    fee_payer: &Pubkey,
) -> FlashloanResult<Instruction> {
    if marginfi_account == Pubkey::default() {
        return Err(FlashloanError::InvalidConfig("缺少 marginfi account"));
    }
    if *authority == Pubkey::default() {
        return Err(FlashloanError::InvalidConfig("缺少 authority 公钥"));
    }
    if *fee_payer == Pubkey::default() {
        return Err(FlashloanError::InvalidConfig("缺少 fee payer 公钥"));
    }

    Ok(Instruction {
        program_id: *PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(marginfi_account, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*fee_payer, true),
        ],
        data: super::CLOSE_ACCOUNT_DISCRIMINATOR.to_vec(),
    })
}

#[derive(Debug, Clone, Copy)]
struct MarginfiBorrowAccounts {
    group: Pubkey,
    marginfi_account: Pubkey,
    authority: Pubkey,
    bank: Pubkey,
    destination_token_account: Pubkey,
    liquidity_vault_authority: Pubkey,
    liquidity_vault: Pubkey,
    token_program: Pubkey,
}

#[derive(Debug, Clone, Copy)]
struct MarginfiRepayAccounts {
    group: Pubkey,
    marginfi_account: Pubkey,
    authority: Pubkey,
    bank: Pubkey,
    signer_token_account: Pubkey,
    liquidity_vault: Pubkey,
    token_program: Pubkey,
}

fn build_begin(marginfi_account: Pubkey, authority: Pubkey, end_index: u64) -> Instruction {
    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&BEGIN_DISCRIMINATOR);
    data[8..].copy_from_slice(&end_index.to_le_bytes());

    Instruction {
        program_id: *PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(marginfi_account, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
        ],
        data: data.to_vec(),
    }
}

fn build_end(
    marginfi_account: Pubkey,
    authority: Pubkey,
    bank: Pubkey,
    remaining: &[Pubkey],
) -> Instruction {
    let mut data = [0u8; 8];
    data.copy_from_slice(&END_DISCRIMINATOR);

    let mut accounts = Vec::with_capacity(remaining.len() + 3);
    accounts.push(AccountMeta::new(marginfi_account, false));
    accounts.push(AccountMeta::new_readonly(authority, true));
    accounts.push(AccountMeta::new(bank, false));

    for account in remaining {
        if *account != Pubkey::default() {
            accounts.push(AccountMeta::new_readonly(*account, false));
        }
    }

    Instruction {
        program_id: *PROGRAM_ID,
        accounts,
        data: data.to_vec(),
    }
}

fn build_borrow(accounts: MarginfiBorrowAccounts, amount: u64) -> Instruction {
    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&BORROW_DISCRIMINATOR);
    data[8..].copy_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(accounts.group, false),
            AccountMeta::new(accounts.marginfi_account, false),
            AccountMeta::new_readonly(accounts.authority, true),
            AccountMeta::new(accounts.bank, false),
            AccountMeta::new(accounts.destination_token_account, false),
            AccountMeta::new_readonly(accounts.liquidity_vault_authority, false),
            AccountMeta::new(accounts.liquidity_vault, false),
            AccountMeta::new_readonly(accounts.token_program, false),
        ],
        data: data.to_vec(),
    }
}

fn build_repay(accounts: MarginfiRepayAccounts, amount: u64) -> Instruction {
    let mut data = Vec::with_capacity(18);
    data.extend_from_slice(&REPAY_DISCRIMINATOR);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(0);

    Instruction {
        program_id: *PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(accounts.group, false),
            AccountMeta::new(accounts.marginfi_account, false),
            AccountMeta::new_readonly(accounts.authority, true),
            AccountMeta::new(accounts.bank, false),
            AccountMeta::new(accounts.signer_token_account, false),
            AccountMeta::new(accounts.liquidity_vault, false),
            AccountMeta::new_readonly(accounts.token_program, false),
        ],
        data,
    }
}

fn find_liquidity_vault(bank: &Pubkey) -> Pubkey {
    let (address, _) =
        Pubkey::find_program_address(&[b"liquidity_vault", bank.as_ref()], &PROGRAM_ID);
    address
}

fn find_liquidity_vault_authority(bank: &Pubkey) -> Pubkey {
    let (address, _) =
        Pubkey::find_program_address(&[b"liquidity_vault_auth", bank.as_ref()], &PROGRAM_ID);
    address
}
