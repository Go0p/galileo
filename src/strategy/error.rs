use std::num::ParseIntError;

use anyhow::Error;
use solana_client::client_error::ClientError;
use thiserror::Error;

use crate::jupiter::error::JupiterError;

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("strategy disabled")]
    Disabled,
    #[error("strategy configuration missing required field: {0}")]
    InvalidConfig(String),
    #[error("failed to parse amount: {0}")]
    ParseAmount(#[from] ParseIntError),
    #[error("jupiter api error: {0}")]
    Jupiter(#[from] JupiterError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("rpc error: {0}")]
    Rpc(#[from] ClientError),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("transaction error: {0}")]
    Transaction(#[from] Error),
    #[error("bundle submission failed: {0}")]
    Bundle(String),
}

pub type StrategyResult<T> = Result<T, StrategyError>;
