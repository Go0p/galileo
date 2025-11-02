use anyhow::Result;
use borsh::BorshSerialize;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use super::types::{JUPITER_V6_EVENT_AUTHORITY, JUPITER_V6_PROGRAM_ID, RoutePlanStepV2};

const ROUTE_V2_DISCRIMINATOR: [u8; 8] = [187, 100, 250, 204, 49, 196, 175, 20];

#[derive(Clone, Debug)]
pub struct RouteV2Accounts {
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub source_token_program: Pubkey,
    pub destination_token_program: Pubkey,
    pub destination_token_account: Option<Pubkey>,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub remaining_accounts: Vec<AccountMeta>,
}

impl RouteV2Accounts {
    pub fn with_defaults(
        user_transfer_authority: Pubkey,
        user_source_token_account: Pubkey,
        user_destination_token_account: Pubkey,
        source_mint: Pubkey,
        destination_mint: Pubkey,
        source_token_program: Pubkey,
        destination_token_program: Pubkey,
    ) -> Self {
        Self {
            user_transfer_authority,
            user_source_token_account,
            user_destination_token_account,
            source_mint,
            destination_mint,
            source_token_program,
            destination_token_program,
            destination_token_account: None,
            event_authority: JUPITER_V6_EVENT_AUTHORITY,
            program: JUPITER_V6_PROGRAM_ID,
            remaining_accounts: Vec::new(),
        }
    }

    pub fn to_account_metas(&self) -> Vec<AccountMeta> {
        let mut accounts = Vec::with_capacity(9 + self.remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(
            self.user_transfer_authority,
            true,
        ));
        accounts.push(AccountMeta::new(self.user_source_token_account, false));
        accounts.push(AccountMeta::new(self.user_destination_token_account, false));
        accounts.push(AccountMeta::new_readonly(self.source_mint, false));
        accounts.push(AccountMeta::new_readonly(self.destination_mint, false));
        accounts.push(AccountMeta::new_readonly(self.source_token_program, false));
        accounts.push(AccountMeta::new_readonly(
            self.destination_token_program,
            false,
        ));
        if let Some(destination_token_account) = self.destination_token_account {
            accounts.push(AccountMeta::new(destination_token_account, false));
        }
        accounts.push(AccountMeta::new_readonly(self.event_authority, false));
        accounts.push(AccountMeta::new_readonly(self.program, false));
        accounts.extend(self.remaining_accounts.iter().cloned());
        accounts
    }
}

#[derive(BorshSerialize)]
struct RouteV2InstructionArgs {
    in_amount: u64,
    quoted_out_amount: u64,
    slippage_bps: u16,
    platform_fee_bps: u16,
    positive_slippage_bps: u16,
    route_plan: Vec<RoutePlanStepV2>,
}

#[derive(Clone, Debug)]
pub struct RouteV2InstructionBuilder {
    pub accounts: RouteV2Accounts,
    pub route_plan: Vec<RoutePlanStepV2>,
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u16,
    pub positive_slippage_bps: u16,
}

impl RouteV2InstructionBuilder {
    pub fn build(self) -> Result<Instruction> {
        let mut data = Vec::new();
        data.extend_from_slice(&ROUTE_V2_DISCRIMINATOR);
        let args = RouteV2InstructionArgs {
            in_amount: self.in_amount,
            quoted_out_amount: self.quoted_out_amount,
            slippage_bps: self.slippage_bps,
            platform_fee_bps: self.platform_fee_bps,
            positive_slippage_bps: self.positive_slippage_bps,
            route_plan: self.route_plan,
        };
        args.serialize(&mut data)?;

        Ok(Instruction {
            program_id: self.accounts.program,
            accounts: self.accounts.to_account_metas(),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::jupiter::types::EncodedSwap;

    #[test]
    fn build_route_v2_instruction() {
        let accounts = RouteV2Accounts {
            user_transfer_authority: Pubkey::new_unique(),
            user_source_token_account: Pubkey::new_unique(),
            user_destination_token_account: Pubkey::new_unique(),
            source_mint: Pubkey::new_unique(),
            destination_mint: Pubkey::new_unique(),
            source_token_program: Pubkey::new_unique(),
            destination_token_program: Pubkey::new_unique(),
            destination_token_account: Some(Pubkey::new_unique()),
            event_authority: JUPITER_V6_EVENT_AUTHORITY,
            program: JUPITER_V6_PROGRAM_ID,
            remaining_accounts: vec![
                AccountMeta::new_readonly(Pubkey::new_unique(), false),
                AccountMeta::new(Pubkey::new_unique(), false),
            ],
        };

        let route_plan = vec![RoutePlanStepV2 {
            swap: EncodedSwap::simple(1),
            bps: 10_000,
            input_index: 0,
            output_index: 1,
        }];

        let builder = RouteV2InstructionBuilder {
            accounts,
            route_plan: route_plan.clone(),
            in_amount: 500,
            quoted_out_amount: 500,
            slippage_bps: 10,
            platform_fee_bps: 0,
            positive_slippage_bps: 20,
        };

        let instruction = builder.build().expect("instruction");
        assert_eq!(instruction.program_id, JUPITER_V6_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 10 + 2); // 10 基础账户 + 2 个剩余账户
        assert_eq!(&instruction.data[0..8], &ROUTE_V2_DISCRIMINATOR);

        let mut expected = Vec::new();
        RouteV2InstructionArgs {
            in_amount: 500,
            quoted_out_amount: 500,
            slippage_bps: 10,
            platform_fee_bps: 0,
            positive_slippage_bps: 20,
            route_plan,
        }
        .serialize(&mut expected)
        .unwrap();

        assert_eq!(&instruction.data[8..], expected.as_slice());
    }
}
