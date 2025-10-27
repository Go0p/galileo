use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

#[cfg(test)]
use std::collections::VecDeque;
#[cfg(test)]
use std::sync::Mutex;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use tracing::{info, warn};

use crate::engine::{DispatchPlan, DispatchStrategy, TxVariant, VariantId};
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
            #[cfg(test)]
            LanderVariant::Test(lander) => {
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

                let variant = match plan.variants_for_lander(lander_idx).first() {
                    Some(variant) => variant.clone(),
                    None => continue,
                };

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
                    let (local_ip, result) = submit_with_lease(
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
                        result,
                    )
                });
            }

            while let Some((lander_name, attempt, variant_id, signature, local_ip, result)) =
                futures.next().await
            {
                match result {
                    Ok(mut receipt) => {
                        if receipt.local_ip.is_none() {
                            receipt.local_ip = local_ip;
                        }
                        events::lander_success(strategy_name, dispatch_label, attempt, &receipt);
                        if let Some(sig) = receipt.signature.as_deref() {
                            info!(
                                target: "lander::stack",
                                signature = sig,
                                "lander submission succeeded"
                            );
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
                            local_ip,
                            &err,
                        );
                        warn!(
                            target: "lander::stack",
                            lander = lander_name,
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
                        let (local_ip, result) = submit_with_lease(
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
                result,
            )) = futures.next().await
            {
                match result {
                    Ok(mut receipt) => {
                        if receipt.local_ip.is_none() {
                            receipt.local_ip = local_ip;
                        }
                        events::lander_success(strategy_name, dispatch_label, attempt, &receipt);
                        if let Some(sig) = receipt.signature.as_deref() {
                            info!(
                                target: "lander::stack",
                                signature = sig,
                                "lander submission succeeded"
                            );
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
                            local_ip,
                            &err,
                        );
                        match endpoint {
                            Some(endpoint_name) => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
                                    endpoint = endpoint_name,
                                    tx_signature = signature.as_deref().unwrap_or(""),
                                    "lander submission failed"
                                );
                            }
                            None => {
                                warn!(
                                    target: "lander::stack",
                                    lander = lander_name,
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
) -> (Option<IpAddr>, Result<LanderReceipt, LanderError>) {
    let endpoint_hash = compute_endpoint_hash(endpoint.as_deref(), variant_id);

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
            return (None, Err(failure));
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
    }

    (local_ip, result)
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
        local_ip: Option<IpAddr>,
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
                    local_ip,
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
    use crate::network::{CooldownConfig, IpAllocator, IpInventory, IpInventoryConfig};
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

        TxVariant::new(id, transaction, Hash::default(), 0, None, signer, 0)
    }

    fn test_ip_allocator() -> Arc<IpAllocator> {
        let config = IpInventoryConfig {
            enable_multiple_ip: false,
            manual_ips: vec![std::net::IpAddr::from([127, 0, 0, 1])],
            blacklist: Vec::new(),
            allow_loopback: true,
        };
        let inventory = IpInventory::new(config).expect("build inventory");
        Arc::new(IpAllocator::from_inventory(
            inventory,
            Some(2),
            CooldownConfig::default(),
        ))
    }

    #[tokio::test]
    async fn all_at_once_dispatches_every_endpoint() {
        let variant = build_variant(0);
        let plan = DispatchPlan::new(DispatchStrategy::AllAtOnce, vec![vec![variant.clone()]]);
        let outcomes = vec![
            TestOutcome::Failure,
            TestOutcome::Success {
                signature: Some("sig-endpoint-b".to_string()),
            },
        ];
        let test_lander = TestLander::new("test", vec!["https://a", "https://b"], outcomes);
        let stack = LanderStack::new(
            vec![LanderVariant::Test(test_lander.clone())],
            0,
            test_ip_allocator(),
        );

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
        let plan = DispatchPlan::new(
            DispatchStrategy::OneByOne,
            vec![vec![variant.clone()], vec![variant.clone()]],
        );
        let outcomes = vec![
            TestOutcome::Failure,
            TestOutcome::Success { signature: None },
        ];
        let test_lander = TestLander::new("test", vec!["endpoint-a", "endpoint-b"], outcomes);
        let stack = LanderStack::new(
            vec![LanderVariant::Test(test_lander.clone())],
            0,
            test_ip_allocator(),
        );

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
