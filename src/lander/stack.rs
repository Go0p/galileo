use std::time::Instant;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use tracing::{info, warn};

use crate::engine::{DispatchPlan, DispatchStrategy, TxVariant, VariantId};
use crate::monitoring::events;

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

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        endpoint: Option<&str>,
    ) -> Result<LanderReceipt, LanderError> {
        match self {
            LanderVariant::Rpc(lander) => lander.submit_variant(variant, deadline).await,
            LanderVariant::Jito(lander) => lander.submit_variant(variant, deadline, endpoint).await,
            LanderVariant::Staked(lander) => {
                lander.submit_variant(variant, deadline, endpoint).await
            }
        }
    }
}

#[derive(Clone)]
pub struct LanderStack {
    landers: Vec<LanderVariant>,
    max_retries: usize,
}

impl LanderStack {
    pub fn new(landers: Vec<LanderVariant>, max_retries: usize) -> Self {
        Self {
            landers,
            max_retries,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.landers.is_empty()
    }

    pub fn count(&self) -> usize {
        self.landers.len()
    }

    pub fn plan_capacity(&self, strategy: DispatchStrategy) -> usize {
        match strategy {
            DispatchStrategy::AllAtOnce => 1,
            DispatchStrategy::OneByOne => self
                .landers
                .iter()
                .map(|lander| lander.one_by_one_capacity())
                .sum::<usize>()
                .max(1),
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
        if plan.variants().is_empty() {
            return Err(LanderError::fatal("dispatch plan missing variants"));
        }

        match plan.strategy() {
            DispatchStrategy::AllAtOnce => {
                self.dispatch_all_at_once(plan, deadline, strategy).await
            }
            DispatchStrategy::OneByOne => self.dispatch_one_by_one(plan, deadline, strategy).await,
        }
    }

    async fn dispatch_all_at_once(
        &self,
        plan: &DispatchPlan,
        deadline: Deadline,
        strategy_name: &str,
    ) -> Result<LanderReceipt, LanderError> {
        let variant = plan
            .primary_variant()
            .cloned()
            .ok_or_else(|| LanderError::fatal("dispatch plan missing primary variant"))?;
        let variant_id = variant.id();
        let dispatch_label = plan.strategy().as_str();
        let total_passes = self.max_retries.saturating_add(1);
        let mut attempt_idx = 0usize;
        let mut last_err: Option<LanderError> = None;

        for _ in 0..total_passes {
            if deadline.expired() {
                return Err(LanderError::fatal("deadline expired before submission"));
            }

            let mut futures = FuturesUnordered::new();

            for lander in &self.landers {
                let lander_clone = lander.clone();
                let variant_clone = variant.clone();
                let variant_signature = variant_clone.signature();
                let lander_name = lander.name();
                let current_attempt = attempt_idx;
                let deadline_copy = deadline;
                events::lander_attempt(
                    strategy_name,
                    dispatch_label,
                    lander_name,
                    variant_id,
                    current_attempt,
                );
                attempt_idx += 1;

                futures.push(async move {
                    let result = lander_clone
                        .submit_variant(variant_clone, deadline_copy, None)
                        .await;
                    (lander_name, current_attempt, variant_signature, result)
                });
            }

            while let Some((lander_name, attempt, signature, result)) = futures.next().await {
                match result {
                    Ok(receipt) => {
                        events::lander_success(strategy_name, dispatch_label, attempt, &receipt);
                        if let Some(sig) = receipt.signature.as_deref() {
                            info!(target: "lander::stack", signature = sig, "lander submission succeeded");
                        } else {
                            info!(target: "lander::stack", "lander submission succeeded");
                        }
                        return Ok(receipt);
                    }
                    Err(err) => {
                        events::lander_failure(
                            strategy_name,
                            dispatch_label,
                            lander_name,
                            variant_id,
                            attempt,
                            &err,
                        );
                        warn!(
                            target: "lander::stack",
                            tx_signature = signature.as_deref().unwrap_or(""),
                            "lander submission failed"
                        );
                        last_err = Some(err);
                    }
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all landers failed to submit transaction")))
    }

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
            for variant in plan.variants() {
                let variant_id = variant.id();
                let variant_signature = variant.signature();
                for lander in &self.landers {
                    let endpoints = lander.endpoints();
                    if endpoints.is_empty() {
                        if deadline.expired() {
                            return Err(LanderError::fatal("deadline expired before submission"));
                        }
                        events::lander_attempt(
                            strategy_name,
                            dispatch_label,
                            lander.name(),
                            variant_id,
                            attempt_idx,
                        );
                        match lander.submit_variant(variant.clone(), deadline, None).await {
                            Ok(receipt) => {
                                events::lander_success(
                                    strategy_name,
                                    dispatch_label,
                                    attempt_idx,
                                    &receipt,
                                );
                                if let Some(sig) = receipt.signature.as_deref() {
                                    info!(target: "lander::stack", signature = sig, "lander submission succeeded");
                                } else {
                                    info!(target: "lander::stack", "lander submission succeeded");
                                }
                                return Ok(receipt);
                            }
                            Err(err) => {
                                events::lander_failure(
                                    strategy_name,
                                    dispatch_label,
                                    lander.name(),
                                    variant_id,
                                    attempt_idx,
                                    &err,
                                );
                                warn!(
                                    target: "lander::stack",
                                    tx_signature = variant_signature
                                        .as_deref()
                                        .unwrap_or(""),
                                    "lander submission failed"
                                );
                                last_err = Some(err);
                            }
                        }
                        attempt_idx += 1;
                    } else {
                        for endpoint in endpoints {
                            if deadline.expired() {
                                return Err(LanderError::fatal(
                                    "deadline expired before submission",
                                ));
                            }
                            events::lander_attempt(
                                strategy_name,
                                dispatch_label,
                                lander.name(),
                                variant_id,
                                attempt_idx,
                            );
                            match lander
                                .submit_variant(variant.clone(), deadline, Some(&endpoint))
                                .await
                            {
                                Ok(receipt) => {
                                    events::lander_success(
                                        strategy_name,
                                        dispatch_label,
                                        attempt_idx,
                                        &receipt,
                                    );
                                    if let Some(sig) = receipt.signature.as_deref() {
                                        info!(target: "lander::stack", signature = sig, "lander submission succeeded");
                                    } else {
                                        info!(target: "lander::stack", "lander submission succeeded");
                                    }
                                    return Ok(receipt);
                                }
                                Err(err) => {
                                    events::lander_failure(
                                        strategy_name,
                                        dispatch_label,
                                        lander.name(),
                                        variant_id,
                                        attempt_idx,
                                        &err,
                                    );
                                    warn!(
                                        target: "lander::stack",
                                        endpoint = endpoint.as_str(),
                                        tx_signature = variant_signature
                                            .as_deref()
                                            .unwrap_or(""),
                                        "lander submission failed"
                                    );
                                    last_err = Some(err);
                                }
                            }
                            attempt_idx += 1;
                        }
                    }
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all landers failed to submit transaction")))
    }
}
