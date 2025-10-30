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
    let batch_display = batch_repr.as_deref().unwrap_or("none");
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = task.pair.input_mint.as_str();
    let output_mint = task.pair.output_mint.as_str();
    let amount = task.amount;
    info!(
        target: "monitoring::quote",
        strategy,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        input_mint = %input_mint,
        output_mint = %output_mint,
        amount,
        "{}",
        format_args!("报价开始: strategy={} input_mint={} output_mint={} amount={} batch_id={} node={}",
            strategy, input_mint, output_mint, amount, batch_display, ip_display)
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

    if success {
        info!(
            target: "monitoring::quote",
            strategy,
            latency_ms,
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
            input_mint = %input_mint,
            output_mint = %output_mint,
            amount,
            "{}",
            format_args!("报价完成: status=success input_mint={} output_mint={} amount={} latency={:.3}ms batch_id={} node={}",
                input_mint, output_mint, amount, latency_ms, batch_display, ip_display)
        );
    } else {
        warn!(
            target: "monitoring::quote",
            strategy,
            latency_ms,
            batch_id = ?batch_repr,
            local_ip = ?ip_repr,
            input_mint = %input_mint,
            output_mint = %output_mint,
            amount,
            "{}",
            format_args!("报价完成: status=empty input_mint={} output_mint={} amount={} latency={:.3}ms batch_id={} node={}",
                input_mint, output_mint, amount, latency_ms, batch_display, ip_display)
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
    let batch_display = batch_repr.as_deref().unwrap_or("none");
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let base_mint = task.pair.input_mint.as_str();
    let quote_mint = task.pair.output_mint.as_str();
    let amount_in = task.amount;
    info!(
        target: "monitoring::quote",
        strategy,
        aggregator,
        batch_id = ?batch_repr,
        local_ip = ?ip_repr,
        base_mint = %base_mint,
        quote_mint = %quote_mint,
        amount_in,
        first_leg_out,
        round_trip_out,
        "{}",
        format_args!("报价回环: aggregator={} base_mint={} quote_mint={} amount_in={} first_leg_out={} round_trip_out={} batch_id={} node={}",
            aggregator, base_mint, quote_mint, amount_in, first_leg_out, round_trip_out, batch_display, ip_display)
    );
}

pub fn profit_detected(strategy: &str, opportunity: &SwapOpportunity) {
    let input_mint = opportunity.pair.input_mint.as_str();
    let output_mint = opportunity.pair.output_mint.as_str();
    let amount_in = opportunity.amount_in;
    let profit = opportunity.profit_lamports;
    let net_profit = opportunity.net_profit();
    info!(
        target: "monitoring::profit",
        strategy,
        "{}",
        format_args!(
            "检测到套利机会: strategy={} input_mint={} output_mint={} amount_in={} profit={} net_profit={}",
            strategy, input_mint, output_mint, amount_in, profit, net_profit
        )
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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = opportunity.pair.input_mint.as_str();
    let output_mint = opportunity.pair.output_mint.as_str();
    let amount_in = opportunity.amount_in;
    info!(
        target: "monitoring::swap",
        strategy,
        input_mint = %input_mint,
        output_mint = %output_mint,
        amount_in,
        compute_unit_limit,
        prioritization_fee,
        local_ip = ?ip_repr,
        "{}",
        format_args!("Swap 指令就绪: strategy={} input_mint={} output_mint={} amount={} compute_unit_limit={} prioritization_fee={} node={}",
            strategy, input_mint, output_mint, amount_in, compute_unit_limit, prioritization_fee, ip_display)
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
    last_valid_block_height: Option<u64>,
    local_ip: Option<IpAddr>,
) {
    let ip_repr = local_ip.map(|value| value.to_string());
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let input_mint = opportunity.pair.input_mint.as_str();
    let output_mint = opportunity.pair.output_mint.as_str();
    let amount_in = opportunity.amount_in;
    info!(
        target: "monitoring::transaction",
        strategy,
        input_mint = %input_mint,
        output_mint = %output_mint,
        amount_in,
        slot,
        blockhash,
        last_valid_block_height,
        local_ip = ?ip_repr,
        "{}",
        format_args!("交易构建完成: strategy={} input_mint={} output_mint={} amount={} slot={} blockhash={} last_valid={} node={}",
            strategy, input_mint, output_mint, amount_in, slot, blockhash,
            last_valid_block_height.map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string()),
            ip_display)
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
    info!(
        target: "monitoring::lander",
        strategy,
        dispatch,
        lander = name,
        variant,
        attempt,
        local_ip = ?ip_repr,
        "{}",
        format_args!(
            "落地器尝试: 策略={} 调度={} 落地器={} 变体={} 尝试={} 节点={}",
            strategy,
            dispatch,
            name,
            variant,
            attempt,
            ip_display
        )
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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
    let signature = receipt.signature.as_deref().unwrap_or_default();
    info!(
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
        "{}",
        format_args!(
            "落地器成功: 策略={} 调度={} 落地器={} 变体={} 尝试={} Slot={} Blockhash={} Endpoint={} 签名={} 节点={}",
            strategy,
            dispatch,
            receipt.lander,
            receipt.variant_id,
            attempt,
            receipt.slot,
            receipt.blockhash,
            receipt.endpoint,
            signature,
            ip_display
        )
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
    let ip_display = ip_repr.as_deref().unwrap_or("unknown");
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
            strategy,
            dispatch,
            name,
            variant,
            attempt,
            ip_display,
            err
        )
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
        wallet = %wallet,
        signature = %signature,
        fanout,
        "{}",
        format_args!(
            "复制候选捕获: 钱包={} 签名={} Fanout={}",
            wallet, signature, fanout
        )
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
