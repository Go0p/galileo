use std::time::Instant;

use tracing::{info, warn};

use crate::engine::PreparedTransaction;
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

    pub async fn submit(
        &self,
        prepared: &PreparedTransaction,
        deadline: Deadline,
    ) -> Result<LanderReceipt, LanderError> {
        match self {
            LanderVariant::Rpc(lander) => lander.submit(prepared, deadline).await,
            LanderVariant::Jito(lander) => lander.submit(prepared, deadline).await,
            LanderVariant::Staked(lander) => lander.submit(prepared, deadline).await,
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

    pub async fn submit(
        &self,
        prepared: &PreparedTransaction,
        deadline: Deadline,
        strategy: &str,
    ) -> Result<LanderReceipt, LanderError> {
        if self.landers.is_empty() {
            return Err(LanderError::fatal("no lander configured"));
        }

        let mut attempt_idx = 0usize;
        let total_passes = self.max_retries.saturating_add(1);
        let mut last_err = None;

        for _pass in 0..total_passes {
            for lander in &self.landers {
                if deadline.expired() {
                    let err = LanderError::fatal("deadline expired before submission");
                    events::lander_failure(strategy, lander.name(), attempt_idx, &err);
                    return Err(err);
                }

                events::lander_attempt(strategy, lander.name(), attempt_idx);
                match lander.submit(prepared, deadline).await {
                    Ok(receipt) => {
                        events::lander_success(strategy, attempt_idx, &receipt);
                        if let Some(signature) = receipt.signature.as_deref() {
                            info!(target: "lander::stack", signature, "lander submission succeeded");
                        } else {
                            info!(target: "lander::stack", "lander submission succeeded");
                        }
                        return Ok(receipt);
                    }
                    Err(err) => {
                        let tx_signature = prepared
                            .transaction
                            .signatures
                            .get(0)
                            .map(|sig| sig.to_string())
                            .unwrap_or_default();
                        events::lander_failure(strategy, lander.name(), attempt_idx, &err);
                        warn!(target: "lander::stack", tx_signature, "lander submission failed");
                        last_err = Some(err);
                    }
                }

                attempt_idx += 1;
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all landers failed to submit transaction")))
    }
}
