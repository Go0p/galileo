use std::time::Duration;

use tracing::{info, warn};

use crate::engine::{QuoteTask, SwapOpportunity};
use crate::lander::{LanderError, LanderReceipt};
use crate::titan::{TitanLeg, TitanQuoteSignal};
use solana_sdk::pubkey::Pubkey;

use super::metrics::prometheus_enabled;
use metrics::{counter, histogram};

pub fn accounts_precheck(
    strategy: &str,
    total_mints: usize,
    created_accounts: usize,
    skipped_mints: usize,
) {
    info!(
        target: "monitoring::accounts",
        event = "precheck",
        strategy,
        total_mints,
        created_accounts,
        skipped_mints,
        "token account precheck finished"
    );

    if prometheus_enabled() {
        let strategy_label = strategy.to_string();
        counter!(
            "galileo_accounts_precheck_total",
            "strategy" => strategy_label.clone()
        )
        .increment(1);
        histogram!(
            "galileo_accounts_precheck_mints",
            "strategy" => strategy_label.clone()
        )
        .record(total_mints as f64);
        histogram!(
            "galileo_accounts_precheck_created",
            "strategy" => strategy_label.clone()
        )
        .record(created_accounts as f64);
        histogram!(
            "galileo_accounts_precheck_skipped",
            "strategy" => strategy_label
        )
        .record(skipped_mints as f64);
    }
}

pub fn flashloan_account_precheck(strategy: &str, account: &Pubkey, created: bool) {
    info!(
        target: "monitoring::accounts",
        event = "flashloan_precheck",
        strategy,
        account = %account,
        created,
        "flashloan account precheck finished"
    );

    if prometheus_enabled() {
        let created_label = if created { "created" } else { "exists" };
        counter!(
            "galileo_flashloan_precheck_total",
            "strategy" => strategy.to_string(),
            "result" => created_label.to_string()
        )
        .increment(1);
    }
}

pub fn titan_quote_signal(strategy: &str, signal: &TitanQuoteSignal) {
    let provider_count = signal.quotes.quotes.len();
    let (best_provider, best_out_amount, best_in_amount) = match signal.quotes.swap_mode {
        crate::titan::types::SwapMode::ExactIn => signal
            .quotes
            .quotes
            .iter()
            .max_by_key(|(_, route)| route.out_amount)
            .map(|(provider, route)| (provider.as_str(), route.out_amount, route.in_amount))
            .unwrap_or(("none", 0, 0)),
        crate::titan::types::SwapMode::ExactOut => signal
            .quotes
            .quotes
            .iter()
            .min_by_key(|(_, route)| route.in_amount)
            .map(|(provider, route)| (provider.as_str(), route.out_amount, route.in_amount))
            .unwrap_or(("none", 0, 0)),
    };

    info!(
        target: "monitoring::titan",
        event = "update",
        strategy,
        input_mint = %signal.base_pair.input_mint,
        output_mint = %signal.base_pair.output_mint,
        amount = signal.amount,
        seq = signal.seq,
        leg = match signal.leg {
            TitanLeg::Forward => "forward",
            TitanLeg::Reverse => "reverse",
        },
        provider_count,
        best_provider,
        best_out_amount,
        best_in_amount,
        swap_mode = match signal.quotes.swap_mode {
            crate::titan::types::SwapMode::ExactIn => "exact_in",
            crate::titan::types::SwapMode::ExactOut => "exact_out",
        },
        "Titan quote stream update"
    );

    if prometheus_enabled() {
        let pair_label = format!(
            "{}-{}",
            signal.base_pair.input_mint, signal.base_pair.output_mint
        );
        let mode_label = match signal.quotes.swap_mode {
            crate::titan::types::SwapMode::ExactIn => "exact_in",
            crate::titan::types::SwapMode::ExactOut => "exact_out",
        };
        let leg_label = match signal.leg {
            TitanLeg::Forward => "forward",
            TitanLeg::Reverse => "reverse",
        };
        counter!(
            "galileo_titan_quote_signal_total",
            "strategy" => strategy.to_string(),
            "pair" => pair_label.clone(),
            "mode" => mode_label.to_string(),
            "leg" => leg_label.to_string(),
            "best_provider" => best_provider.to_string()
        )
        .increment(1);
        histogram!(
            "galileo_titan_provider_count",
            "strategy" => strategy.to_string(),
            "pair" => pair_label.clone(),
            "mode" => mode_label.to_string(),
            "leg" => leg_label.to_string()
        )
        .record(provider_count as f64);
        histogram!(
            "galileo_titan_best_out_amount",
            "strategy" => strategy.to_string(),
            "pair" => pair_label.clone(),
            "mode" => mode_label.to_string(),
            "leg" => leg_label.to_string()
        )
        .record(best_out_amount as f64);
        histogram!(
            "galileo_titan_best_in_amount",
            "strategy" => strategy.to_string(),
            "pair" => pair_label,
            "mode" => mode_label.to_string(),
            "leg" => leg_label.to_string()
        )
        .record(best_in_amount as f64);
    }
}

pub fn quote_start(strategy: &str, task: &QuoteTask) {
    info!(
        target: "monitoring::quote",
        event = "start",
        strategy,
        input_mint = %task.pair.input_mint,
        output_mint = %task.pair.output_mint,
        amount = task.amount,
        "quote started"
    );
}

pub fn quote_end(strategy: &str, task: &QuoteTask, success: bool, elapsed: Duration) {
    let latency_ms = elapsed.as_secs_f64() * 1_000.0;

    if success {
        info!(
            target: "monitoring::quote",
            event = "end",
            status = "success",
            strategy,
            input_mint = %task.pair.input_mint,
            output_mint = %task.pair.output_mint,
            amount = task.amount,
            latency_ms,
            "quote completed"
        );
    } else {
        warn!(
            target: "monitoring::quote",
            event = "end",
            status = "empty",
            strategy,
            input_mint = %task.pair.input_mint,
            output_mint = %task.pair.output_mint,
            amount = task.amount,
            latency_ms,
            "quote returned no result"
        );
    }

    if prometheus_enabled() {
        let result = if success { "success" } else { "empty" };
        counter!(
            "galileo_quote_total",
            "strategy" => strategy.to_string(),
            "result" => result
        )
        .increment(1);
        histogram!(
            "galileo_quote_latency_ms",
            "strategy" => strategy.to_string(),
            "result" => result
        )
        .record(latency_ms);
    }
}

pub fn profit_detected(strategy: &str, opportunity: &SwapOpportunity) {
    info!(
        target: "monitoring::profit",
        event = "detected",
        strategy,
        input_mint = %opportunity.pair.input_mint,
        output_mint = %opportunity.pair.output_mint,
        amount_in = opportunity.amount_in,
        profit_lamports = opportunity.profit_lamports,
        tip_lamports = opportunity.tip_lamports,
        net_profit = opportunity.net_profit(),
        "profitable opportunity"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_opportunity_detected_total",
            "strategy" => strategy.to_string()
        )
        .increment(1);
        histogram!(
            "galileo_opportunity_profit_lamports",
            "strategy" => strategy.to_string()
        )
        .record(opportunity.profit_lamports as f64);
    }
}

pub fn swap_fetched(
    strategy: &str,
    opportunity: &SwapOpportunity,
    compute_unit_limit: u32,
    prioritization_fee: u64,
) {
    info!(
        target: "monitoring::swap",
        event = "fetched",
        strategy,
        input_mint = %opportunity.pair.input_mint,
        output_mint = %opportunity.pair.output_mint,
        amount_in = opportunity.amount_in,
        compute_unit_limit,
        prioritization_fee,
        "swap instructions ready"
    );

    if prometheus_enabled() {
        histogram!(
            "galileo_swap_compute_unit_limit",
            "strategy" => strategy.to_string()
        )
        .record(compute_unit_limit as f64);
        histogram!(
            "galileo_swap_prioritization_fee_lamports",
            "strategy" => strategy.to_string()
        )
        .record(prioritization_fee as f64);
    }
}

pub fn flashloan_applied(
    strategy: &str,
    protocol: &str,
    mint: &Pubkey,
    borrow_amount: u64,
    inner_instruction_count: usize,
) {
    info!(
        target: "monitoring::flashloan",
        event = "applied",
        strategy,
        protocol,
        mint = %mint,
        borrow_amount,
        inner_instruction_count,
        "flashloan applied"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_flashloan_applied_total",
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        )
        .increment(1);
        histogram!(
            "galileo_flashloan_amount_lamports",
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        )
        .record(borrow_amount as f64);
        histogram!(
            "galileo_flashloan_inner_instruction_count",
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        )
        .record(inner_instruction_count as f64);
    }
}

pub fn transaction_built(
    strategy: &str,
    opportunity: &SwapOpportunity,
    slot: u64,
    blockhash: &str,
) {
    info!(
        target: "monitoring::transaction",
        event = "built",
        strategy,
        input_mint = %opportunity.pair.input_mint,
        output_mint = %opportunity.pair.output_mint,
        amount_in = opportunity.amount_in,
        slot,
        blockhash,
        "transaction prepared"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_transaction_built_total",
            "strategy" => strategy.to_string()
        )
        .increment(1);
    }
}

pub fn lander_attempt(strategy: &str, name: &str, attempt: usize) {
    info!(
        target: "monitoring::lander",
        event = "attempt",
        strategy,
        lander = name,
        attempt,
        "lander submission attempt"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_lander_attempt_total",
            "strategy" => strategy.to_string(),
            "lander" => name.to_string()
        )
        .increment(1);
    }
}

pub fn lander_success(strategy: &str, attempt: usize, receipt: &LanderReceipt) {
    info!(
        target: "monitoring::lander",
        event = "success",
        strategy,
        lander = receipt.lander,
        attempt,
        endpoint = %receipt.endpoint,
        slot = receipt.slot,
        blockhash = %receipt.blockhash,
        signature = receipt.signature.as_deref().unwrap_or_default(),
        "lander submission succeeded"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_lander_success_total",
            "strategy" => strategy.to_string(),
            "lander" => receipt.lander
        )
        .increment(1);
    }
}

pub fn lander_failure(strategy: &str, name: &str, attempt: usize, _err: &LanderError) {
    warn!(
        target: "monitoring::lander",
        event = "failure",
        strategy,
        lander = name,
        attempt,
        "lander submission failed"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_lander_failure_total",
            "strategy" => strategy.to_string(),
            "lander" => name.to_string()
        )
        .increment(1);
    }
}
