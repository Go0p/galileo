use std::{convert::TryFrom, env, str::FromStr, sync::Arc};

use bs58;

use serde_json::Value;
use tracing::{debug, info, warn};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::api::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, QuoteResponse,
    SwapInstructionsRequest,
};
use crate::config::{
    BlindConfig, BotConfig, InstructionConfig, LanderSettings, RequestParamsConfig, SpamConfig,
    WalletConfig,
};

use super::config::StrategyConfig;
use super::error::{StrategyError, StrategyResult};
use super::execution::{ExecutionServices, StrategyMode};
use super::tip::TipCalculator;
use super::types::{ArbitrageOpportunity, TradePair};

pub struct ArbitrageEngine {
    mode: StrategyMode,
    config: StrategyConfig,
    client: JupiterApiClient,
    tip_calculator: TipCalculator,
    pairs: Vec<TradePair>,
    trade_amounts: Vec<u64>,
    identity: StrategyIdentity,
    request_params: RequestParamsConfig,
    execution: ExecutionServices,
}

impl ArbitrageEngine {
    pub fn new(
        mode: StrategyMode,
        mut config: StrategyConfig,
        bot: BotConfig,
        wallet: WalletConfig,
        client: JupiterApiClient,
        request_params: RequestParamsConfig,
        instruction: InstructionConfig,
        spam_config: SpamConfig,
        blind_config: BlindConfig,
        lander_settings: LanderSettings,
        rpc_client: Arc<RpcClient>,
        http_client: reqwest::Client,
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

        if request_params.only_direct_routes || matches!(mode, StrategyMode::Spam) {
            config.only_direct_routes = true;
        }
        if !request_params.restrict_intermediate_tokens && !matches!(mode, StrategyMode::Spam) {
            config.restrict_intermediate_tokens = false;
        }
        if config.controls.only_quote_dexs.is_empty() && !request_params.included_dexes.is_empty() {
            config.controls.only_quote_dexs = request_params.included_dexes.clone();
        }
        match mode {
            StrategyMode::Spam => {
                if !spam_config.enable_dexs.is_empty() {
                    config.controls.only_quote_dexs = spam_config.enable_dexs.clone();
                }
            }
            StrategyMode::Blind => {
                if !blind_config.enable_dexs.is_empty() {
                    config.controls.only_quote_dexs = blind_config.enable_dexs.clone();
                }
            }
        }

        let identity = StrategyIdentity::from_sources(&wallet)?;

        let tip_calculator =
            TipCalculator::new(&config.controls.static_tip_config, config.max_tip_lamports);

        let execution = ExecutionServices::new(
            rpc_client,
            http_client,
            &instruction,
            &lander_settings,
            &spam_config,
            &blind_config,
        )?
        .with_simulation(bot.enable_simulation);

        Ok(Self {
            mode,
            config,
            client,
            tip_calculator,
            pairs,
            trade_amounts,
            identity,
            request_params,
            execution,
        })
    }

    pub async fn run(&self) -> StrategyResult<()> {
        let delay = self.config.trade_delay();
        info!(
            target: "strategy",
            mode = ?self.mode,
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
        let first_request = self.build_quote_request(pair, amount)?;
        let quote_first = self.client.quote(&first_request).await?;
        let first_out = quote_first.out_amount as u128;
        if first_out == 0 {
            return Ok(None);
        }

        let second_amount = match u64::try_from(first_out) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let reverse_pair = pair.reversed();
        let second_request = self.build_quote_request(&reverse_pair, second_amount)?;
        let quote_second = self.client.quote(&second_request).await?;
        let second_out = quote_second.out_amount as u128;

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

    fn build_quote_request(&self, pair: &TradePair, amount: u64) -> StrategyResult<QuoteRequest> {
        let input_mint = Pubkey::from_str(&pair.input_mint).map_err(|err| {
            StrategyError::InvalidConfig(format!("invalid input mint {}: {err}", pair.input_mint))
        })?;
        let output_mint = Pubkey::from_str(&pair.output_mint).map_err(|err| {
            StrategyError::InvalidConfig(format!("invalid output mint {}: {err}", pair.output_mint))
        })?;
        let mut request =
            QuoteRequest::new(input_mint, output_mint, amount, self.config.slippage_bps);
        request.only_direct_routes = Some(self.config.only_direct_routes);
        request.restrict_intermediate_tokens = Some(self.config.restrict_intermediate_tokens);
        if let Some(max_accounts) = self.config.quote_max_accounts {
            request.max_accounts = Some(max_accounts as usize);
        }
        if !self.config.controls.only_quote_dexs.is_empty() {
            let dexes = self.config.controls.only_quote_dexs.join(",");
            request.dexes = Some(dexes.clone());
            request
                .extra_query_params
                .entry("onlyDexes".to_string())
                .or_insert(dexes);
        }
        self.apply_request_defaults(&mut request);
        Ok(request)
    }

    fn apply_request_defaults(&self, request: &mut QuoteRequest) {
        if request.dexes.is_none() && !self.request_params.included_dexes.is_empty() {
            let dexes = self.request_params.included_dexes.join(",");
            request.dexes = Some(dexes.clone());
            request
                .extra_query_params
                .entry("onlyDexes".to_string())
                .or_insert(dexes);
        }

        if request.excluded_dexes.is_none() && !self.request_params.excluded_dexes.is_empty() {
            let dexes = self.request_params.excluded_dexes.join(",");
            request.excluded_dexes = Some(dexes.clone());
            request
                .extra_query_params
                .entry("excludeDexes".to_string())
                .or_insert(dexes);
        }

        if !request.only_direct_routes.unwrap_or(false) && self.request_params.only_direct_routes {
            request.only_direct_routes = Some(true);
        }

        if request.restrict_intermediate_tokens.unwrap_or(true)
            && !self.request_params.restrict_intermediate_tokens
        {
            request.restrict_intermediate_tokens = Some(false);
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
                Value::String(second.output_mint.to_string()),
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

        let mut swap_request =
            SwapInstructionsRequest::new(opportunity.merged_quote.clone(), self.identity.pubkey);
        swap_request.config.wrap_and_unwrap_sol = self.identity.wrap_and_unwrap_sol;
        swap_request.config.use_shared_accounts = Some(self.identity.use_shared_accounts);
        swap_request.config.skip_user_accounts_rpc_calls =
            self.identity.skip_user_accounts_rpc_calls.unwrap_or(true);
        if let Some(ref fee) = self.identity.fee_account {
            match Pubkey::from_str(fee) {
                Ok(pubkey) => swap_request.config.fee_account = Some(pubkey),
                Err(err) => {
                    warn!(
                        target: "strategy::execution",
                        fee_account = %fee,
                        error = %err,
                        "invalid fee account configured, skipping"
                    );
                }
            }
        }
        if let Some(override_price) = self.execution.compute_unit_price_override(self.mode) {
            swap_request.config.compute_unit_price_micro_lamports =
                Some(ComputeUnitPriceMicroLamports::MicroLamports(override_price));
        } else if let Some(price) = self.identity.compute_unit_price_micro_lamports {
            swap_request.config.compute_unit_price_micro_lamports =
                Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
        }
        let instructions = self.client.swap_instructions(&swap_request).await?;
        info!(
            target: "strategy::execution",
            compute_unit_limit = ?instructions.compute_unit_limit,
            prioritization_fee_lamports = ?instructions.prioritization_fee_lamports,
            setup_ix = instructions.setup_instructions.len(),
            other_ix = instructions.other_instructions.len(),
            "swap instructions ready"
        );

        let prepared = self
            .execution
            .prepare_transaction(
                &self.identity,
                &instructions,
                opportunity.tip_lamports,
                self.mode,
            )
            .await?;
        let landers = self.execution.allowed_landers(self.mode);
        let attempts = match self.mode {
            StrategyMode::Spam => std::cmp::max(1, self.execution.spam_max_retries() as usize + 1),
            StrategyMode::Blind => 1,
        };
        let mut last_err = None;

        for attempt in 0..attempts {
            match self.execution.submit(prepared.clone(), &landers).await {
                Ok(bundle) => {
                    info!(
                        target: "strategy::bundle",
                        attempt,
                        lander = %bundle.lander,
                        endpoint = %bundle.endpoint,
                        slot = bundle.slot,
                        blockhash = %bundle.blockhash,
                        bundle_id = bundle.bundle_id.as_deref().unwrap_or_default(),
                        "bundle submission succeeded"
                    );
                    return Ok(());
                }
                Err(err) => {
                    warn!(
                        target: "strategy::bundle",
                        attempt,
                        error = %err,
                        "bundle submission failed"
                    );
                    last_err = Some(err);
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| StrategyError::Bundle("bundle submission exhausted retries".into())))
    }
}

#[derive(Clone, Debug)]
pub(super) struct StrategyIdentity {
    pub pubkey: Pubkey,
    fee_account: Option<String>,
    wrap_and_unwrap_sol: bool,
    use_shared_accounts: bool,
    compute_unit_price_micro_lamports: Option<u64>,
    skip_user_accounts_rpc_calls: Option<bool>,
    pub(super) signer: Arc<Keypair>,
}

impl StrategyIdentity {
    fn from_sources(wallet: &WalletConfig) -> StrategyResult<Self> {
        let signer = load_keypair(wallet)?;
        let pubkey = match env::var("GALILEO_USER_PUBKEY") {
            Ok(value) => Pubkey::from_str(value.trim()).map_err(|err| {
                StrategyError::InvalidConfig(format!("invalid GALILEO_USER_PUBKEY: {err}"))
            })?,
            Err(env::VarError::NotPresent) => signer.pubkey(),
            Err(err) => {
                return Err(StrategyError::InvalidConfig(format!(
                    "failed to read GALILEO_USER_PUBKEY: {err}"
                )));
            }
        };

        let fee_account = env::var("GALILEO_FEE_ACCOUNT")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let wrap_and_unwrap_sol = wallet.warp_or_unwrap_sol.wrap_and_unwrap_sol;

        let compute_unit_price_micro_lamports =
            match env::var("GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS") {
                Ok(value) => {
                    let parsed = value.trim().parse::<u64>().map_err(|err| {
                        StrategyError::InvalidConfig(format!(
                            "invalid GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS: {err}"
                        ))
                    })?;
                    Some(parsed)
                }
                Err(env::VarError::NotPresent) => {
                    let configured = wallet.warp_or_unwrap_sol.compute_unit_price_micro_lamports;
                    if configured > 0 {
                        Some(configured)
                    } else {
                        None
                    }
                }
                Err(err) => {
                    return Err(StrategyError::InvalidConfig(format!(
                        "failed to read GALILEO_COMPUTE_UNIT_PRICE_MICROLAMPORTS: {err}"
                    )));
                }
            };

        let use_shared_accounts = parse_env_bool("GALILEO_USE_SHARED_ACCOUNTS")?.unwrap_or(false);
        let skip_user_accounts_rpc_calls = parse_env_bool("GALILEO_SKIP_USER_ACCOUNTS_RPC_CALLS")?;

        Ok(Self {
            pubkey,
            fee_account,
            wrap_and_unwrap_sol,
            use_shared_accounts,
            compute_unit_price_micro_lamports,
            skip_user_accounts_rpc_calls,
            signer,
        })
    }
}

fn parse_env_bool(var: &str) -> StrategyResult<Option<bool>> {
    match env::var(var) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "1" | "yes" => Ok(Some(true)),
                "false" | "0" | "no" => Ok(Some(false)),
                _ => Err(StrategyError::InvalidConfig(format!(
                    "invalid boolean value for {var}: {value}"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(StrategyError::InvalidConfig(format!(
            "failed to read {var}: {err}"
        ))),
    }
}

fn load_keypair(wallet: &WalletConfig) -> StrategyResult<Arc<Keypair>> {
    if let Ok(value) = env::var("GALILEO_PRIVATE_KEY") {
        if !value.trim().is_empty() {
            let keypair = parse_keypair_string(value.trim()).map_err(|err| {
                StrategyError::InvalidConfig(format!("invalid GALILEO_PRIVATE_KEY: {err}"))
            })?;
            return Ok(Arc::new(keypair));
        }
    }

    if !wallet.private_key.trim().is_empty() {
        let keypair = parse_keypair_string(wallet.private_key.trim()).map_err(|err| {
            StrategyError::InvalidConfig(format!("invalid global.wallet.private_key: {err}"))
        })?;
        return Ok(Arc::new(keypair));
    }

    Err(StrategyError::InvalidConfig(
        "missing private key (configure global.wallet.private_key or GALILEO_PRIVATE_KEY)".into(),
    ))
}

fn parse_keypair_string(raw: &str) -> Result<Keypair, anyhow::Error> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("keypair string empty");
    }

    if trimmed.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(trimmed)?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else if trimmed.contains(',') {
        let bytes = trimmed
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else {
        let data = bs58::decode(trimmed).into_vec()?;
        Ok(Keypair::try_from(data.as_slice())?)
    }
}
