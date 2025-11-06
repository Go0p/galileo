use std::fmt;

use async_trait::async_trait;

use crate::engine::assembly::decorators::{
    ComputeBudgetDecorator, FlashloanDecorator, GuardBudgetDecorator, GuardStrategy,
    ProfitGuardDecorator, TipDecorator,
};
use crate::engine::assembly::{
    AssemblyContext, DecoratorChain, InstructionBundle, attach_lighthouse,
};
use crate::engine::builder::PreparedTransaction;
use crate::engine::landing::execution_plan::ExecutionPlan;
use crate::engine::landing::profile::{GuardBudgetKind, LanderKind, LandingProfile, TipStrategy};
use crate::engine::plugins::flashloan::{FlashloanMetadata, MarginfiFlashloanManager};
use crate::engine::{EngineError, LighthouseRuntime, TransactionBuilder};

#[derive(Debug)]
pub enum LandingAssemblyError {
    Engine(EngineError),
}

impl fmt::Display for LandingAssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LandingAssemblyError::Engine(err) => write!(f, "{err}"),
        }
    }
}

impl From<EngineError> for LandingAssemblyError {
    fn from(value: EngineError) -> Self {
        LandingAssemblyError::Engine(value)
    }
}

pub type LandingAssemblyResult<T> = Result<T, LandingAssemblyError>;

#[derive(Debug, Clone, Copy)]
pub struct TipComputation {
    pub kind: TipComputationKind,
    pub lamports: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum TipComputationKind {
    Opportunity,
    JitoPlan,
}

#[derive(Debug, Clone, Copy)]
pub struct GuardComputation {
    pub kind: GuardStrategy,
    pub lamports: u64,
}

#[derive(Debug)]
pub struct LandingPlanEntry {
    pub profile: LandingProfile,
    pub prepared: PreparedTransaction,
    pub tip: TipComputation,
    pub guard: GuardComputation,
    pub prioritization_fee_lamports: u64,
    pub flashloan_metadata: Option<FlashloanMetadata>,
}

pub struct LandingAssemblyContext<'a> {
    pub identity: &'a crate::engine::EngineIdentity,
    pub tx_builder: &'a TransactionBuilder,
    pub flashloan: Option<&'a MarginfiFlashloanManager>,
    pub lighthouse: &'a mut LighthouseRuntime,
}

impl<'a> LandingAssemblyContext<'a> {
    pub fn new(
        identity: &'a crate::engine::EngineIdentity,
        tx_builder: &'a TransactionBuilder,
        flashloan: Option<&'a MarginfiFlashloanManager>,
        lighthouse: &'a mut LighthouseRuntime,
    ) -> Self {
        Self {
            identity,
            tx_builder,
            flashloan,
            lighthouse,
        }
    }
}

#[async_trait]
pub trait LandingAssembler: Send + Sync {
    async fn assemble_landing(
        &self,
        ctx: &mut LandingAssemblyContext<'_>,
        profile: &LandingProfile,
        plan: &ExecutionPlan,
    ) -> LandingAssemblyResult<LandingPlanEntry>;
}

pub struct DefaultLandingAssembler;

impl DefaultLandingAssembler {
    pub fn new() -> Self {
        Self
    }

    fn compute_prioritization_fee(limit: u32, price: Option<u64>) -> u64 {
        match price {
            Some(0) | None => 0,
            Some(value) => {
                let fee = (value as u128)
                    .saturating_mul(limit as u128)
                    .checked_div(1_000_000u128)
                    .unwrap_or(0);
                fee.min(u64::MAX as u128) as u64
            }
        }
    }
}

#[async_trait]
impl LandingAssembler for DefaultLandingAssembler {
    async fn assemble_landing(
        &self,
        ctx: &mut LandingAssemblyContext<'_>,
        profile: &LandingProfile,
        plan: &ExecutionPlan,
    ) -> LandingAssemblyResult<LandingPlanEntry> {
        let mut variant = plan.swap_variant.clone();

        let mut bundle = InstructionBundle::from_instructions(variant.flatten_instructions());
        bundle.set_lookup_tables(
            variant.address_lookup_table_addresses().to_vec(),
            variant.resolved_lookup_tables().to_vec(),
        );

        let (tip_plan, tip, extra_guard_lamports) = match &profile.tip_strategy {
            TipStrategy::UseOpportunity => {
                let lamports = plan.base_tip_lamports;
                (
                    None,
                    TipComputation {
                        kind: TipComputationKind::Opportunity,
                        lamports,
                    },
                    0,
                )
            }
            TipStrategy::Jito {
                plan: Some(plan),
                extra_guard_lamports,
                ..
            } => {
                let lamports = plan.lamports;
                (
                    Some(plan.clone()),
                    TipComputation {
                        kind: TipComputationKind::JitoPlan,
                        lamports,
                    },
                    *extra_guard_lamports,
                )
            }
            TipStrategy::Jito {
                plan: None,
                extra_guard_lamports,
                ..
            } => (
                None,
                TipComputation {
                    kind: TipComputationKind::JitoPlan,
                    lamports: 0,
                },
                *extra_guard_lamports,
            ),
        };

        let compute_unit_price = profile.compute_unit_strategy.value();
        let prioritization_fee =
            Self::compute_prioritization_fee(plan.compute_unit_limit, compute_unit_price);

        let guard_strategy = match profile.guard_budget {
            GuardBudgetKind::BasePlusTip => GuardStrategy::BasePlusTip,
            GuardBudgetKind::BasePlusPrioritizationFee => GuardStrategy::BasePlusPrioritizationFee,
        };

        let mut assembly_ctx = AssemblyContext::new(ctx.identity);
        assembly_ctx.base_mint = Some(&plan.base_mint);
        assembly_ctx.compute_unit_limit = plan.compute_unit_limit;
        assembly_ctx.compute_unit_price = compute_unit_price;
        assembly_ctx.guard_required = plan.base_guard_lamports;
        assembly_ctx.guard_strategy = guard_strategy;
        assembly_ctx.prioritization_fee = prioritization_fee;
        assembly_ctx.tip_lamports = tip.lamports;
        assembly_ctx.jito_tip_budget = if matches!(profile.lander_kind, LanderKind::Jito) {
            tip.lamports.saturating_add(extra_guard_lamports)
        } else {
            0
        };
        assembly_ctx.jito_tip_plan = tip_plan.clone();
        assembly_ctx.variant = Some(&mut variant);
        assembly_ctx.opportunity = Some(&plan.opportunity);
        assembly_ctx.flashloan_manager = ctx.flashloan;

        attach_lighthouse(&mut assembly_ctx, ctx.lighthouse);

        let mut decorators = DecoratorChain::new();
        decorators.register(FlashloanDecorator);
        decorators.register(ComputeBudgetDecorator);
        decorators.register(TipDecorator);
        decorators.register(GuardBudgetDecorator);
        decorators.register(ProfitGuardDecorator);

        decorators
            .apply_all(&mut bundle, &mut assembly_ctx)
            .await
            .map_err(LandingAssemblyError::Engine)?;

        let guard_lamports = assembly_ctx.guard_required;
        let flashloan_metadata = assembly_ctx.flashloan_metadata.clone();

        drop(assembly_ctx);

        let final_instructions = bundle.into_flattened();

        let mut prepared = ctx
            .tx_builder
            .build_with_sequence(
                ctx.identity,
                &variant,
                final_instructions,
                tip.lamports,
                tip_plan.clone(),
            )
            .await
            .map_err(LandingAssemblyError::Engine)?;

        let tip_label = profile.tip_strategy.label();

        prepared.prioritization_fee_lamports = prioritization_fee;
        prepared.guard_lamports = guard_lamports;
        prepared.guard_strategy = guard_strategy;
        prepared.compute_unit_price_micro_lamports = compute_unit_price;
        prepared.tip_strategy_label = tip_label;
        prepared.compute_unit_price_strategy_label = profile.compute_unit_strategy.label();

        let guard = GuardComputation {
            kind: guard_strategy,
            lamports: guard_lamports,
        };

        Ok(LandingPlanEntry {
            profile: profile.clone(),
            prepared,
            tip,
            guard,
            prioritization_fee_lamports: prioritization_fee,
            flashloan_metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::AltCache;
    use crate::engine::aggregator::{MultiLegInstructions, SwapInstructionsVariant};
    use crate::engine::builder::TransactionBuilder;
    use crate::engine::identity::EngineIdentity;
    use crate::engine::landing::profile::{
        ComputeUnitPriceStrategy, GuardBudgetKind, LanderKind, LandingProfile, TipStrategy,
    };
    use crate::engine::types::SwapOpportunity;
    use crate::engine::{BuilderConfig, JitoTipPlan, LighthouseSettings};
    use crate::network::{CooldownConfig, IpAllocator, IpInventory};
    use crate::strategy::types::TradePair;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::instruction::Instruction;
    use solana_sdk::message::AddressLookupTableAccount;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Keypair;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

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
            data: vec![0u8; 4],
        }
    }

    fn build_allocator() -> Arc<IpAllocator> {
        let inventory = IpInventory::builder()
            .allow_loopback(true)
            .manual_ips([IpAddr::V4(Ipv4Addr::LOCALHOST)])
            .build()
            .expect("ip inventory");
        Arc::new(IpAllocator::from_inventory(
            inventory,
            None,
            CooldownConfig::default(),
        ))
    }

    fn build_transaction_builder() -> TransactionBuilder {
        let rpc = Arc::new(RpcClient::new_mock("http://localhost:8899".to_string()));
        let config = BuilderConfig::new(None);
        TransactionBuilder::new(rpc, config, build_allocator(), None, AltCache::new(), false)
    }

    fn sample_execution_plan(
        base_tip: u64,
        compute_unit_limit: u32,
        base_guard: u64,
        prioritization_fee: u64,
    ) -> ExecutionPlan {
        let lookup_address = Pubkey::new_unique();
        let lookup_entry = AddressLookupTableAccount {
            key: lookup_address,
            addresses: vec![Pubkey::new_unique()],
        };
        let compute_limit_ix =
            crate::instructions::compute_budget::compute_unit_limit_instruction(compute_unit_limit);
        let swap_instruction = dummy_instruction();
        let multi_leg = MultiLegInstructions::new(
            vec![compute_limit_ix],
            vec![swap_instruction],
            vec![lookup_address],
            vec![lookup_entry],
            Some(prioritization_fee),
            compute_unit_limit,
        );
        let variant = SwapInstructionsVariant::MultiLeg(multi_leg);
        let base_mint = Pubkey::new_unique();
        let pair = TradePair::from_pubkeys(base_mint, Pubkey::new_unique());
        let opportunity = SwapOpportunity {
            pair,
            amount_in: 1,
            profit_lamports: 0,
            tip_lamports: base_tip,
            merged_quote: None,
            ultra_legs: None,
        };
        ExecutionPlan::new(
            opportunity,
            variant,
            base_mint,
            base_tip,
            base_guard,
            compute_unit_limit,
            prioritization_fee,
            Instant::now() + Duration::from_secs(60),
        )
    }

    #[tokio::test(flavor = "current_thread")]
    async fn assembler_uses_tip_for_jito_guard() {
        let identity = dummy_identity();
        let plan = sample_execution_plan(15, 100_000, 5_000, 0);
        let profile = LandingProfile::new(
            LanderKind::Jito,
            TipStrategy::Jito {
                plan: Some(JitoTipPlan::new(15, Pubkey::new_unique())),
                label: "stream",
                extra_guard_lamports: 0,
            },
            GuardBudgetKind::BasePlusTip,
            ComputeUnitPriceStrategy::Fixed(0),
        );

        let mut lighthouse = LighthouseRuntime::new(&LighthouseSettings::default(), 1);
        let builder = build_transaction_builder();
        let mut context = LandingAssemblyContext::new(&identity, &builder, None, &mut lighthouse);

        let assembler = DefaultLandingAssembler::new();
        let entry = assembler
            .assemble_landing(&mut context, &profile, &plan)
            .await
            .expect("assembled jito plan");

        assert_eq!(entry.prepared.tip_lamports, 15);
        assert_eq!(entry.prepared.guard_lamports, 5_015);
        assert_eq!(entry.prepared.compute_unit_price_micro_lamports, Some(0));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn assembler_uses_prioritization_fee_for_rpc_guard() {
        let identity = dummy_identity();
        let plan = sample_execution_plan(7, 200_000, 5_000, 0);
        let profile = LandingProfile::new(
            LanderKind::Staked,
            TipStrategy::UseOpportunity,
            GuardBudgetKind::BasePlusPrioritizationFee,
            ComputeUnitPriceStrategy::Fixed(1_000),
        );

        let mut lighthouse = LighthouseRuntime::new(&LighthouseSettings::default(), 1);
        let builder = build_transaction_builder();
        let mut context = LandingAssemblyContext::new(&identity, &builder, None, &mut lighthouse);

        let assembler = DefaultLandingAssembler::new();
        let entry = assembler
            .assemble_landing(&mut context, &profile, &plan)
            .await
            .expect("assembled rpc plan");

        let expected_fee = (200_000u64 * 1_000) / 1_000_000;
        assert_eq!(entry.prepared.tip_lamports, 7);
        assert_eq!(entry.prepared.prioritization_fee_lamports, expected_fee);
        assert_eq!(entry.prepared.guard_lamports, 5_000 + expected_fee);
        assert_eq!(
            entry.prepared.compute_unit_price_micro_lamports,
            Some(1_000)
        );
    }
}
