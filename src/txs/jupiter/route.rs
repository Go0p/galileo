#[allow(dead_code)]
use anyhow::Result;
use borsh::BorshSerialize;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use super::types::{JUPITER_V6_EVENT_AUTHORITY, JUPITER_V6_PROGRAM_ID, RoutePlanStep};

#[allow(dead_code)]
const ROUTE_DISCRIMINATOR: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RouteAccounts {
    pub token_program: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub destination_token_account: Option<Pubkey>,
    pub destination_mint: Pubkey,
    pub platform_fee_account: Option<Pubkey>,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub remaining_accounts: Vec<AccountMeta>,
}

#[allow(dead_code)]
impl RouteAccounts {
    #[allow(dead_code)]
    pub fn with_defaults(
        token_program: Pubkey,
        user_transfer_authority: Pubkey,
        user_source_token_account: Pubkey,
        user_destination_token_account: Pubkey,
        destination_mint: Pubkey,
    ) -> Self {
        Self {
            token_program,
            user_transfer_authority,
            user_source_token_account,
            user_destination_token_account,
            destination_token_account: None,
            destination_mint,
            platform_fee_account: None,
            event_authority: JUPITER_V6_EVENT_AUTHORITY,
            program: JUPITER_V6_PROGRAM_ID,
            remaining_accounts: Vec::new(),
        }
    }

    pub fn to_account_metas(&self) -> Vec<AccountMeta> {
        let mut accounts = Vec::with_capacity(8 + self.remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(self.token_program, false));
        accounts.push(AccountMeta::new_readonly(
            self.user_transfer_authority,
            true,
        ));
        accounts.push(AccountMeta::new(self.user_source_token_account, false));
        accounts.push(AccountMeta::new(self.user_destination_token_account, false));
        if let Some(destination_token_account) = self.destination_token_account {
            accounts.push(AccountMeta::new(destination_token_account, false));
        }
        accounts.push(AccountMeta::new_readonly(self.destination_mint, false));
        if let Some(platform_fee_account) = self.platform_fee_account {
            accounts.push(AccountMeta::new(platform_fee_account, false));
        }
        accounts.push(AccountMeta::new_readonly(self.event_authority, false));
        accounts.push(AccountMeta::new_readonly(self.program, false));
        accounts.extend(self.remaining_accounts.iter().cloned());
        accounts
    }
}

#[derive(BorshSerialize)]
#[allow(dead_code)]
struct RouteInstructionArgs {
    route_plan: Vec<RoutePlanStep>,
    in_amount: u64,
    quoted_out_amount: u64,
    slippage_bps: u16,
    platform_fee_bps: u8,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RouteInstructionBuilder {
    pub accounts: RouteAccounts,
    pub route_plan: Vec<RoutePlanStep>,
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

#[allow(dead_code)]
impl RouteInstructionBuilder {
    pub fn build(self) -> Result<Instruction> {
        let mut data = Vec::new();
        data.extend_from_slice(&ROUTE_DISCRIMINATOR);
        let args = RouteInstructionArgs {
            route_plan: self.route_plan,
            in_amount: self.in_amount,
            quoted_out_amount: self.quoted_out_amount,
            slippage_bps: self.slippage_bps,
            platform_fee_bps: self.platform_fee_bps,
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
    use crate::txs::jupiter::types::EncodedSwap;

    #[test]
    fn build_route_instruction() {
        let accounts = RouteAccounts {
            token_program: Pubkey::new_unique(),
            user_transfer_authority: Pubkey::new_unique(),
            user_source_token_account: Pubkey::new_unique(),
            user_destination_token_account: Pubkey::new_unique(),
            destination_token_account: None,
            destination_mint: Pubkey::new_unique(),
            platform_fee_account: None,
            event_authority: JUPITER_V6_EVENT_AUTHORITY,
            program: JUPITER_V6_PROGRAM_ID,
            remaining_accounts: vec![AccountMeta::new(Pubkey::new_unique(), false)],
        };

        let route_plan = vec![RoutePlanStep {
            swap: EncodedSwap::simple(0),
            percent: 100,
            input_index: 0,
            output_index: 1,
        }];

        let builder = RouteInstructionBuilder {
            accounts,
            route_plan: route_plan.clone(),
            in_amount: 1_000,
            quoted_out_amount: 1_000,
            slippage_bps: 50,
            platform_fee_bps: 0,
        };

        let instruction = builder.build().expect("instruction");
        assert_eq!(instruction.program_id, JUPITER_V6_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 7 + 1); // 7 固定账户 + 剩余账户
        assert_eq!(&instruction.data[0..8], &ROUTE_DISCRIMINATOR);

        let mut expected = Vec::new();
        RouteInstructionArgs {
            route_plan,
            in_amount: 1_000,
            quoted_out_amount: 1_000,
            slippage_bps: 50,
            platform_fee_bps: 0,
        }
        .serialize(&mut expected)
        .unwrap();

        assert_eq!(&instruction.data[8..], expected.as_slice());
    }
}
