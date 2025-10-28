use std::net::IpAddr;
use std::time::Duration;

use tracing::{info, trace, warn};

use crate::engine::{QuoteTask, SwapOpportunity, VariantId};
use crate::lander::{LanderError, LanderReceipt};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

use super::metrics::prometheus_enabled;
use metrics::{counter, gauge, histogram};

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

pub fn pure_blind_route_registered(route: &str, legs: usize, source: &str) {
    info!(
        target: "monitoring::pure_blind",
        event = "route_registered",
        route,
        legs,
        source,
        "pure blind route registered"
    );

    if prometheus_enabled() {
        let route_label = route.to_string();
        let source_label = source.to_string();
        counter!(
            "galileo_pure_blind_routes_total",
            "route" => route_label.clone(),
            "source" => source_label.clone()
        )
        .increment(1);
        histogram!(
            "galileo_pure_blind_route_legs",
            "route" => route_label,
            "source" => source_label
        )
        .record(legs as f64);
    }
}

pub fn pure_blind_orders_prepared(route: &str, direction: &str, source: &str, count: usize) {
    if count == 0 {
        return;
    }

    info!(
        target: "monitoring::pure_blind",
        event = "orders_prepared",
        route,
        direction,
        source,
        count,
        "pure blind orders prepared"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_pure_blind_orders_total",
            "route" => route.to_string(),
            "direction" => direction.to_string(),
            "source" => source.to_string()
        )
        .increment(count as u64);
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

fn ip_label(ip: Option<IpAddr>) -> String {
    ip.map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn ip_inventory(ip: IpAddr, slot_kind: &str) {
    let ip_str = ip.to_string();
    trace!(
        target: "monitoring::ip",
        event = "inventory",
        ip = %ip_str,
        slot_kind,
        "registered ip slot"
    );
    if prometheus_enabled() {
        let slot_kind_label = slot_kind.to_string();
        let inventory_gauge = gauge!(
            "galileo_ip_inventory_total",
            "ip" => ip_str.clone(),
            "slot_kind" => slot_kind_label.clone()
        );
        inventory_gauge.set(1.0);

        let inflight_gauge = gauge!(
            "galileo_ip_inflight",
            "ip" => ip_str,
            "slot_kind" => slot_kind_label
        );
        inflight_gauge.set(0.0);
    }
}

pub fn ip_inflight(ip: IpAddr, slot_kind: &str, inflight: usize) {
    let ip_str = ip.to_string();
    trace!(
        target: "monitoring::ip",
        event = "inflight",
        ip = %ip_str,
        slot_kind,
        inflight,
        "ip inflight updated"
    );
    if prometheus_enabled() {
        let slot_kind_label = slot_kind.to_string();
        let inflight_gauge = gauge!(
            "galileo_ip_inflight",
            "ip" => ip_str,
            "slot_kind" => slot_kind_label
        );
        inflight_gauge.set(inflight as f64);
    }
}

pub fn ip_cooldown(ip: IpAddr, reason: &str, duration: Duration) {
    let millis = duration.as_millis() as f64;
    let ip_str = ip.to_string();
    trace!(
        target: "monitoring::ip",
        event = "cooldown",
        ip = %ip_str,
        reason,
        cooldown_ms = millis,
        "ip cooldown triggered"
    );
    if prometheus_enabled() {
        counter!(
            "galileo_ip_cooldown_total",
            "ip" => ip_str.clone(),
            "reason" => reason.to_string()
        )
        .increment(1);
        histogram!(
            "galileo_ip_cooldown_ms",
            "ip" => ip_str,
            "reason" => reason.to_string()
        )
        .record(millis);
    }
}

fn batch_label(batch_id: Option<u64>) -> String {
    batch_id
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

pub fn quote_start(
    strategy: &str,
    task: &QuoteTask,
    batch_id: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let batch_repr = batch_id.map(|value| value.to_string());
    let ip_repr = local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::quote",
        event = "start",
        strategy,
        input_mint = %task.pair.input_mint,
        output_mint = %task.pair.output_mint,
        amount = task.amount,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        "quote started"
    );
}

pub fn quote_end(
    strategy: &str,
    task: &QuoteTask,
    success: bool,
    elapsed: Duration,
    batch_id: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let latency_ms = elapsed.as_secs_f64() * 1_000.0;
    let batch_repr = batch_id.map(|value| value.to_string());
    let ip_repr = local_ip.map(|value| value.to_string());

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
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
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
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
            "quote returned no result"
        );
    }

    if prometheus_enabled() {
        let result = if success { "success" } else { "empty" };
        let ip_value = ip_label(local_ip);
        counter!(
            "galileo_quote_total",
            "strategy" => strategy.to_string(),
            "result" => result,
            "local_ip" => ip_value.clone(),
            "batch_id" => batch_label(batch_id)
        )
        .increment(1);
        histogram!(
            "galileo_quote_latency_ms",
            "strategy" => strategy.to_string(),
            "result" => result,
            "local_ip" => ip_value,
            "batch_id" => batch_label(batch_id)
        )
        .record(latency_ms);
    }
}

pub fn quote_round_trip(
    strategy: &str,
    task: &QuoteTask,
    aggregator: &str,
    first_leg_out: u64,
    round_trip_out: u64,
    batch_id: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let batch_repr = batch_id.map(|value| value.to_string());
    let ip_repr = local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::quote",
        event = "round_trip",
        strategy,
        base_mint = %task.pair.input_mint,
        quote_mint = %task.pair.output_mint,
        aggregator,
        amount_in = task.amount,
        first_leg_out = first_leg_out,
        round_trip_out = round_trip_out,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        "round-trip quote summary"
    );
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
    local_ip: Option<IpAddr>,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::swap",
        event = "fetched",
        strategy,
        input_mint = %opportunity.pair.input_mint,
        output_mint = %opportunity.pair.output_mint,
        amount_in = opportunity.amount_in,
        compute_unit_limit,
        prioritization_fee,
        local_ip = ?ip_repr,
        "swap instructions ready"
    );

    if prometheus_enabled() {
        let ip_value = ip_label(local_ip);
        histogram!(
            "galileo_swap_compute_unit_limit",
            "strategy" => strategy.to_string(),
            "local_ip" => ip_value.clone()
        )
        .record(compute_unit_limit as f64);
        histogram!(
            "galileo_swap_prioritization_fee_lamports",
            "strategy" => strategy.to_string(),
            "local_ip" => ip_value
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
    last_valid_block_height: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::transaction",
        event = "built",
        strategy,
        input_mint = %opportunity.pair.input_mint,
        output_mint = %opportunity.pair.output_mint,
        amount_in = opportunity.amount_in,
        slot,
        blockhash,
        last_valid_block_height,
        local_ip = ?ip_repr,
        "transaction prepared"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_transaction_built_total",
            "strategy" => strategy.to_string(),
            "local_ip" => ip_label(local_ip)
        )
        .increment(1);
    }
}

pub fn lander_attempt(
    strategy: &str,
    dispatch: &str,
    name: &str,
    variant: VariantId,
    attempt: usize,
    local_ip: Option<IpAddr>,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::lander",
        event = "attempt",
        strategy,
        dispatch,
        lander = name,
        variant,
        attempt,
        local_ip = ?ip_repr,
        "lander submission attempt"
    );

    if prometheus_enabled() {
        let ip_value = ip_label(local_ip);
        counter!(
            "galileo_lander_attempt_total",
            "strategy" => strategy.to_string(),
            "lander" => name.to_string(),
            "dispatch" => dispatch.to_string(),
            "variant" => variant.to_string(),
            "local_ip" => ip_value
        )
        .increment(1);
    }
}

pub fn lander_success(strategy: &str, dispatch: &str, attempt: usize, receipt: &LanderReceipt) {
    let ip_repr = receipt.local_ip.map(|value| value.to_string());
    info!(
        target: "monitoring::lander",
        event = "success",
        strategy,
        dispatch,
        lander = receipt.lander,
        variant = receipt.variant_id,
        attempt,
        endpoint = %receipt.endpoint,
        slot = receipt.slot,
        blockhash = %receipt.blockhash,
        signature = receipt.signature.as_deref().unwrap_or_default(),
        local_ip = ?ip_repr,
        "lander submission succeeded"
    );

    if prometheus_enabled() {
        let ip_value = ip_label(receipt.local_ip);
        counter!(
            "galileo_lander_success_total",
            "strategy" => strategy.to_string(),
            "lander" => receipt.lander.to_string(),
            "dispatch" => dispatch.to_string(),
            "variant" => receipt.variant_id.to_string(),
            "local_ip" => ip_value.clone()
        )
        .increment(1);
        counter!(
            "galileo_lander_submission_total",
            "strategy" => strategy.to_string(),
            "lander" => receipt.lander.to_string(),
            "dispatch" => dispatch.to_string(),
            "variant" => receipt.variant_id.to_string(),
            "local_ip" => ip_value,
            "result" => "success".to_string()
        )
        .increment(1);
    }
}

pub fn lander_failure(
    strategy: &str,
    dispatch: &str,
    name: &str,
    variant: VariantId,
    attempt: usize,
    local_ip: Option<IpAddr>,
    err: &LanderError,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    warn!(
        target: "monitoring::lander",
        event = "failure",
        strategy,
        dispatch,
        lander = name,
        variant,
        attempt,
        local_ip = ?ip_repr,
        error = %err,
        "lander submission failed"
    );

    if prometheus_enabled() {
        let ip_value = ip_label(local_ip);
        counter!(
            "galileo_lander_failure_total",
            "strategy" => strategy.to_string(),
            "lander" => name.to_string(),
            "dispatch" => dispatch.to_string(),
            "variant" => variant.to_string(),
            "local_ip" => ip_value.clone()
        )
        .increment(1);
        counter!(
            "galileo_lander_submission_total",
            "strategy" => strategy.to_string(),
            "lander" => name.to_string(),
            "dispatch" => dispatch.to_string(),
            "variant" => variant.to_string(),
            "local_ip" => ip_value,
            "result" => "failure".to_string()
        )
        .increment(1);
    }
}

pub fn copy_transaction_captured(wallet: &Pubkey, signature: &Signature, fanout: u32) {
    info!(
        target: "monitoring::copy",
        event = "captured",
        wallet = %wallet,
        signature = %signature,
        fanout,
        "copy candidate captured"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_copy_captured_total",
            "wallet" => wallet.to_string()
        )
        .increment(1);
    }
}

pub fn copy_transaction_dispatched(wallet: &Pubkey, signature: &Signature, attempt: u32) {
    info!(
        target: "monitoring::copy",
        event = "dispatched",
        wallet = %wallet,
        signature = %signature,
        attempt,
        "copy transaction dispatched"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_copy_dispatched_total",
            "wallet" => wallet.to_string()
        )
        .increment(1);
    }
}
