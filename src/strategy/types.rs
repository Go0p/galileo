#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
