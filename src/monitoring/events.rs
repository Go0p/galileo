use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tracing::{debug, info, trace, warn};

use crate::engine::{QuoteTask, SwapOpportunity, VariantId};
use crate::lander::{LanderError, LanderReceipt};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

use super::format::short_mint_str;
use super::metrics::prometheus_enabled;
use metrics::{counter, gauge, histogram};

static SUMMARY_ONLY_MODE: AtomicBool = AtomicBool::new(false);

pub struct SummaryModeGuard {
    previous: bool,
}

impl SummaryModeGuard {
    pub fn new(enable: bool) -> Self {
        let previous = SUMMARY_ONLY_MODE.swap(enable, Ordering::Relaxed);
        Self { previous }
    }
}

impl Drop for SummaryModeGuard {
    fn drop(&mut self) {
        SUMMARY_ONLY_MODE.store(self.previous, Ordering::Relaxed);
    }
}

pub fn summary_only_enabled() -> bool {
    SUMMARY_ONLY_MODE.load(Ordering::Relaxed)
}

fn base_mint_label(base_mint: Option<&Pubkey>) -> String {
    base_mint
        .map(|mint| short_mint_str(&mint.to_string()).into_owned())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn assembly_pipeline_started(base_mint: Option<&Pubkey>, decorator_count: usize) {
    let mint_label = base_mint_label(base_mint);
    trace!(
        target: "monitoring::assembly",
        event = "pipeline_start",
        base_mint = %mint_label,
        decorators = decorator_count,
        "instruction assembly pipeline started"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_assembly_pipeline_total",
            "base_mint" => mint_label.clone()
        )
        .increment(1);
        histogram!(
            "galileo_assembly_decorator_count",
            "base_mint" => mint_label
        )
        .record(decorator_count as f64);
    }
}

pub fn assembly_decorator_applied(
    decorator: &str,
    base_mint: Option<&Pubkey>,
    before_limit: u32,
    after_limit: u32,
    before_guard: u64,
    after_guard: u64,
    before_price: Option<u64>,
    after_price: Option<u64>,
) {
    let mint_label = base_mint_label(base_mint);
    trace!(
        target: "monitoring::assembly",
        event = "decorator_applied",
        decorator,
        base_mint = %mint_label,
        before_limit,
        after_limit,
        before_guard,
        after_guard,
        before_price = before_price.unwrap_or(0),
        after_price = after_price.unwrap_or(0),
        "assembly decorator applied"
    );

    if prometheus_enabled() {
        let decorator_label = decorator.to_string();
        counter!(
            "galileo_assembly_decorator_total",
            "decorator" => decorator_label.clone(),
            "base_mint" => mint_label.clone()
        )
        .increment(1);
        histogram!(
            "galileo_assembly_compute_units",
            "decorator" => decorator_label.clone(),
            "base_mint" => mint_label.clone()
        )
        .record(after_limit as f64);
        histogram!(
            "galileo_assembly_guard_lamports",
            "decorator" => decorator_label.clone(),
            "base_mint" => mint_label.clone()
        )
        .record(after_guard as f64);
        if let Some(price) = after_price {
            histogram!(
                "galileo_assembly_compute_price",
                "decorator" => decorator_label,
                "base_mint" => mint_label
            )
            .record(price as f64);
        }
    }
}

pub fn assembly_decorator_failed(decorator: &str, base_mint: Option<&Pubkey>, error: &str) {
    let mint_label = base_mint_label(base_mint);
    warn!(
        target: "monitoring::assembly",
        event = "decorator_failed",
        decorator,
        base_mint = %mint_label,
        error,
        "assembly decorator failed"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_assembly_decorator_failures_total",
            "decorator" => decorator.to_string(),
            "base_mint" => mint_label
        )
        .increment(1);
    }
}

pub fn assembly_pipeline_completed(
    base_mint: Option<&Pubkey>,
    decorator_count: usize,
    compute_len: usize,
    pre_len: usize,
    main_len: usize,
    post_len: usize,
    compute_unit_limit: u32,
    compute_unit_price: Option<u64>,
    guard_required: u64,
    flashloan_applied: bool,
) {
    let mint_label = base_mint_label(base_mint);
    let total_instructions = compute_len + pre_len + main_len + post_len;
    let compute_price = compute_unit_price.unwrap_or(0);

    debug!(
        target: "monitoring::assembly",
        event = "pipeline_complete",
        base_mint = %mint_label,
        decorators = decorator_count,
        total_instructions,
        compute_unit_limit,
        compute_unit_price = compute_price,
        guard_required,
        flashloan = flashloan_applied,
        "instruction assembly pipeline finished"
    );

    if prometheus_enabled() {
        let final_label = "final".to_string();
        histogram!(
            "galileo_assembly_instruction_span",
            "base_mint" => mint_label.clone()
        )
        .record(total_instructions as f64);
        histogram!(
            "galileo_assembly_compute_units",
            "decorator" => final_label.clone(),
            "base_mint" => mint_label.clone()
        )
        .record(compute_unit_limit as f64);
        histogram!(
            "galileo_assembly_guard_lamports",
            "decorator" => final_label.clone(),
            "base_mint" => mint_label.clone()
        )
        .record(guard_required as f64);
        if let Some(price) = compute_unit_price {
            histogram!(
                "galileo_assembly_compute_price",
                "decorator" => final_label.clone(),
                "base_mint" => mint_label.clone()
            )
            .record(price as f64);
        }
        if flashloan_applied {
            counter!(
                "galileo_assembly_flashloan_total",
                "base_mint" => mint_label
            )
            .increment(1);
        }
    }
}

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

pub fn pure_blind_cache_snapshot_written(target: &str, entries: usize) {
    info!(
        target: "monitoring::pure_blind",
        event = "cache_snapshot_written",
        target,
        entries,
        "pure blind cache snapshot written"
    );

    if prometheus_enabled() {
        let counter = counter!(
            "galileo_pure_blind_cache_snapshot_written_total",
            "target" => target.to_string()
        );
        counter.increment(1);
        histogram!(
            "galileo_pure_blind_cache_snapshot_entries",
            "target" => target.to_string()
        )
        .record(entries as f64);
    }
}

pub fn pure_blind_cache_snapshot_skipped(target: &str, reason: &str) {
    debug!(
        target: "monitoring::pure_blind",
        event = "cache_snapshot_skipped",
        target,
        reason,
        "pure blind cache snapshot skipped"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_pure_blind_cache_snapshot_skipped_total",
            "target" => target.to_string(),
            "reason" => reason.to_string()
        )
        .increment(1);
    }
}

pub fn pure_blind_cache_pruned(target: &str, count: usize) {
    if count == 0 {
        return;
    }

    warn!(
        target: "monitoring::pure_blind",
        event = "cache_pruned",
        target,
        count,
        "pure blind catalog pruned entries"
    );

    if prometheus_enabled() {
        counter!(
            "galileo_pure_blind_cache_pruned_total",
            "target" => target.to_string()
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

fn batch_label(batch_id: Option<u64>) -> &'static str {
    if batch_id.is_some() {
        "present"
    } else {
        "none"
    }
}

pub fn quote_start(
    strategy: &str,
    task: &QuoteTask,
    batch_id: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let batch_repr = batch_id.map(|value| value.to_string());
    let ip_repr = local_ip.map(|value| value.to_string());
    let batch_display = batch_repr.as_deref().unwrap_or("none");
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = task.pair.input_mint.as_str();
    let output_mint = task.pair.output_mint.as_str();
    let amount = task.amount;
    let input_display = short_mint_str(input_mint);
    let output_display = short_mint_str(output_mint);
    debug!(
        target: "monitoring::quote",
        strategy,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        input_mint = %input_display,
        output_mint = %output_display,
        amount,
        "quote_start batch={} node={}",
        batch_display,
        ip_display
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
    let batch_display = batch_repr.as_deref().unwrap_or("none");
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = task.pair.input_mint.as_str();
    let output_mint = task.pair.output_mint.as_str();
    let amount = task.amount;

    let input_display = short_mint_str(input_mint);
    let output_display = short_mint_str(output_mint);
    let result_label = if success { "success" } else { "empty" };
    if success {
        debug!(
            target: "monitoring::quote",
            strategy,
            result = result_label,
            latency_ms,
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
            input_mint = %input_display,
            output_mint = %output_display,
            amount,
            "quote_end batch={} node={}",
            batch_display,
            ip_display
        );
    } else {
        warn!(
            target: "monitoring::quote",
            strategy,
            result = result_label,
            latency_ms,
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
            input_mint = %input_display,
            output_mint = %output_display,
            amount,
            "quote_end batch={} node={}",
            batch_display,
            ip_display
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
            "batch" => batch_label(batch_id)
        )
        .increment(1);
        histogram!(
            "galileo_quote_latency_ms",
            "strategy" => strategy.to_string(),
            "result" => result,
            "local_ip" => ip_value,
            "batch" => batch_label(batch_id)
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
    let batch_display = batch_repr.as_deref().unwrap_or("none");
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let base_mint = task.pair.input_mint.as_str();
    let quote_mint = task.pair.output_mint.as_str();
    let amount_in = task.amount;
    let base_display = short_mint_str(base_mint);
    let quote_display = short_mint_str(quote_mint);
    debug!(
        target: "monitoring::quote",
        strategy,
        aggregator,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        base_mint = %base_display,
        quote_mint = %quote_display,
        amount_in,
        first_leg_out,
        round_trip_out,
        "quote_round_trip batch={} node={}",
        batch_display,
        ip_display
    );
}

pub fn profit_shortfall(
    base_mint: &str,
    aggregator: &str,
    forward_in: u64,
    forward_out: u64,
    forward_latency_ms: Option<f64>,
    reverse_in: u64,
    reverse_out: u64,
    reverse_latency_ms: Option<f64>,
    profit: u64,
    expected_profit: u64,
) {
    if summary_only_enabled() {
        return;
    }
    let base_display = short_mint_str(base_mint);
    let forward_latency_str = forward_latency_ms.map(|ms| format!("{ms:.3}"));
    let reverse_latency_str = reverse_latency_ms.map(|ms| format!("{ms:.3}"));
    let forward_latency_display = forward_latency_str.as_deref().unwrap_or("-");
    let reverse_latency_display = reverse_latency_str.as_deref().unwrap_or("-");

    info!(
        target: "monitoring::profit",
        event = "shortfall",
        aggregator,
        profit,
        expected_profit,
        "利润不足 base_mint={} forward={{in={},out={},latency_ms={}}} reverse={{in={},out={},latency_ms={}}}",
        base_display,
        forward_in,
        forward_out,
        forward_latency_display,
        reverse_in,
        reverse_out,
        reverse_latency_display
    );
}

pub fn profit_opportunity(
    base_mint: &str,
    aggregator: &str,
    forward_in: u64,
    forward_out: u64,
    forward_latency_ms: Option<f64>,
    reverse_in: u64,
    reverse_out: u64,
    reverse_latency_ms: Option<f64>,
    profit: u64,
    net_profit: i128,
    expected_profit: u64,
    multi_ip_enabled: bool,
    forward_ip: Option<IpAddr>,
    reverse_ip: Option<IpAddr>,
    total_latency_ms: Option<f64>,
) {
    if summary_only_enabled() {
        return;
    }
    let base_display = short_mint_str(base_mint);
    let forward_latency_str = forward_latency_ms.map(|ms| format!("{ms:.3}"));
    let reverse_latency_str = reverse_latency_ms.map(|ms| format!("{ms:.3}"));
    let forward_latency_display = forward_latency_str.as_deref().unwrap_or("-");
    let reverse_latency_display = reverse_latency_str.as_deref().unwrap_or("-");
    let total_latency_str = total_latency_ms.map(|ms| format!("{ms:.3}"));
    let total_latency_display = total_latency_str.as_deref().unwrap_or("-");

    if multi_ip_enabled {
        let forward_ip_display = forward_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "-".to_string());
        let reverse_ip_display = reverse_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "-".to_string());

        info!(
            target: "monitoring::profit",
            event = "opportunity",
            aggregator,
            net_profit,
            expected_profit,
            total_latency_ms = total_latency_display,
            "发现机会 base_mint={} profit={} forward={{in={},out={},latency_ms={},ip={}}} reverse={{in={},out={},latency_ms={},ip={}}}",
            base_display,
            profit,
            forward_in,
            forward_out,
            forward_latency_display,
            forward_ip_display,
            reverse_in,
            reverse_out,
            reverse_latency_display,
            reverse_ip_display
        );
    } else {
        info!(
            target: "monitoring::profit",
            event = "opportunity",
            aggregator,
            net_profit,
            expected_profit,
            total_latency_ms = total_latency_display,
            "发现机会 base_mint={} profit={} forward={{in={},out={},latency_ms={}}} reverse={{in={},out={},latency_ms={}}}",
            base_display,
            profit,
            forward_in,
            forward_out,
            forward_latency_display,
            reverse_in,
            reverse_out,
            reverse_latency_display
        );
    }
}

pub fn profit_detected(strategy: &str, opportunity: &SwapOpportunity) {
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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = opportunity.pair.input_mint.as_str();
    let output_mint = opportunity.pair.output_mint.as_str();
    let amount_in = opportunity.amount_in;
    let input_display = short_mint_str(input_mint);
    let output_display = short_mint_str(output_mint);
    debug!(
        target: "monitoring::swap",
        strategy,
        input_mint = %input_display,
        output_mint = %output_display,
        amount_in,
        compute_unit_limit,
        prioritization_fee,
        local_ip = ?ip_repr,
        "swap_ready node={}",
        ip_display
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
        strategy,
        protocol,
        mint = %mint,
        borrow_amount,
        inner_instruction_count,
        "{}",
        format_args!(
            "闪电贷注入: 策略={} 协议={} 铸币={} 借用量={} 指令数={}",
            strategy,
            protocol,
            mint,
            borrow_amount,
            inner_instruction_count
        )
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
    local_ip: Option<IpAddr>,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = opportunity.pair.input_mint.as_str();
    let output_mint = opportunity.pair.output_mint.as_str();
    let amount_in = opportunity.amount_in;
    let input_display = short_mint_str(input_mint);
    let output_display = short_mint_str(output_mint);
    debug!(
        target: "monitoring::transaction",
        strategy,
        input_mint = %input_display,
        output_mint = %output_display,
        amount_in,
        slot,
        blockhash,
        local_ip = ?ip_repr,
        "transaction_built node={}",
        ip_display
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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    if !summary_only_enabled() {
        debug!(
            target: "monitoring::lander",
            strategy,
            dispatch,
            lander = name,
            variant,
            attempt,
            local_ip = ?ip_repr,
            "lander_attempt node={}",
            ip_display
        );
    }

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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let signature = receipt.signature.as_deref().unwrap_or_default();
    if !summary_only_enabled() {
        debug!(
            target: "monitoring::lander",
            strategy,
            dispatch,
            lander = receipt.lander,
            variant = receipt.variant_id,
            attempt,
            endpoint = %receipt.endpoint,
            slot = receipt.slot,
            blockhash = %receipt.blockhash,
            signature = signature,
            local_ip = ?ip_repr,
            "lander_success node={}",
            ip_display
        );
    }

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
    tip: Option<(&str, u64)>,
    compute_unit_price: Option<(&str, u64)>,
    err: &LanderError,
) {
    if summary_only_enabled() {
        return;
    }
    let ip_repr = local_ip.map(|value| value.to_string());
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    match (tip, compute_unit_price) {
        (Some((tip_strategy, tips)), Some((price_strategy, cu_price))) => {
            warn!(
                target: "monitoring::lander",
                strategy,
                dispatch,
                lander = name,
                variant,
                attempt,
                local_ip = ?ip_repr,
                tip_strategy,
                tips,
                compute_unit_price_strategy = price_strategy,
                cu_price,
                error = %err,
                "{}",
                format_args!(
                    "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 节点={} tip_strategy={} tips={} compute_unit_price_strategy={} cu_price={} 错误={}",
                    strategy,
                    dispatch,
                    name,
                    variant,
                    attempt,
                    ip_display,
                    tip_strategy,
                    tips,
                    price_strategy,
                    cu_price,
                    err
                )
            );
        }
        (Some((tip_strategy, tips)), None) => {
            warn!(
                target: "monitoring::lander",
                strategy,
                dispatch,
                lander = name,
                variant,
                attempt,
                local_ip = ?ip_repr,
                tip_strategy,
                tips,
                error = %err,
                "{}",
                format_args!(
                    "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 节点={} tip_strategy={} tips={} 错误={}",
                    strategy, dispatch, name, variant, attempt, ip_display, tip_strategy, tips, err
                )
            );
        }
        (None, Some((price_strategy, cu_price))) => {
            warn!(
                target: "monitoring::lander",
                strategy,
                dispatch,
                lander = name,
                variant,
                attempt,
                local_ip = ?ip_repr,
                compute_unit_price_strategy = price_strategy,
                cu_price,
                error = %err,
                "{}",
                format_args!(
                    "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 节点={} compute_unit_price_strategy={} cu_price={} 错误={}",
                    strategy, dispatch, name, variant, attempt, ip_display, price_strategy, cu_price, err
                )
            );
        }
        (None, None) => {
            warn!(
                target: "monitoring::lander",
                strategy,
                dispatch,
                lander = name,
                variant,
                attempt,
                local_ip = ?ip_repr,
                error = %err,
                "{}",
                format_args!(
                    "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 节点={} 错误={}",
                    strategy, dispatch, name, variant, attempt, ip_display, err
                )
            );
        }
    }
}

pub fn copy_transaction_captured(
    wallet: &Pubkey,
    signature: &Signature,
    fanout: u32,
    source_tips: Option<u64>,
) {
    match source_tips {
        Some(source_tips) => info!(
            target: "monitoring::copy",
            wallet = %wallet,
            signature = %signature,
            fanout,
            source_tips = source_tips,
            "{}",
            format_args!(
                "复制候选捕获: 钱包={} 签名={} Fanout={} source_tips={}",
                wallet, signature, fanout, source_tips
            )
        ),
        None => info!(
            target: "monitoring::copy",
            wallet = %wallet,
            signature = %signature,
            fanout,
            "{}",
            format_args!(
                "复制候选捕获: 钱包={} 签名={} Fanout={}",
                wallet, signature, fanout
            )
        ),
    }

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
        wallet = %wallet,
        signature = %signature,
        attempt,
        "{}",
        format_args!(
            "复制交易下发: 钱包={} 签名={} 尝试={}",
            wallet, signature, attempt
        )
    );

    if prometheus_enabled() {
        counter!(
            "galileo_copy_dispatched_total",
            "wallet" => wallet.to_string()
        )
        .increment(1);
    }
}

pub fn copy_queue_depth(wallet: &Pubkey, depth: usize) {
    if prometheus_enabled() {
        gauge!(
            "galileo_copy_queue_depth",
            "wallet" => wallet.to_string()
        )
        .set(depth as f64);
    }
}

pub fn copy_queue_task_dropped(wallet: &Pubkey) {
    if prometheus_enabled() {
        counter!(
            "galileo_copy_queue_dropped_total",
            "wallet" => wallet.to_string()
        )
        .increment(1);
    }
}

pub fn copy_queue_workers(wallet: &Pubkey, workers: usize) {
    if prometheus_enabled() {
        gauge!(
            "galileo_copy_queue_workers",
            "wallet" => wallet.to_string()
        )
        .set(workers as f64);
    }
}

pub fn copy_wallet_refresh_success(wallet: &Pubkey, accounts: usize) {
    info!(
        target: "monitoring::copy",
        wallet = %wallet,
        accounts,
        "{}",
        format_args!(
            "复制钱包刷新成功: 钱包={} 账户数={}",
            wallet, accounts
        )
    );

    if prometheus_enabled() {
        counter!(
            "galileo_copy_wallet_refresh_total",
            "wallet" => wallet.to_string(),
            "result" => "success".to_string()
        )
        .increment(1);
        gauge!(
            "galileo_copy_wallet_accounts",
            "wallet" => wallet.to_string()
        )
        .set(accounts as f64);
    }
}

pub fn copy_wallet_refresh_error(wallet: &Pubkey) {
    warn!(
        target: "monitoring::copy",
        wallet = %wallet,
        "{}",
        format_args!("复制钱包刷新失败: 钱包={}", wallet)
    );

    if prometheus_enabled() {
        counter!(
            "galileo_copy_wallet_refresh_total",
            "wallet" => wallet.to_string(),
            "result" => "error".to_string()
        )
        .increment(1);
    }
}
