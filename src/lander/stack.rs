use std::time::Instant;

#[cfg(test)]
use std::collections::VecDeque;
#[cfg(test)]
use std::sync::{Arc, Mutex};

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
    #[cfg(test)]
    Test(TestLander),
}

impl LanderVariant {
    pub fn name(&self) -> &'static str {
        match self {
            LanderVariant::Rpc(_) => "rpc",
            LanderVariant::Jito(_) => "jito",
            LanderVariant::Staked(_) => "staked",
            #[cfg(test)]
            LanderVariant::Test(lander) => lander.name(),
        }
    }

    pub fn one_by_one_capacity(&self) -> usize {
        match self {
            LanderVariant::Rpc(_) => 1,
            LanderVariant::Jito(lander) => lander.endpoints().max(1),
            LanderVariant::Staked(lander) => lander.endpoints_len().max(1),
            #[cfg(test)]
            LanderVariant::Test(lander) => lander.one_by_one_capacity(),
        }
    }

    pub fn endpoints(&self) -> Vec<String> {
        match self {
            LanderVariant::Rpc(_) => Vec::new(),
            LanderVariant::Jito(lander) => lander.endpoint_list(),
            LanderVariant::Staked(lander) => lander.endpoint_list(),
            #[cfg(test)]
            LanderVariant::Test(lander) => lander.endpoint_list(),
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
            #[cfg(test)]
            LanderVariant::Test(lander) => lander.submit_variant(variant, deadline, endpoint).await,
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

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
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
                let endpoints = lander.endpoints();
                let targets: Vec<Option<String>> = if endpoints.is_empty() {
                    vec![None]
                } else {
                    endpoints.into_iter().map(Some).collect()
                };

                for endpoint in targets {
                    if deadline.expired() {
                        return Err(LanderError::fatal("deadline expired before submission"));
                    }

                    let lander_clone = lander.clone();
                    let variant_clone = variant.clone();
                    let variant_signature = variant_clone.signature();
                    let lander_name = lander.name();
                    let current_attempt = attempt_idx;
                    let deadline_copy = deadline;
                    let endpoint_label = endpoint.clone();

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
                            .submit_variant(variant_clone, deadline_copy, endpoint_label.as_deref())
                            .await;
                        (
                            lander_name,
                            endpoint_label,
                            current_attempt,
                            variant_signature,
                            result,
                        )
                    });
                }
            }

            while let Some((lander_name, endpoint, attempt, signature, result)) =
                futures.next().await
            {
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
                        match endpoint {
                            Some(ref endpoint_name) => {
                                warn!(
                                    target: "lander::stack",
                                    endpoint = endpoint_name.as_str(),
                                    tx_signature = signature.as_deref().unwrap_or(""),
                                    "lander submission failed"
                                );
                            }
                            None => {
                                warn!(
                                    target: "lander::stack",
                                    tx_signature = signature.as_deref().unwrap_or(""),
                                    "lander submission failed"
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

#[cfg(test)]
#[derive(Clone)]
pub enum TestOutcome {
    Success { signature: Option<String> },
    Failure,
}

#[cfg(test)]
#[derive(Clone)]
pub struct TestLander {
    name: &'static str,
    endpoints: Vec<String>,
    submissions: Arc<Mutex<Vec<Option<String>>>>,
    outcomes: Arc<Mutex<VecDeque<TestOutcome>>>,
}

#[cfg(test)]
impl TestLander {
    pub fn new(name: &'static str, endpoints: Vec<&str>, outcomes: Vec<TestOutcome>) -> Self {
        Self {
            name,
            endpoints: endpoints
                .into_iter()
                .map(|endpoint| endpoint.to_string())
                .collect(),
            submissions: Arc::new(Mutex::new(Vec::new())),
            outcomes: Arc::new(Mutex::new(VecDeque::from(outcomes))),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn one_by_one_capacity(&self) -> usize {
        self.endpoints.len().max(1)
    }

    pub fn endpoint_list(&self) -> Vec<String> {
        self.endpoints.clone()
    }

    pub fn recorded_endpoints(&self) -> Vec<Option<String>> {
        self.submissions.lock().unwrap().clone()
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        _deadline: Deadline,
        endpoint: Option<&str>,
    ) -> Result<LanderReceipt, LanderError> {
        let recorded = endpoint.map(|value| value.to_string());
        self.submissions.lock().unwrap().push(recorded.clone());

        let outcome = self
            .outcomes
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(TestOutcome::Failure);

        match outcome {
            TestOutcome::Success { signature } => {
                let endpoint_string = recorded.unwrap_or_default();
                Ok(LanderReceipt {
                    lander: self.name,
                    endpoint: endpoint_string,
                    slot: variant.slot(),
                    blockhash: variant.blockhash().to_string(),
                    signature,
                    variant_id: variant.id(),
                })
            }
            TestOutcome::Failure => Err(LanderError::fatal("test failure")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{DispatchPlan, DispatchStrategy, VariantId};
    use solana_sdk::hash::Hash;
    use solana_sdk::message::{Message, VersionedMessage};
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::transaction::VersionedTransaction;
    use std::time::{Duration, Instant};

    fn build_variant(id: VariantId) -> TxVariant {
        let signer = Arc::new(Keypair::new());
        let payer = signer.pubkey();
        let message = Message::new(&[], Some(&payer));
        let versioned = VersionedMessage::Legacy(message);
        let transaction =
            VersionedTransaction::try_new(versioned, &[signer.as_ref()]).expect("build tx");

        TxVariant::new(id, transaction, Hash::default(), 0, signer, 0)
    }

    #[tokio::test]
    async fn all_at_once_dispatches_every_endpoint() {
        let variant = build_variant(0);
        let plan = DispatchPlan::new(DispatchStrategy::AllAtOnce, vec![variant.clone()]);
        let outcomes = vec![
            TestOutcome::Failure,
            TestOutcome::Success {
                signature: Some("sig-endpoint-b".to_string()),
            },
        ];
        let test_lander = TestLander::new("test", vec!["https://a", "https://b"], outcomes);
        let stack = LanderStack::new(vec![LanderVariant::Test(test_lander.clone())], 0);

        let deadline = Deadline::from_instant(Instant::now() + Duration::from_millis(50));
        let receipt = stack
            .submit_plan(&plan, deadline, "unit-test")
            .await
            .expect("all_at_once succeeds");

        assert_eq!(receipt.endpoint, "https://b");
        assert_eq!(receipt.signature.as_deref(), Some("sig-endpoint-b"));

        let attempts = test_lander.recorded_endpoints();
        assert_eq!(attempts.len(), 2);
        assert!(attempts.contains(&Some("https://a".to_string())));
        assert!(attempts.contains(&Some("https://b".to_string())));
    }

    #[tokio::test]
    async fn one_by_one_walks_endpoints_in_order() {
        let variant = build_variant(1);
        let plan = DispatchPlan::new(DispatchStrategy::OneByOne, vec![variant.clone()]);
        let outcomes = vec![
            TestOutcome::Failure,
            TestOutcome::Success { signature: None },
        ];
        let test_lander = TestLander::new("test", vec!["endpoint-a", "endpoint-b"], outcomes);
        let stack = LanderStack::new(vec![LanderVariant::Test(test_lander.clone())], 0);

        let deadline = Deadline::from_instant(Instant::now() + Duration::from_millis(50));
        let receipt = stack
            .submit_plan(&plan, deadline, "unit-test")
            .await
            .expect("one_by_one succeeds");

        assert_eq!(receipt.endpoint, "endpoint-b");
        assert_eq!(receipt.signature, None);

        let attempts = test_lander.recorded_endpoints();
        assert_eq!(
            attempts,
            vec![
                Some("endpoint-a".to_string()),
                Some("endpoint-b".to_string())
            ]
        );
    }
}
