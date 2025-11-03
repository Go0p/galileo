use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use solana_sdk::instruction::Instruction;
use tracing::{info, warn};

use crate::engine::{
    COMPUTE_BUDGET_PROGRAM_ID, DispatchPlan, DispatchStrategy, JitoTipPlan, TxVariant, VariantId,
};
use crate::monitoring::events;
use crate::network::{IpAllocator, IpLeaseMode, IpLeaseOutcome, IpTaskKind};

use super::error::LanderError;
use super::jito::JitoLander;
use super::rpc::RpcLander;
use super::staked::StakedLander;

#[derive(Clone, Copy)]
pub struct Deadline(Instant);

impl Deadline {
    pub fn from_instant(instant: Instant) -> Self {
        Self(instant)
    }

    pub fn expired(&self) -> bool {
        Instant::now() > self.0
    }
}

#[derive(Debug, Clone)]
pub struct LanderReceipt {
    pub lander: &'static str,
    pub endpoint: String,
    pub slot: u64,
    pub blockhash: String,
    pub signature: Option<String>,
    pub variant_id: VariantId,
    pub local_ip: Option<IpAddr>,
}

#[derive(Clone)]
pub enum LanderVariant {
    Rpc(RpcLander),
    Jito(JitoLander),
    Staked(StakedLander),
}

impl LanderVariant {
    pub fn name(&self) -> &'static str {
        match self {
            LanderVariant::Rpc(_) => "rpc",
            LanderVariant::Jito(_) => "jito",
            LanderVariant::Staked(_) => "staked",
        }
    }

    pub fn one_by_one_capacity(&self) -> usize {
        match self {
            LanderVariant::Rpc(_) => 1,
            LanderVariant::Jito(lander) => lander.endpoints().max(1),
            LanderVariant::Staked(lander) => lander.endpoints_len().max(1),
        }
    }

    pub fn endpoints(&self) -> Vec<String> {
        match self {
            LanderVariant::Rpc(_) => Vec::new(),
            LanderVariant::Jito(lander) => lander.endpoint_list(),
            LanderVariant::Staked(lander) => lander.endpoint_list(),
        }
    }

    pub fn draw_tip_plan(&self) -> Option<JitoTipPlan> {
        match self {
            LanderVariant::Jito(lander) => lander.draw_tip_plan(),
            _ => None,
        }
    }

    pub fn tip_strategy_label(&self) -> Option<&'static str> {
        match self {
            LanderVariant::Jito(lander) => Some(lander.tip_strategy_label()),
            _ => None,
        }
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        endpoint: Option<&str>,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        match self {
            LanderVariant::Rpc(lander) => lander.submit_variant(variant, deadline, local_ip).await,
            LanderVariant::Jito(lander) => {
                lander
                    .submit_variant(variant, deadline, endpoint, local_ip)
                    .await
            }
            LanderVariant::Staked(lander) => {
                lander
                    .submit_variant(variant, deadline, endpoint, local_ip)
                    .await
            }
        }
    }
}

#[derive(Clone)]
pub struct LanderStack {
    landers: Vec<LanderVariant>,
    max_retries: usize,
    ip_allocator: Arc<IpAllocator>,
}

impl LanderStack {
    pub fn new(
        landers: Vec<LanderVariant>,
        max_retries: usize,
        ip_allocator: Arc<IpAllocator>,
    ) -> Self {
        Self {
            landers,
            max_retries,
            ip_allocator,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.landers.is_empty()
    }

    pub fn count(&self) -> usize {
        self.landers.len()
    }

    pub fn has_jito(&self) -> bool {
        self.landers
            .iter()
            .any(|lander| matches!(lander, LanderVariant::Jito(_)))
    }

    pub fn has_non_jito(&self) -> bool {
        self.landers
            .iter()
            .any(|lander| !matches!(lander, LanderVariant::Jito(_)))
    }

    pub fn variants(&self) -> &[LanderVariant] {
        &self.landers
    }

    pub fn draw_jito_tip_plan(&self) -> Option<JitoTipPlan> {
        self.landers.iter().find_map(|lander| match lander {
            LanderVariant::Jito(lander) => lander.draw_tip_plan(),
            _ => None,
        })
    }

    pub fn variant_layout(&self, strategy: DispatchStrategy) -> Vec<usize> {
        match strategy {
            DispatchStrategy::AllAtOnce => vec![1; self.landers.len()],
            DispatchStrategy::OneByOne => self
                .landers
                .iter()
                .map(|lander| lander.one_by_one_capacity().max(1))
                .collect(),
        }
    }

    pub async fn submit_plan(
        &self,
        plan: &DispatchPlan,
        deadline: Deadline,
        strategy: &str,
    ) -> Result<LanderReceipt, LanderError> {
        if self.landers.is_empty() {
            return Err(LanderError::fatal("no lander configured"));
        }
        if plan.is_empty() {
            return Err(LanderError::fatal("dispatch plan missing variants"));
        }

        match plan.strategy() {
            DispatchStrategy::AllAtOnce => {
                self.dispatch_all_at_once(plan, deadline, strategy).await
            }
            DispatchStrategy::OneByOne => self.dispatch_one_by_one(plan, deadline, strategy).await,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn dispatch_all_at_once(
        &self,
        plan: &DispatchPlan,
        deadline: Deadline,
        strategy_name: &str,
    ) -> Result<LanderReceipt, LanderError> {
        let dispatch_label = plan.strategy().as_str();
        let total_passes = self.max_retries.saturating_add(1);
        let mut attempt_idx = 0usize;
        let mut last_err: Option<LanderError> = None;

        for _ in 0..total_passes {
            if deadline.expired() {
                return Err(LanderError::fatal("deadline expired before submission"));
            }

            let mut futures = FuturesUnordered::new();

            for (lander_idx, lander) in self.landers.iter().enumerate() {
                if deadline.expired() {
                    return Err(LanderError::fatal("deadline expired before submission"));
                }

                let variants = plan.variants_for_lander(lander_idx);
                for variant in variants.iter().cloned() {
                    let lander_clone = lander.clone();
                    let lander_name = lander.name();
                    let variant_id = variant.id();
                    let variant_signature = variant.signature();
                    let current_attempt = attempt_idx;
                    let deadline_copy = deadline;
                    let allocator = Arc::clone(&self.ip_allocator);
                    let strategy = strategy_name.to_string();
                    let dispatch = dispatch_label.to_string();

                    attempt_idx += 1;

                    futures.push(async move {
                        let (local_ip, telemetry, result) = submit_with_lease(
                            allocator,
                            lander_clone,
                            variant,
                            deadline_copy,
                            None,
                            &strategy,
                            &dispatch,
                            lander_name,
                            variant_id,
                            current_attempt,
                        )
                        .await;
                        (
                            lander_name,
                            current_attempt,
                            variant_id,
                            variant_signature,
                            local_ip,
                            telemetry,
                            result,
                        )
                    });
                }
            }

            while let Some((
                lander_name,
                attempt,
                variant_id,
                signature,
                local_ip,
                telemetry,
                result,
            )) = futures.next().await
            {
                match result {
                    Ok(mut receipt) => {
                        if receipt.local_ip.is_none() {
                            receipt.local_ip = local_ip;
                        }
                        events::lander_success(strategy_name, dispatch_label, attempt, &receipt);
                        return Ok(receipt);
                    }
                    Err(err) => {
                        events::lander_failure(
                            strategy_name,
                            dispatch_label,
                            lander_name,
                            variant_id,
                            attempt,
                            local_ip,
                            telemetry.tip,
                            telemetry.compute_unit_price,
                            &err,
                        );
                        let sig_display = signature.as_deref().unwrap_or("<none>");
                        match (telemetry.tip, telemetry.compute_unit_price) {
                            (Some((tip_strategy, tips)), Some((price_strategy, cu_price))) => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
                                    tip_strategy,
                                    tips,
                                    compute_unit_price_strategy = price_strategy,
                                    cu_price,
                                    variant = variant_id,
                                    attempt,
                                    tx_signature = sig_display,
                                    "{}",
                                    format_args!(
                                        "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} tip_strategy={} tips={} compute_unit_price_strategy={} cu_price={}",
                                        strategy_name,
                                        dispatch_label,
                                        lander_name,
                                        variant_id,
                                        attempt,
                                        sig_display,
                                        tip_strategy,
                                        tips,
                                        price_strategy,
                                        cu_price
                                    )
                                );
                            }
                            (Some((tip_strategy, tips)), None) => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
                                    tip_strategy,
                                    tips,
                                    variant = variant_id,
                                    attempt,
                                    tx_signature = sig_display,
                                    "{}",
                                    format_args!(
                                        "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} tip_strategy={} tips={}",
                                        strategy_name,
                                        dispatch_label,
                                        lander_name,
                                        variant_id,
                                        attempt,
                                        sig_display,
                                        tip_strategy,
                                        tips
                                    )
                                );
                            }
                            (None, Some((price_strategy, cu_price))) => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
                                    compute_unit_price_strategy = price_strategy,
                                    cu_price,
                                    variant = variant_id,
                                    attempt,
                                    tx_signature = sig_display,
                                    "{}",
                                    format_args!(
                                        "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} compute_unit_price_strategy={} cu_price={}",
                                        strategy_name,
                                        dispatch_label,
                                        lander_name,
                                        variant_id,
                                        attempt,
                                        sig_display,
                                        price_strategy,
                                        cu_price
                                    )
                                );
                            }
                            (None, None) => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
                                    variant = variant_id,
                                    attempt,
                                    tx_signature = sig_display,
                                    "{}",
                                    format_args!(
                                        "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={}",
                                        strategy_name,
                                        dispatch_label,
                                        lander_name,
                                        variant_id,
                                        attempt,
                                        sig_display
                                    )
                                );
                            }
                        }
                        last_err = Some(err);
                    }
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all landers failed to submit transaction")))
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn dispatch_one_by_one(
        &self,
        plan: &DispatchPlan,
        deadline: Deadline,
        strategy_name: &str,
    ) -> Result<LanderReceipt, LanderError> {
        let dispatch_label = plan.strategy().as_str();
        let total_passes = self.max_retries.saturating_add(1);
        let mut attempt_idx = 0usize;
        let mut last_err: Option<LanderError> = None;

        for _ in 0..total_passes {
            if deadline.expired() {
                return Err(LanderError::fatal("deadline expired before submission"));
            }

            let mut futures = FuturesUnordered::new();

            for (lander_idx, lander) in self.landers.iter().enumerate() {
                let variants = plan.variants_for_lander(lander_idx);
                if variants.is_empty() {
                    continue;
                }

                let deliveries: Vec<(Option<String>, TxVariant)> = {
                    let endpoints = lander.endpoints();
                    if endpoints.is_empty() {
                        vec![(None, variants[0].clone())]
                    } else {
                        variants
                            .iter()
                            .cloned()
                            .zip(endpoints.into_iter())
                            .map(|(variant, endpoint)| (Some(endpoint), variant))
                            .collect()
                    }
                };

                for (endpoint_label, variant) in deliveries {
                    if deadline.expired() {
                        return Err(LanderError::fatal("deadline expired before submission"));
                    }

                    let lander_clone = lander.clone();
                    let lander_name = lander.name();
                    let variant_id = variant.id();
                    let variant_signature = variant.signature();
                    let current_attempt = attempt_idx;
                    let deadline_copy = deadline;
                    let allocator = Arc::clone(&self.ip_allocator);
                    let strategy = strategy_name.to_string();
                    let dispatch = dispatch_label.to_string();
                    let endpoint_hint = endpoint_label.clone();

                    attempt_idx += 1;

                    futures.push(async move {
                        let (local_ip, telemetry, result) = submit_with_lease(
                            allocator,
                            lander_clone,
                            variant,
                            deadline_copy,
                            endpoint_hint.clone(),
                            &strategy,
                            &dispatch,
                            lander_name,
                            variant_id,
                            current_attempt,
                        )
                        .await;
                        (
                            lander_name,
                            endpoint_label,
                            current_attempt,
                            variant_id,
                            variant_signature,
                            local_ip,
                            telemetry,
                            result,
                        )
                    });
                }
            }

            while let Some((
                lander_name,
                endpoint,
                attempt,
                variant_id,
                signature,
                local_ip,
                telemetry,
                result,
            )) = futures.next().await
            {
                match result {
                    Ok(mut receipt) => {
                        if receipt.local_ip.is_none() {
                            receipt.local_ip = local_ip;
                        }
                        events::lander_success(strategy_name, dispatch_label, attempt, &receipt);
                        return Ok(receipt);
                    }
                    Err(err) => {
                        events::lander_failure(
                            strategy_name,
                            dispatch_label,
                            lander_name,
                            variant_id,
                            attempt,
                            local_ip,
                            telemetry.tip,
                            telemetry.compute_unit_price,
                            &err,
                        );
                        match endpoint {
                            Some(endpoint_name) => {
                                match (telemetry.tip, telemetry.compute_unit_price) {
                                    (
                                        Some((tip_strategy, tips)),
                                        Some((price_strategy, cu_price)),
                                    ) => {
                                        warn!(
                                            target: "lander::stack",
                                            lander = lander_name,
                                            endpoint = endpoint_name,
                                            tip_strategy,
                                            tips,
                                            compute_unit_price_strategy = price_strategy,
                                            cu_price,
                                            tx_signature = signature.as_deref().unwrap_or(""),
                                            "{}",
                                            format_args!(
                                                "落地器失败: 策略={} 调度={} 落地器={} Endpoint={} 变体={} 尝试={} 签名={} tip_strategy={} tips={} compute_unit_price_strategy={} cu_price={}",
                                                strategy_name,
                                                dispatch_label,
                                                lander_name,
                                                endpoint_name,
                                                variant_id,
                                                attempt,
                                                signature.as_deref().unwrap_or("<none>"),
                                                tip_strategy,
                                                tips,
                                                price_strategy,
                                                cu_price
                                            )
                                        );
                                    }
                                    (Some((tip_strategy, tips)), None) => {
                                        warn!(
                                            target: "lander::stack",
                                            lander = lander_name,
                                            endpoint = endpoint_name,
                                            tip_strategy,
                                            tips,
                                            tx_signature = signature.as_deref().unwrap_or(""),
                                            "{}",
                                            format_args!(
                                                "落地器失败: 策略={} 调度={} 落地器={} Endpoint={} 变体={} 尝试={} 签名={} tip_strategy={} tips={}",
                                                strategy_name,
                                                dispatch_label,
                                                lander_name,
                                                endpoint_name,
                                                variant_id,
                                                attempt,
                                                signature.as_deref().unwrap_or("<none>"),
                                                tip_strategy,
                                                tips
                                            )
                                        );
                                    }
                                    (None, Some((price_strategy, cu_price))) => {
                                        warn!(
                                            target: "lander::stack",
                                            lander = lander_name,
                                            endpoint = endpoint_name,
                                            compute_unit_price_strategy = price_strategy,
                                            cu_price,
                                            tx_signature = signature.as_deref().unwrap_or(""),
                                            "{}",
                                            format_args!(
                                                "落地器失败: 策略={} 调度={} 落地器={} Endpoint={} 变体={} 尝试={} 签名={} compute_unit_price_strategy={} cu_price={}",
                                                strategy_name,
                                                dispatch_label,
                                                lander_name,
                                                endpoint_name,
                                                variant_id,
                                                attempt,
                                                signature.as_deref().unwrap_or("<none>"),
                                                price_strategy,
                                                cu_price
                                            )
                                        );
                                    }
                                    (None, None) => {
                                        warn!(
                                            target: "lander::stack",
                                            lander = lander_name,
                                            endpoint = endpoint_name,
                                            tx_signature = signature.as_deref().unwrap_or(""),
                                            "{}",
                                            format_args!(
                                                "落地器失败: 策略={} 调度={} 落地器={} Endpoint={} 变体={} 尝试={} 签名={}",
                                                strategy_name,
                                                dispatch_label,
                                                lander_name,
                                                endpoint_name,
                                                variant_id,
                                                attempt,
                                                signature.as_deref().unwrap_or("<none>")
                                            )
                                        );
                                    }
                                }
                            }
                            None => match (telemetry.tip, telemetry.compute_unit_price) {
                                (Some((tip_strategy, tips)), Some((price_strategy, cu_price))) => {
                                    warn!(
                                        target: "lander::stack",
                                        lander = lander_name,
                                        tip_strategy,
                                        tips,
                                        compute_unit_price_strategy = price_strategy,
                                        cu_price,
                                        tx_signature = signature.as_deref().unwrap_or(""),
                                        "{}",
                                        format_args!(
                                            "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} tip_strategy={} tips={} compute_unit_price_strategy={} cu_price={}",
                                            strategy_name,
                                            dispatch_label,
                                            lander_name,
                                            variant_id,
                                            attempt,
                                            signature.as_deref().unwrap_or("<none>"),
                                            tip_strategy,
                                            tips,
                                            price_strategy,
                                            cu_price
                                        )
                                    );
                                }
                                (Some((tip_strategy, tips)), None) => {
                                    warn!(
                                        target: "lander::stack",
                                        lander = lander_name,
                                        tip_strategy,
                                        tips,
                                        tx_signature = signature.as_deref().unwrap_or(""),
                                        "{}",
                                        format_args!(
                                            "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} tip_strategy={} tips={}",
                                            strategy_name,
                                            dispatch_label,
                                            lander_name,
                                            variant_id,
                                            attempt,
                                            signature.as_deref().unwrap_or("<none>"),
                                            tip_strategy,
                                            tips
                                        )
                                    );
                                }
                                (None, Some((price_strategy, cu_price))) => {
                                    warn!(
                                        target: "lander::stack",
                                        lander = lander_name,
                                        compute_unit_price_strategy = price_strategy,
                                        cu_price,
                                        tx_signature = signature.as_deref().unwrap_or(""),
                                        "{}",
                                        format_args!(
                                            "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={} compute_unit_price_strategy={} cu_price={}",
                                            strategy_name,
                                            dispatch_label,
                                            lander_name,
                                            variant_id,
                                            attempt,
                                            signature.as_deref().unwrap_or("<none>"),
                                            price_strategy,
                                            cu_price
                                        )
                                    );
                                }
                                (None, None) => {
                                    warn!(
                                        target: "lander::stack",
                                        lander = lander_name,
                                        tx_signature = signature.as_deref().unwrap_or(""),
                                        "{}",
                                        format_args!(
                                            "落地器失败: 策略={} 调度={} 落地器={} 变体={} 尝试={} 签名={}",
                                            strategy_name,
                                            dispatch_label,
                                            lander_name,
                                            variant_id,
                                            attempt,
                                            signature.as_deref().unwrap_or("<none>")
                                        )
                                    );
                                }
                            },
                        }
                        last_err = Some(err);
                    }
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all landers failed to submit transaction")))
    }
}

#[derive(Clone, Copy, Debug)]
struct SubmitTelemetry {
    tip: Option<(&'static str, u64)>,
    compute_unit_price: Option<(&'static str, u64)>,
}

async fn submit_with_lease(
    allocator: Arc<IpAllocator>,
    lander: LanderVariant,
    variant: TxVariant,
    deadline: Deadline,
    endpoint: Option<String>,
    strategy: &str,
    dispatch: &str,
    lander_name: &'static str,
    variant_id: VariantId,
    attempt: usize,
) -> (
    Option<IpAddr>,
    SubmitTelemetry,
    Result<LanderReceipt, LanderError>,
) {
    let endpoint_hash = compute_endpoint_hash(endpoint.as_deref(), variant_id);

    let instruction_price = compute_unit_price_from_instructions(variant.instructions());
    let tip_lamports = variant
        .jito_tip_plan()
        .map(|plan| plan.lamports)
        .unwrap_or_else(|| variant.tip_lamports());
    let _ = variant.prioritization_fee_lamports();
    let compute_unit_price_micro = variant
        .compute_unit_price_micro_lamports()
        .or(instruction_price)
        .unwrap_or(0);

    let is_jito = matches!(lander, LanderVariant::Jito(_));
    let tip = is_jito.then_some((variant.tip_strategy_label(), tip_lamports));
    let compute_unit_price = if is_jito {
        None
    } else {
        Some((
            variant.compute_unit_price_strategy_label(),
            compute_unit_price_micro,
        ))
    };

    let telemetry = SubmitTelemetry {
        tip,
        compute_unit_price,
    };

    let (local_ip, mut lease_handle) = match allocator
        .acquire(
            IpTaskKind::LanderSubmit { endpoint_hash },
            IpLeaseMode::Ephemeral,
        )
        .await
    {
        Ok(lease) => {
            let handle = lease.handle();
            let ip = handle.ip();
            drop(lease);
            (Some(ip), Some(handle))
        }
        Err(err) => {
            events::lander_attempt(strategy, dispatch, lander_name, variant_id, attempt, None);
            let failure = LanderError::fatal(format!("获取 IP 资源失败: {err}"));
            return (None, telemetry, Err(failure));
        }
    };

    events::lander_attempt(
        strategy,
        dispatch,
        lander_name,
        variant_id,
        attempt,
        local_ip,
    );

    let submission_started = Instant::now();
    let mut result = lander
        .submit_variant(variant, deadline, endpoint.as_deref(), local_ip)
        .await;

    if let Some(handle) = lease_handle.take() {
        if let Err(err) = &result {
            if let Some(outcome) = classify_lander_error(err) {
                handle.mark_outcome(outcome);
            }
        }
        drop(handle);
    }

    if let Ok(receipt) = &mut result {
        if receipt.local_ip.is_none() {
            receipt.local_ip = local_ip;
        }
        let elapsed_ms = submission_started.elapsed().as_secs_f64() * 1_000.0;
        let identifier = receipt
            .signature
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| receipt.endpoint.clone());
        let endpoint_num = receipt.variant_id;
        let ip_label = receipt
            .local_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "-".to_string());
        match (telemetry.tip, telemetry.compute_unit_price) {
            (Some((tip_strategy, tips)), _) => {
                info!(
                    target: "lander::submit",
                    strategy,
                    dispatch,
                    lander = receipt.lander,
                    tip_strategy,
                    tips,
                    "发送交易 endpoint_num={} endpoint={} id={} elapsed_ms={:.3} ip={}",
                    endpoint_num,
                    receipt.endpoint.as_str(),
                    identifier.as_str(),
                    elapsed_ms,
                    ip_label,
                );
            }
            (None, Some((price_strategy, cu_price))) => {
                info!(
                    target: "lander::submit",
                    strategy,
                    dispatch,
                    lander = receipt.lander,
                    compute_unit_price_strategy = price_strategy,
                    cu_price,
                    "发送交易 endpoint_num={} endpoint={} id={} elapsed_ms={:.3} ip={}",
                    endpoint_num,
                    receipt.endpoint.as_str(),
                    identifier.as_str(),
                    elapsed_ms,
                    ip_label,
                );
            }
            (None, None) => {
                info!(
                    target: "lander::submit",
                    strategy,
                    dispatch,
                    lander = receipt.lander,
                    "发送交易 endpoint_num={} endpoint={} id={} elapsed_ms={:.3} ip={}",
                    endpoint_num,
                    receipt.endpoint.as_str(),
                    identifier.as_str(),
                    elapsed_ms,
                    ip_label,
                );
            }
        }
    }

    (local_ip, telemetry, result)
}

fn compute_unit_price_from_instructions(instructions: &[Instruction]) -> Option<u64> {
    let mut price = None;
    for ix in instructions {
        if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            continue;
        }
        if ix.data.first().copied() == Some(3) && ix.data.len() >= 9 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&ix.data[1..9]);
            price = Some(u64::from_le_bytes(buf));
        }
    }
    price
}

fn compute_endpoint_hash(endpoint: Option<&str>, variant_id: VariantId) -> u64 {
    let mut hasher = DefaultHasher::new();
    if let Some(value) = endpoint {
        value.hash(&mut hasher);
    }
    variant_id.hash(&mut hasher);
    hasher.finish()
}

fn classify_lander_error(err: &LanderError) -> Option<IpLeaseOutcome> {
    match err {
        LanderError::Network(inner) => classify_reqwest(inner),
        LanderError::Rpc(_) => Some(IpLeaseOutcome::NetworkError),
        LanderError::Fatal(_) => Some(IpLeaseOutcome::NetworkError),
        _ => None,
    }
}

fn classify_reqwest(err: &reqwest::Error) -> Option<IpLeaseOutcome> {
    if err.is_timeout() {
        return Some(IpLeaseOutcome::Timeout);
    }
    if let Some(status) = err.status() {
        if status.as_u16() == 429 {
            return Some(IpLeaseOutcome::RateLimited);
        }
        if status.as_u16() == 408 || status.as_u16() == 504 {
            return Some(IpLeaseOutcome::Timeout);
        }
        if status.is_server_error() {
            return Some(IpLeaseOutcome::NetworkError);
        }
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}
