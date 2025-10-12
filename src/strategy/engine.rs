use std::convert::TryFrom;

use serde_json::Value;
use tracing::{debug, info, warn};

use crate::config::{BotConfig, BotIdentityConfig, JupiterRequestParamsConfig};
use crate::jupiter::{JupiterApiClient, QuoteRequest, QuoteResponse, SwapRequest};

use super::config::StrategyConfig;
use super::error::{StrategyError, StrategyResult};
use super::tip::TipCalculator;
use super::types::{ArbitrageOpportunity, TradePair};

pub struct ArbitrageEngine {
    config: StrategyConfig,
    _bot: BotConfig,
    client: JupiterApiClient,
    tip_calculator: TipCalculator,
    pairs: Vec<TradePair>,
    trade_amounts: Vec<u64>,
    identity: BotIdentityConfig,
    request_params: JupiterRequestParamsConfig,
}

impl ArbitrageEngine {
    pub fn new(
        mut config: StrategyConfig,
        bot: BotConfig,
        client: JupiterApiClient,
        request_params: JupiterRequestParamsConfig,
    ) -> StrategyResult<Self> {
        if !config.is_enabled() {
            return Err(StrategyError::Disabled);
        }

        let mut pairs = config.resolved_pairs();
        if pairs.is_empty() {
            return Err(StrategyError::InvalidConfig(
                "trade pairs or quote mints missing".into(),
            ));
        }

        if config.controls.enable_reverse_trade {
            let mut reversed: Vec<TradePair> = pairs.iter().map(|p| p.reversed()).collect();
            pairs.extend(reversed.drain(..));
        }

        let trade_amounts = config.effective_trade_amounts();
        if trade_amounts.is_empty() {
            return Err(StrategyError::InvalidConfig(
                "trade_range produced no amounts".into(),
            ));
        }

        if config.quote_max_accounts.is_none() {
            config.quote_max_accounts = request_params.max_accounts;
        }
        if request_params.use_direct_route_only {
            config.only_direct_routes = true;
        }
        if !request_params.restrict_intermediate_tokens {
            config.restrict_intermediate_tokens = false;
        }
        if config.controls.only_quote_dexs.is_empty()
            && !request_params.included_dex_program_ids.is_empty()
        {
            config.controls.only_quote_dexs =
                request_params.included_dex_program_ids.clone();
        }

        let identity = bot.identity.clone();
        if identity.user_pubkey.is_none() {
            return Err(StrategyError::InvalidConfig(
                "bot.identity.user_pubkey".into(),
            ));
        }

        let tip_calculator =
            TipCalculator::new(&config.controls.static_tip_config, config.max_tip_lamports);

        Ok(Self {
            config,
            _bot: bot,
            client,
            tip_calculator,
            pairs,
            trade_amounts,
            identity,
            request_params,
        })
    }

    pub async fn run(&self) -> StrategyResult<()> {
        let delay = self.config.trade_delay();
        info!(
            target: "strategy",
            pair_count = self.pairs.len(),
            amount_count = self.trade_amounts.len(),
            delay_ms = delay.as_millis() as u64,
            "arbitrage engine started"
        );

        loop {
            for pair in &self.pairs {
                for &amount in &self.trade_amounts {
                    match self.evaluate_pair(pair, amount).await {
                        Ok(Some(opportunity)) => {
                            if let Err(err) = self.execute_opportunity(&opportunity).await {
                                warn!(
                                    target: "strategy",
                                    error = %err,
                                    input_mint = %pair.input_mint,
                                    output_mint = %pair.output_mint,
                                    amount,
                                    "failed to execute opportunity"
                                );
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(
                                target: "strategy",
                                error = %err,
                                input_mint = %pair.input_mint,
                                output_mint = %pair.output_mint,
                                amount,
                                "evaluation failed"
                            );
                        }
                    }
                }
            }

            tokio::time::sleep(delay).await;
        }
    }

    async fn evaluate_pair(
        &self,
        pair: &TradePair,
        amount: u64,
    ) -> StrategyResult<Option<ArbitrageOpportunity>> {
        let first_request = self.build_quote_request(pair, amount);
        let quote_first = self.client.quote(&first_request).await?;
        let first_out = parse_amount(&quote_first.out_amount)?;
        if first_out == 0 {
            return Ok(None);
        }

        let second_amount = match u64::try_from(first_out) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let reverse_pair = pair.reversed();
        let second_request = self.build_quote_request(&reverse_pair, second_amount);
        let quote_second = self.client.quote(&second_request).await?;
        let second_out = parse_amount(&quote_second.out_amount)?;

        let profit = second_out.saturating_sub(amount as u128);
        let profit_u64 = profit.min(u128::from(u64::MAX)) as u64;
        if profit_u64 <= self.config.min_profit_threshold_lamports {
            debug!(
                target: "strategy",
                input_mint = %pair.input_mint,
                output_mint = %pair.output_mint,
                amount,
                profit = profit_u64,
                "profit below threshold"
            );
            return Ok(None);
        }

        let tip_lamports = self.tip_calculator.calculate(profit_u64);
        let merged_quote = self.merge_quotes(&quote_first, &quote_second, amount, tip_lamports)?;

        let opportunity = ArbitrageOpportunity {
            pair: pair.clone(),
            amount_in: amount,
            first_leg_out: first_out.min(u128::from(u64::MAX)) as u64,
            second_leg_out: second_out.min(u128::from(u64::MAX)) as u64,
            profit_lamports: profit_u64,
            tip_lamports,
            merged_quote,
        };

        Ok(Some(opportunity))
    }

    fn build_quote_request(&self, pair: &TradePair, amount: u64) -> QuoteRequest {
        let mut request = QuoteRequest::new(
            pair.input_mint.clone(),
            pair.output_mint.clone(),
            amount,
            self.config.slippage_bps,
        );
        request.only_direct_routes = self.config.only_direct_routes;
        request.restrict_intermediate_tokens = self.config.restrict_intermediate_tokens;
        if let Some(max_accounts) = self.config.quote_max_accounts {
            request
                .extra
                .insert("maxAccounts".to_string(), max_accounts.to_string());
        }
        if !self.config.controls.only_quote_dexs.is_empty() {
            request.extra.insert(
                "onlyDexes".to_string(),
                self.config.controls.only_quote_dexs.join(","),
            );
        }
        self.apply_request_defaults(&mut request);
        request
    }

    fn apply_request_defaults(&self, request: &mut QuoteRequest) {
        if request.extra.get("maxAccounts").is_none() {
            if let Some(max_accounts) = self.request_params.max_accounts {
                request
                    .extra
                    .insert("maxAccounts".to_string(), max_accounts.to_string());
            }
        }

        if request.extra.get("onlyDexes").is_none()
            && !self.request_params.included_dex_program_ids.is_empty()
        {
            request.extra.insert(
                "onlyDexes".to_string(),
                self.request_params
                    .included_dex_program_ids
                    .join(","),
            );
        }

        if request.extra.get("excludeDexes").is_none()
            && !self.request_params.excluded_dex_program_ids.is_empty()
        {
            request.extra.insert(
                "excludeDexes".to_string(),
                self.request_params
                    .excluded_dex_program_ids
                    .join(","),
            );
        }

        if !request.only_direct_routes && self.request_params.use_direct_route_only {
            request.only_direct_routes = true;
        }

        if request.restrict_intermediate_tokens && !self.request_params.restrict_intermediate_tokens
        {
            request.restrict_intermediate_tokens = false;
        }
    }

    fn merge_quotes(
        &self,
        first: &QuoteResponse,
        second: &QuoteResponse,
        original_amount: u64,
        tip_lamports: u64,
    ) -> StrategyResult<Value> {
        let mut merged = first.raw.clone();
        let total_out = (original_amount as u128)
            .saturating_add(tip_lamports as u128)
            .min(u128::from(u64::MAX)) as u64;
        if let Some(obj) = merged.as_object_mut() {
            obj.insert(
                "outputMint".to_string(),
                Value::String(second.output_mint.clone()),
            );
            obj.insert("priceImpactPct".to_string(), Value::String("0".into()));
            obj.insert(
                "outAmount".to_string(),
                Value::String(total_out.to_string()),
            );
            obj.insert(
                "otherAmountThreshold".to_string(),
                Value::String(total_out.to_string()),
            );
            if let Some(route_plan) = obj.get_mut("routePlan") {
                if let Some(route_array) = route_plan.as_array_mut() {
                    if let Some(second_plan) =
                        second.raw.get("routePlan").and_then(|v| v.as_array())
                    {
                        route_array.extend(second_plan.iter().cloned());
                    }
                }
            }
        }
        Ok(merged)
    }

    async fn execute_opportunity(&self, opportunity: &ArbitrageOpportunity) -> StrategyResult<()> {
        info!(
            target: "strategy::opportunity",
            input_mint = %opportunity.pair.input_mint,
            output_mint = %opportunity.pair.output_mint,
            amount_in = opportunity.amount_in,
            profit_lamports = opportunity.profit_lamports,
            tip_lamports = opportunity.tip_lamports,
            net_profit = opportunity.net_profit(),
            "profitable opportunity detected"
        );

        let mut swap_request = SwapRequest::new(
            opportunity.merged_quote.clone(),
            self.identity.user_pubkey.clone().unwrap(),
        );
        swap_request.wrap_and_unwrap_sol = Some(self.identity.wrap_and_unwrap_sol);
        swap_request.use_shared_accounts = Some(self.identity.use_shared_accounts);
        swap_request.fee_account = self.identity.fee_account.clone();
        swap_request.compute_unit_price_micro_lamports =
            self.identity.compute_unit_price_micro_lamports;
        swap_request.skip_user_accounts_rpc_calls =
            self.identity.skip_user_accounts_rpc_calls.or(Some(true));
        let instructions = self.client.swap_instructions(&swap_request).await?;
        info!(
            target: "strategy::execution",
            compute_unit_limit = ?instructions.compute_unit_limit,
            prioritization_fee_lamports = ?instructions.prioritization_fee_lamports,
            setup_ix = instructions.setup_instructions.len(),
            other_ix = instructions.other_instructions.len(),
            "swap instructions ready"
        );

        // TODO: assemble transaction using instructions.* and submit via configured lander/Jito.
        Ok(())
    }
}

fn parse_amount(value: &str) -> Result<u128, std::num::ParseIntError> {
    value.parse::<u128>()
}
