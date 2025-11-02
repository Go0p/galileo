use std::sync::Arc;

use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;

use crate::engine::plugins::flashloan::{FlashloanMetadata, marginfi::MarginfiFlashloanManager};
use crate::engine::{
    EngineIdentity, EngineResult, JitoTipPlan, LighthouseRuntime, SwapInstructionsVariant,
    SwapOpportunity,
};
use crate::monitoring::events;

use super::bundle::InstructionBundle;

pub mod compute_budget;
pub mod flashloan;
pub mod guard_budget;
pub mod profit_guard;
pub mod tip;

pub use compute_budget::ComputeBudgetDecorator;
pub use flashloan::FlashloanDecorator;
pub use guard_budget::GuardBudgetDecorator;
pub use profit_guard::ProfitGuardDecorator;
pub use tip::TipDecorator;

/// 装配上下文，在装饰器执行过程中传递交易相关信息。
pub struct AssemblyContext<'a> {
    pub identity: &'a EngineIdentity,
    pub base_mint: Option<&'a Pubkey>,
    pub compute_unit_limit: u32,
    pub compute_unit_price: Option<u64>,
    pub guard_required: u64,
    pub prioritization_fee: u64,
    pub tip_lamports: u64,
    pub jito_tip_budget: u64,
    pub jito_tip_plan: Option<JitoTipPlan>,
    pub variant: Option<&'a mut SwapInstructionsVariant>,
    pub opportunity: Option<&'a SwapOpportunity>,
    pub flashloan_manager: Option<&'a MarginfiFlashloanManager>,
    pub flashloan_metadata: Option<FlashloanMetadata>,
    lighthouse: Option<&'a mut LighthouseRuntime>,
}

impl<'a> AssemblyContext<'a> {
    pub fn new(identity: &'a EngineIdentity) -> Self {
        Self {
            identity,
            base_mint: None,
            compute_unit_limit: 0,
            compute_unit_price: None,
            guard_required: 0,
            prioritization_fee: 0,
            tip_lamports: 0,
            jito_tip_budget: 0,
            jito_tip_plan: None,
            variant: None,
            opportunity: None,
            flashloan_manager: None,
            flashloan_metadata: None,
            lighthouse: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::config::types::{AutoUnwrapConfig, FlashloanMarginfiConfig};
    use crate::engine::aggregator::MultiLegInstructions;
    use crate::engine::assembly::bundle::InstructionBundle;
    use crate::engine::plugins::flashloan::{
        MarginfiAccountRegistry, MarginfiFlashloanManager, MarginfiFlashloanPreparation,
    };
    use crate::engine::types::{JitoTipPlan, SwapOpportunity};
    use crate::engine::{EngineIdentity, LighthouseSettings};
    use crate::instructions::compute_budget::{
        compute_unit_limit_instruction, compute_unit_price_instruction,
    };
    use crate::instructions::flashloan::types::FlashloanProtocol;
    use crate::instructions::guards::lighthouse::program::LIGHTHOUSE_PROGRAM_ID;
    use crate::strategy::types::TradePair;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::instruction::Instruction;
    use solana_sdk::message::AddressLookupTableAccount;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Keypair;
    use std::sync::Arc;

    fn dummy_identity() -> EngineIdentity {
        let keypair = Keypair::new();
        let private_key = format!(
            "[{}]",
            keypair
                .to_bytes()
                .iter()
                .map(|byte| byte.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        EngineIdentity::from_private_key(&private_key).expect("identity from private key")
    }

    fn dummy_instruction() -> Instruction {
        Instruction {
            program_id: Pubkey::new_unique(),
            accounts: Vec::new(),
            data: vec![42],
        }
    }

    fn sample_multi_leg_variant() -> SwapInstructionsVariant {
        let lookup_address = Pubkey::new_unique();
        let lookup_entry = AddressLookupTableAccount {
            key: lookup_address,
            addresses: vec![Pubkey::new_unique()],
        };
        let compute_limit = compute_unit_limit_instruction(200_000);
        let compute_price = compute_unit_price_instruction(1_000);
        let swap_instruction = dummy_instruction();
        let response = MultiLegInstructions::new(
            vec![compute_limit.clone(), compute_price.clone()],
            vec![swap_instruction],
            vec![lookup_address],
            vec![lookup_entry],
            Some(0),
            0,
        );
        SwapInstructionsVariant::MultiLeg(response)
    }

    fn sample_jupiter_variant() -> SwapInstructionsVariant {
        sample_multi_leg_variant()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn compute_budget_decorator_updates_bundle_and_variant() {
        let identity = dummy_identity();
        let mut variant = sample_multi_leg_variant();
        let mut bundle = InstructionBundle::from_instructions(variant.flatten_instructions());
        bundle.set_lookup_tables(
            variant.address_lookup_table_addresses().to_vec(),
            variant.resolved_lookup_tables().to_vec(),
        );
        let expected_lookups = bundle.lookup_addresses.clone();

        let mut ctx = AssemblyContext::new(&identity);
        ctx.compute_unit_limit = 400_000;
        ctx.compute_unit_price = Some(5_000);
        ctx.variant = Some(&mut variant);

        ComputeBudgetDecorator
            .apply(&mut bundle, &mut ctx)
            .await
            .expect("compute decorator");

        ctx.variant = None;
        assert_eq!(ctx.compute_unit_limit, 400_000);

        let _ = bundle.flatten();
        assert_eq!(bundle.compute_budget.len(), 2);
        let mut limit_bytes = [0u8; 4];
        let limit_ix = bundle
            .compute_budget
            .iter()
            .find(|ix| ix.data.first() == Some(&2))
            .expect("limit instruction");
        limit_bytes.copy_from_slice(&limit_ix.data[1..5]);
        assert_eq!(u32::from_le_bytes(limit_bytes), 400_000);

        let mut price_bytes = [0u8; 8];
        let price_ix = bundle
            .compute_budget
            .iter()
            .find(|ix| ix.data.first() == Some(&3))
            .expect("price instruction");
        price_bytes.copy_from_slice(&price_ix.data[1..9]);
        assert_eq!(u64::from_le_bytes(price_bytes), 5_000);
        assert_eq!(bundle.lookup_addresses, expected_lookups);

        if let SwapInstructionsVariant::MultiLeg(response) = &variant {
            assert_eq!(response.compute_unit_limit, 400_000);
            let mut response_limit = [0u8; 4];
            let limit_ix = response
                .compute_budget_instructions
                .iter()
                .find(|ix| ix.data.first() == Some(&2))
                .expect("response limit");
            response_limit.copy_from_slice(&limit_ix.data[1..5]);
            assert_eq!(u32::from_le_bytes(response_limit), 400_000);

            let mut response_price = [0u8; 8];
            let price_ix = response
                .compute_budget_instructions
                .iter()
                .find(|ix| ix.data.first() == Some(&3))
                .expect("response price");
            response_price.copy_from_slice(&price_ix.data[1..9]);
            assert_eq!(u64::from_le_bytes(response_price), 5_000);
        } else {
            panic!("expected MultiLeg variant");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn guard_and_profit_decorators_apply_lighthouse_guard() {
        let identity = dummy_identity();
        let mut bundle = InstructionBundle::default();
        let base_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

        let mut ctx = AssemblyContext::new(&identity);
        ctx.base_mint = Some(&base_mint);
        ctx.guard_required = 5_000;
        ctx.prioritization_fee = 1_000;
        ctx.jito_tip_budget = 2_000;
        ctx.tip_lamports = 2_000;
        ctx.jito_tip_plan = Some(JitoTipPlan::new(2_000, Pubkey::new_unique()));

        let settings = LighthouseSettings {
            enable: true,
            profit_guard_mints: vec![base_mint],
            memory_slots: Some(4),
            existing_memory_ids: Vec::new(),
            sol_price_feed: None,
        };
        let mut lighthouse = LighthouseRuntime::new(&settings, 4);
        super::attach_lighthouse_internal(&mut ctx, &mut lighthouse);

        let mut chain = DecoratorChain::new();
        chain.register(TipDecorator);
        chain.register(GuardBudgetDecorator);
        chain.register(ProfitGuardDecorator);

        chain
            .apply_all(&mut bundle, &mut ctx)
            .await
            .expect("guard chain");

        assert_eq!(ctx.guard_required, 8_000);
        assert_eq!(bundle.pre.len(), 1);
        assert_eq!(bundle.post.len(), 2);
        assert_eq!(
            bundle.post[0].program_id,
            solana_sdk::pubkey!("11111111111111111111111111111111")
        );
        assert_eq!(bundle.pre[0].program_id, LIGHTHOUSE_PROGRAM_ID);
        assert_eq!(bundle.post[1].program_id, LIGHTHOUSE_PROGRAM_ID);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn flashloan_decorator_wraps_bundle_and_sets_metadata() {
        let identity = dummy_identity();
        let mut variant = sample_jupiter_variant();
        let baseline = variant.flatten_instructions().len();
        let mut bundle = InstructionBundle::from_instructions(variant.flatten_instructions());
        bundle.set_lookup_tables(
            variant.address_lookup_table_addresses().to_vec(),
            variant.resolved_lookup_tables().to_vec(),
        );
        let expected_lookups = bundle.lookup_addresses.clone();

        let mut cfg = FlashloanMarginfiConfig::default();
        cfg.compute_unit_overhead = 15_000;

        let rpc = Arc::new(RpcClient::new("http://127.0.0.1:8899".to_string()));
        let marginfi_account = Pubkey::new_unique();
        let registry = MarginfiAccountRegistry::new(Some(marginfi_account));
        let mut manager =
            MarginfiFlashloanManager::new(&cfg, true, false, Arc::clone(&rpc), registry);
        manager.adopt_preparation(MarginfiFlashloanPreparation {
            account: marginfi_account,
            created: false,
        });

        let base_mint = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
        let opportunity = SwapOpportunity {
            pair: TradePair::from_pubkeys(base_mint, Pubkey::new_unique()),
            amount_in: 1_000_000,
            profit_lamports: 0,
            tip_lamports: 0,
            merged_quote: None,
            ultra_legs: None,
        };

        let mut ctx = AssemblyContext::new(&identity);
        ctx.base_mint = Some(&base_mint);
        ctx.compute_unit_limit = 200_000;
        ctx.variant = Some(&mut variant);
        ctx.opportunity = Some(&opportunity);
        ctx.flashloan_manager = Some(&manager);

        FlashloanDecorator
            .apply(&mut bundle, &mut ctx)
            .await
            .expect("flashloan decorator");

        ctx.variant = None;
        ctx.opportunity = None;
        ctx.flashloan_manager = None;

        assert!(ctx.flashloan_metadata.is_some());
        let metadata = ctx.flashloan_metadata.clone().unwrap();
        assert_eq!(metadata.protocol, FlashloanProtocol::Marginfi);
        assert_eq!(metadata.borrow_amount, opportunity.amount_in);
        let flattened = bundle.flatten();
        assert!(
            flattened.len() >= baseline + 4,
            "expected flashloan instructions injected"
        );
        assert_eq!(bundle.compute_budget.len(), 2);
        assert_eq!(bundle.lookup_addresses, expected_lookups);
        assert_eq!(ctx.compute_unit_limit, 200_000 + cfg.compute_unit_overhead);
    }
}

pub(super) fn attach_lighthouse_internal<'a>(
    ctx: &mut AssemblyContext<'a>,
    value: &'a mut LighthouseRuntime,
) {
    ctx.lighthouse = Some(value);
}

/// 指令装饰器：在流水线中增删改指令。
#[async_trait]
pub trait InstructionDecorator: Send + Sync {
    async fn apply(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()>;
}

/// 装饰器链：按照顺序执行全部装饰器。
struct DecoratorEntry {
    name: &'static str,
    handler: Arc<dyn InstructionDecorator>,
}

#[derive(Default)]
pub struct DecoratorChain {
    decorators: Vec<DecoratorEntry>,
}

impl DecoratorChain {
    pub fn new() -> Self {
        Self {
            decorators: Vec::new(),
        }
    }

    pub fn register<D>(&mut self, decorator: D)
    where
        D: InstructionDecorator + 'static,
    {
        let name = std::any::type_name::<D>();
        let short_name = name.rsplit("::").next().unwrap_or(name);
        self.decorators.push(DecoratorEntry {
            name: short_name,
            handler: Arc::new(decorator),
        });
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn apply_all(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        events::assembly_pipeline_started(context.base_mint, self.decorators.len());

        for entry in &self.decorators {
            let before_limit = context.compute_unit_limit;
            let before_guard = context.guard_required;
            let before_price = context.compute_unit_price;

            match entry.handler.apply(bundle, context).await {
                Ok(()) => {
                    events::assembly_decorator_applied(
                        entry.name,
                        context.base_mint,
                        before_limit,
                        context.compute_unit_limit,
                        before_guard,
                        context.guard_required,
                        before_price,
                        context.compute_unit_price,
                    );
                }
                Err(err) => {
                    let message = err.to_string();
                    events::assembly_decorator_failed(entry.name, context.base_mint, &message);
                    return Err(err);
                }
            }
        }

        events::assembly_pipeline_completed(
            context.base_mint,
            self.decorators.len(),
            bundle.compute_budget.len(),
            bundle.pre.len(),
            bundle.main.len(),
            bundle.post.len(),
            context.compute_unit_limit,
            context.compute_unit_price,
            context.guard_required,
            context.flashloan_metadata.is_some(),
        );

        Ok(())
    }
}
