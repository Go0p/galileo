use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use solana_compute_budget_interface as compute_budget;
use solana_sdk::instruction::{AccountMeta, Instruction};
use thiserror::Error;
use tracing::debug;

use crate::api::titan::{Instruction as TitanInstruction, SwapRoute, TitanError};
use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};

/// Titan 报价源抽象，便于在单元测试中注入 mock。
#[async_trait]
pub trait TitanQuoteSource: Send + Sync {
    async fn quote(&self, intent: &QuoteIntent, side: LegSide)
    -> Result<TitanQuote, TitanLegError>;
}

/// Titan 报价结构，封装选定的 SwapRoute 及相关上下文。
#[derive(Debug, Clone)]
pub struct TitanQuote {
    pub route: SwapRoute,
    pub provider: String,
    pub quote_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TitanLegProvider<S> {
    descriptor: LegDescriptor,
    source: Arc<S>,
}

impl<S> TitanLegProvider<S> {
    pub fn new(source: S, side: LegSide) -> Self {
        Self {
            descriptor: LegDescriptor::new(AggregatorKind::Titan, side),
            source: Arc::new(source),
        }
    }
}

#[derive(Debug, Error)]
pub enum TitanLegError {
    #[error("Titan 报价源错误: {0}")]
    Source(String),
    #[error("Titan API 错误: {0}")]
    Api(#[from] TitanError),
}

impl TitanLegError {
    pub fn from_source<E: std::error::Error>(err: E) -> Self {
        TitanLegError::Source(err.to_string())
    }
}

#[async_trait]
impl<S> LegProvider for TitanLegProvider<S>
where
    S: TitanQuoteSource + Send + Sync + Debug + 'static,
{
    type QuoteResponse = TitanQuote;
    type BuildError = TitanLegError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn quote(&self, intent: &QuoteIntent) -> Result<Self::QuoteResponse, Self::BuildError> {
        debug!(
            target: "multi_leg::titan",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            side = %self.descriptor.side,
            "请求 Titan 报价"
        );
        self.source.quote(intent, self.descriptor.side).await
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        _context: &LegBuildContext,
    ) -> Result<Self::Plan, Self::BuildError> {
        let instructions = quote
            .route
            .instructions
            .iter()
            .map(convert_instruction)
            .collect::<Vec<_>>();

        let (compute_budget_instructions, other_instructions): (Vec<_>, Vec<_>) = instructions
            .into_iter()
            .partition(|ix| ix.program_id == compute_budget::id());
        let mut quote_meta = LegQuote::new(
            quote.route.in_amount,
            quote.route.out_amount,
            quote.route.slippage_bps,
        );
        quote_meta.provider = Some(quote.provider.clone());
        quote_meta.quote_id = quote.quote_id.clone();
        quote_meta.context_slot = quote.route.context_slot;
        quote_meta.expires_at_ms = quote.route.expires_at_ms;
        quote_meta.expires_after_slot = quote.route.expires_after_slot;

        Ok(LegPlan {
            descriptor: self.descriptor.clone(),
            quote: quote_meta,
            instructions: other_instructions,
            compute_budget_instructions,
            address_lookup_table_addresses: quote.route.address_lookup_tables.clone(),
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports: None,
            blockhash: None,
            raw_transaction: None,
        })
    }
}

fn convert_instruction(ix: &TitanInstruction) -> Instruction {
    let accounts = ix
        .accounts
        .iter()
        .map(|meta| AccountMeta {
            pubkey: meta.pubkey,
            is_signer: meta.signer,
            is_writable: meta.writable,
        })
        .collect::<Vec<_>>();
    Instruction {
        program_id: ix.program_id,
        accounts,
        data: ix.data.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::titan::types::AccountMeta as TitanAccountMeta;
    use crate::multi_leg::types::QuoteIntent;
    use solana_sdk::pubkey::Pubkey;
    use tokio::sync::Mutex;

    #[derive(Clone, Default, Debug)]
    struct MockSource {
        quote: Arc<Mutex<Option<TitanQuote>>>,
    }

    impl MockSource {
        fn with_quote(route: SwapRoute) -> Self {
            let quote = TitanQuote {
                route,
                provider: "Titan".to_string(),
                quote_id: Some("mock".into()),
            };
            Self {
                quote: Arc::new(Mutex::new(Some(quote))),
            }
        }
    }

    #[async_trait]
    impl TitanQuoteSource for MockSource {
        async fn quote(
            &self,
            _intent: &QuoteIntent,
            _side: LegSide,
        ) -> Result<TitanQuote, TitanLegError> {
            self.quote
                .lock()
                .await
                .take()
                .ok_or_else(|| TitanLegError::Source("no quote".into()))
        }
    }

    fn build_route() -> SwapRoute {
        let payer = Pubkey::new_unique();
        let account = Pubkey::new_unique();
        let compute_ix =
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);

        SwapRoute {
            in_amount: 100,
            out_amount: 99,
            slippage_bps: 10,
            platform_fee: None,
            steps: Vec::new(),
            instructions: vec![
                TitanInstruction {
                    program_id: compute_ix.program_id,
                    accounts: Vec::new(),
                    data: compute_ix.data,
                },
                TitanInstruction {
                    program_id: Pubkey::new_unique(),
                    accounts: vec![
                        TitanAccountMeta {
                            pubkey: payer,
                            signer: true,
                            writable: true,
                        },
                        TitanAccountMeta {
                            pubkey: account,
                            signer: false,
                            writable: false,
                        },
                    ],
                    data: vec![1, 2, 3],
                },
            ],
            address_lookup_tables: vec![Pubkey::new_unique()],
            context_slot: None,
            time_taken_ns: None,
            expires_at_ms: None,
            expires_after_slot: None,
            compute_units: None,
            compute_units_safe: None,
            transaction: None,
            reference_id: None,
        }
    }

    #[tokio::test]
    async fn titan_leg_provider_converts_instructions() {
        let route = build_route();
        let intent = QuoteIntent::new(Pubkey::new_unique(), Pubkey::new_unique(), 100, 50);
        let provider = TitanLegProvider::new(MockSource::with_quote(route), LegSide::Buy);

        let quote = provider.quote(&intent).await.expect("quote");
        let plan = provider
            .build_plan(&quote, &LegBuildContext::default())
            .await
            .expect("plan");

        assert_eq!(plan.compute_budget_instructions.len(), 1);
        assert_eq!(plan.instructions.len(), 1);
        assert_eq!(plan.address_lookup_table_addresses.len(), 1);
        assert!(plan.raw_transaction.is_none());
    }
}
