use std::num::ParseIntError;

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
}

pub type StrategyResult<T> = Result<T, StrategyError>;
