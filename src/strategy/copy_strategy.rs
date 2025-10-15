use std::str::FromStr;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::api::{SwapInstructionsResponse, swap_instructions::PrioritizationType};

const DFLOW_PROGRAM: &str = "DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

const HUMIDIFI_SWAP_IDS: [u64; 4] = [
    3217692516193821715,
    14838073818726933429,
    18390110071270421531,
    5243714052430215353,
];

const ALT_ADDRESSES: &[&str] = &[
    "GicycFiT6uM9oET9w57vijz2vrmDK2dL6vVhkCyjUow",
    "3bDh8FpfSZezpiwfKZZJXFKHyoCBqu4LfQaf8Xo3kvJX",
    "6JjsmWMgQtjUrBmA1obh4NZpc2CPqLcQ9cRPd2C5WBoM",
    "7U2UmEFVRVoenZ4bviXLZ3eTC9wTBe3vTEHAFvKhmiQ5",
    "HVck7rwUxqFPNxbzdrsFGcCHDpMVEkCAoUh6ZiZXsSow",
    "FqeeFKhvn4d19iJA7G4qU9tcfjHNUkEo2rKsht5Btztm",
    "9AKCoNoAGYLW71TwTHY9e7KrZUWWL3c7VtHKb66NT3EV",
];

const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];

#[derive(Clone, Copy)]
enum AccountKind {
    Fixed(&'static str),
    Wallet,
    WalletWsolAta,
    WalletUsdcAta,
}

#[derive(Clone, Copy)]
pub struct AccountSpec {
    kind: AccountKind,
    is_writable: bool,
    is_signer: bool,
}

const fn spec(kind: AccountKind, is_writable: bool, is_signer: bool) -> AccountSpec {
    AccountSpec {
        kind,
        is_writable,
        is_signer,
    }
}

const FORWARD_ACCOUNTS: &[AccountSpec] = &[
    spec(AccountKind::Fixed(TOKEN_PROGRAM), false, false),
    spec(AccountKind::Fixed(ASSOCIATED_TOKEN_PROGRAM), false, false),
    spec(AccountKind::Fixed(SYSTEM_PROGRAM), false, false),
    spec(AccountKind::Wallet, true, true),
    spec(
        AccountKind::Fixed("8xeaWCsJYxRoudEZGJWURdfrtFhLYZz9b4iHJnW5tb3d"),
        false,
        false,
    ),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(
        AccountKind::Fixed("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        false,
        false,
    ),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(AccountKind::Wallet, true, true),
    spec(AccountKind::WalletWsolAta, true, false),
    spec(AccountKind::WalletUsdcAta, true, false),
    spec(AccountKind::Fixed(WSOL_MINT), false, false),
    spec(AccountKind::Fixed(USDC_MINT), false, false),
    spec(
        AccountKind::Fixed("Sysvar1nstructions1111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("DB3sUCP2H4icbeKmK6yb6nUxU5ogbcRHtGuq7W2RoRwW"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("8BrVfsvzb1DZqCactbYWoKSv24AfsLBuXJqzpzYCwznF"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("HsQcHFFNUVTp3MWrXYbuZchBNd4Pwk8636bKzLvpfYNR"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("6n9VhCwQ7EwK6NqFDjnHPzEk6wZdRBTfh43RFgHQWHuQ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("Cv9St5tDTGwpbG5UVvM6QvFmf3FYSXc14W9BYvQN5wAZ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("7Rf8Gu8YemSoGjZT3z1cL5BT9HLbGywcyaz8Mrbhd1MH"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FksffEqnBRixYGR791Qw2MgdU7zNCpHVFYBL4Fa4qVuH"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("C3FzbX9n1YD2dow2dCmEv5uNyyf22Gb3TLAEqGBhw5fY"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("3RWFAQBRkNGq7CMGcTLK3kXDgFTe9jgMeFYqk8nHwcWh"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("AvGeFw71N5sNfV97mZ1uNrHg4yfufRicCJUrS9j2ehTX"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("ECEPWwZJ1U1Vjsj1X5sUbZYETKMSCjYHuoTMVitCn64t"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("FBWtVVvzsRuAAzVX8ua1hden9KmgPrC2rFijuwEn1ngJ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("2ny7eGyZCoeEVTkNLf5HcnJFBKkyA4p4gcrtb3b8y8ou"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FmxXDSR9WvpJTCh738D1LEDuhMoA8geCtZgHb3isy7Dp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("CRo8DBwrmd97DJfAnvCv96tZPL5Mktf2NZy2ZnhDer1A"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("GhFfLFSprPpfoRaWakPMmJTMJBHuz6C694jYwxy2dAic"),
        true,
        false,
    ),
];

const REVERSE_ACCOUNTS: &[AccountSpec] = &[
    spec(AccountKind::Fixed(TOKEN_PROGRAM), false, false),
    spec(AccountKind::Fixed(ASSOCIATED_TOKEN_PROGRAM), false, false),
    spec(AccountKind::Fixed(SYSTEM_PROGRAM), false, false),
    spec(AccountKind::Wallet, true, true),
    spec(
        AccountKind::Fixed("8xeaWCsJYxRoudEZGJWURdfrtFhLYZz9b4iHJnW5tb3d"),
        false,
        false,
    ),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(
        AccountKind::Fixed("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        false,
        false,
    ),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(AccountKind::Fixed(DFLOW_PROGRAM), false, false),
    spec(AccountKind::Wallet, true, true),
    spec(AccountKind::WalletUsdcAta, true, false),
    spec(AccountKind::WalletWsolAta, true, false),
    spec(AccountKind::Fixed(USDC_MINT), false, false),
    spec(AccountKind::Fixed(WSOL_MINT), false, false),
    spec(
        AccountKind::Fixed("Sysvar1nstructions1111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("DB3sUCP2H4icbeKmK6yb6nUxU5ogbcRHtGuq7W2RoRwW"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("8BrVfsvzb1DZqCactbYWoKSv24AfsLBuXJqzpzYCwznF"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("HsQcHFFNUVTp3MWrXYbuZchBNd4Pwk8636bKzLvpfYNR"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("6n9VhCwQ7EwK6NqFDjnHPzEk6wZdRBTfh43RFgHQWHuQ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("Cv9St5tDTGwpbG5UVvM6QvFmf3FYSXc14W9BYvQN5wAZ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("7Rf8Gu8YemSoGjZT3z1cL5BT9HLbGywcyaz8Mrbhd1MH"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FksffEqnBRixYGR791Qw2MgdU7zNCpHVFYBL4Fa4qVuH"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("C3FzbX9n1YD2dow2dCmEv5uNyyf22Gb3TLAEqGBhw5fY"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("3RWFAQBRkNGq7CMGcTLK3kXDgFTe9jgMeFYqk8nHwcWh"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("AvGeFw71N5sNfV97mZ1uNrHg4yfufRicCJUrS9j2ehTX"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("ECEPWwZJ1U1Vjsj1X5sUbZYETKMSCjYHuoTMVitCn64t"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("FBWtVVvzsRuAAzVX8ua1hden9KmgPrC2rFijuwEn1ngJ"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("SysvarC1ock11111111111111111111111111111111"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("2ny7eGyZCoeEVTkNLf5HcnJFBKkyA4p4gcrtb3b8y8ou"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("FmxXDSR9WvpJTCh738D1LEDuhMoA8geCtZgHb3isy7Dp"),
        false,
        false,
    ),
    spec(
        AccountKind::Fixed("CRo8DBwrmd97DJfAnvCv96tZPL5Mktf2NZy2ZnhDer1A"),
        true,
        false,
    ),
    spec(
        AccountKind::Fixed("GhFfLFSprPpfoRaWakPMmJTMJBHuz6C694jYwxy2dAic"),
        true,
        false,
    ),
];

#[derive(Debug, Clone, Copy)]
pub struct CopySwapParams {
    pub amount_in: u64,
    pub reverse_amount: u64,
}

pub struct CopyStrategy {
    wallet: Pubkey,
    wsol_ata: Pubkey,
    usdc_ata: Pubkey,
    alt_addresses: Vec<Pubkey>,
    compute_unit_limit: u32,
    compute_unit_price_micro_lamports: u64,
}

impl CopyStrategy {
    pub fn new(
        wallet: Pubkey,
        wsol_ata: Pubkey,
        usdc_ata: Pubkey,
        compute_unit_limit: u32,
        compute_unit_price_micro_lamports: u64,
    ) -> Self {
        let alt_addresses = ALT_ADDRESSES
            .iter()
            .map(|addr| Pubkey::from_str(addr).expect("invalid ALT pubkey"))
            .collect();
        Self {
            wallet,
            wsol_ata,
            usdc_ata,
            alt_addresses,
            compute_unit_limit,
            compute_unit_price_micro_lamports,
        }
    }

    pub fn build_swap_instructions(&self, params: CopySwapParams) -> SwapInstructionsResponse {
        let forward = self.build_instruction(FORWARD_ACCOUNTS, params.amount_in, 0);
        let reverse =
            self.build_instruction(REVERSE_ACCOUNTS, params.reverse_amount, params.amount_in);

        let compute_budget_instructions = vec![
            compute_unit_limit_instruction(self.compute_unit_limit),
            compute_unit_price_instruction(self.compute_unit_price_micro_lamports),
        ];

        SwapInstructionsResponse {
            raw: serde_json::Value::Null,
            token_ledger_instruction: None,
            compute_budget_instructions,
            setup_instructions: vec![forward],
            swap_instruction: reverse,
            cleanup_instruction: None,
            other_instructions: Vec::new(),
            address_lookup_table_addresses: self.alt_addresses.clone(),
            prioritization_fee_lamports: 0,
            compute_unit_limit: self.compute_unit_limit,
            prioritization_type: Some(PrioritizationType::ComputeBudget {
                micro_lamports: self.compute_unit_price_micro_lamports,
                estimated_micro_lamports: Some(self.compute_unit_price_micro_lamports),
            }),
            dynamic_slippage_report: None,
            simulation_error: None,
        }
    }

    fn build_instruction(
        &self,
        specs: &[AccountSpec],
        amount_in: u64,
        quoted_out: u64,
    ) -> Instruction {
        let accounts = self.resolve_accounts(specs);
        let mut data = Vec::with_capacity(8 + 68);
        data.extend_from_slice(&SWAP_DISCRIMINATOR);
        data.extend(encode_swap_params(amount_in, quoted_out));

        Instruction {
            program_id: Pubkey::from_str(DFLOW_PROGRAM).expect("invalid dflow program"),
            accounts,
            data,
        }
    }

    fn resolve_accounts(&self, specs: &[AccountSpec]) -> Vec<AccountMeta> {
        specs
            .iter()
            .map(|spec| {
                let pubkey = match spec.kind {
                    AccountKind::Fixed(text) => {
                        Pubkey::from_str(text).expect("invalid fixed pubkey")
                    }
                    AccountKind::Wallet => self.wallet,
                    AccountKind::WalletWsolAta => self.wsol_ata,
                    AccountKind::WalletUsdcAta => self.usdc_ata,
                };
                AccountMeta {
                    pubkey,
                    is_signer: spec.is_signer,
                    is_writable: spec.is_writable,
                }
            })
            .collect()
    }
}

fn compute_unit_limit_instruction(limit: u32) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str("ComputeBudget111111111111111111111111111111")
            .expect("invalid compute budget program"),
        accounts: Vec::new(),
        data: {
            let mut data = Vec::with_capacity(5);
            data.push(0x02);
            data.extend_from_slice(&limit.to_le_bytes());
            data
        },
    }
}

fn compute_unit_price_instruction(price: u64) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str("ComputeBudget111111111111111111111111111111")
            .expect("invalid compute budget program"),
        accounts: Vec::new(),
        data: {
            let mut data = Vec::with_capacity(9);
            data.push(0x03);
            data.extend_from_slice(&price.to_le_bytes());
            data
        },
    }
}

fn encode_swap_params(amount_in: u64, quoted_out: u64) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(68);
    buffer.extend_from_slice(&1u32.to_le_bytes()); // actions vec len
    buffer.push(31); // DFlowDynamicRouteV1 variant index
    buffer.extend_from_slice(&6u32.to_le_bytes()); // candidate actions len
    buffer.push(2); // TesseraV
    for swap_id in HUMIDIFI_SWAP_IDS {
        buffer.push(3); // HumidiFi
        buffer.extend_from_slice(&swap_id.to_le_bytes());
    }
    buffer.push(4); // SolFiV2
    buffer.extend_from_slice(&amount_in.to_le_bytes());
    buffer.push(0); // orchestrator flags
    buffer.extend_from_slice(&quoted_out.to_le_bytes());
    buffer.extend_from_slice(&0u16.to_le_bytes()); // slippage_bps
    buffer.extend_from_slice(&0u16.to_le_bytes()); // platform_fee_bps
    buffer
}

pub fn compute_associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let token_program = Pubkey::from_str(TOKEN_PROGRAM).expect("invalid token program");
    let ata_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).expect("invalid ATA program");
    let (address, _) = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    );
    address
}

pub fn wsol_mint() -> Pubkey {
    Pubkey::from_str(WSOL_MINT).expect("invalid wsol mint")
}

pub fn usdc_mint() -> Pubkey {
    Pubkey::from_str(USDC_MINT).expect("invalid usdc mint")
}
