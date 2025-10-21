use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

/// Jupiter route_v2 固定的 10 个账户顺序。
#[allow(dead_code)]
pub struct RouteV2CommonAccounts {
    pub authority: Pubkey,
    pub user_base: Pubkey,
    pub user_quote: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub program: Pubkey,
    pub event_authority: Pubkey,
}

impl RouteV2CommonAccounts {
    #[allow(dead_code)]
    pub fn to_account_metas(&self) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new(self.user_base, false),
            AccountMeta::new(self.user_quote, false),
            AccountMeta::new_readonly(self.base_mint, false),
            AccountMeta::new_readonly(self.quote_mint, false),
            AccountMeta::new_readonly(self.base_token_program, false),
            AccountMeta::new_readonly(self.quote_token_program, false),
            AccountMeta::new_readonly(self.event_authority, false),
            AccountMeta::new_readonly(self.program, false),
        ]
    }
}

#[allow(dead_code)]
pub struct RemainingAccountsBuilder<'a> {
    buffer: &'a mut Vec<AccountMeta>,
}

#[allow(dead_code)]
impl<'a> RemainingAccountsBuilder<'a> {
    pub fn new(buffer: &'a mut Vec<AccountMeta>) -> Self {
        buffer.clear();
        Self { buffer }
    }

    pub fn push(&mut self, meta: AccountMeta) {
        self.buffer.push(meta);
    }

    pub fn extend(&mut self, metas: impl IntoIterator<Item = AccountMeta>) {
        self.buffer.extend(metas);
    }

    pub fn finish(self) -> Vec<AccountMeta> {
        self.buffer.clone()
    }
}
