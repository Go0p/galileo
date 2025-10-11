use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TradePair {
    pub input_mint: String,
    pub output_mint: String,
}

impl TradePair {
    pub fn reversed(&self) -> TradePair {
        TradePair {
            input_mint: self.output_mint.clone(),
            output_mint: self.input_mint.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: TradePair,
    pub amount_in: u64,
    #[allow(dead_code)]
    pub first_leg_out: u64,
    #[allow(dead_code)]
    pub second_leg_out: u64,
    pub profit_lamports: u64,
    pub tip_lamports: u64,
    pub merged_quote: Value,
}

impl ArbitrageOpportunity {
    pub fn net_profit(&self) -> i128 {
        self.profit_lamports as i128 - self.tip_lamports as i128
    }
}
