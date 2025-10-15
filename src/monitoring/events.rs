use std::time::Duration;

use tracing::{info, warn};

use crate::engine::{QuoteTask, SwapOpportunity};
use crate::lander::{LanderError, LanderReceipt};
use solana_sdk::pubkey::Pubkey;

use super::metrics::prometheus_enabled;
use metrics::{counter, histogram};

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
            1,
            "strategy" => strategy.to_string(),
            "result" => result
        );
        histogram!(
            "galileo_quote_latency_ms",
            latency_ms,
            "strategy" => strategy.to_string(),
            "result" => result
        );
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
            1,
            "strategy" => strategy.to_string()
        );
        histogram!(
            "galileo_opportunity_profit_lamports",
            opportunity.profit_lamports as f64,
            "strategy" => strategy.to_string()
        );
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
            compute_unit_limit as f64,
            "strategy" => strategy.to_string()
        );
        histogram!(
            "galileo_swap_prioritization_fee_lamports",
            prioritization_fee as f64,
            "strategy" => strategy.to_string()
        );
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
            1,
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        );
        histogram!(
            "galileo_flashloan_amount_lamports",
            borrow_amount as f64,
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        );
        histogram!(
            "galileo_flashloan_inner_instruction_count",
            inner_instruction_count as f64,
            "strategy" => strategy.to_string(),
            "protocol" => protocol.to_string(),
            "mint" => mint.to_string()
        );
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
            1,
            "strategy" => strategy.to_string()
        );
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
                1,
            "strategy" => strategy.to_string(),
            "lander" => name.to_string()
        );
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
                1,
            "strategy" => strategy.to_string(),
            "lander" => receipt.lander
        );
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
                1,
            "strategy" => strategy.to_string(),
            "lander" => name.to_string()
        );
    }
}
