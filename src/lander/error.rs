use std::fmt;

use bincode::error::EncodeError;
use reqwest::Error as ReqwestError;
use solana_client::client_error::ClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LanderError {
    #[error("RPC 提交失败: {0}")]
    Rpc(#[from] ClientError),
    #[error("网络请求失败: {0}")]
    Network(#[from] ReqwestError),
    #[error("JSON 解析失败: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("序列化交易失败: {0}")]
    Encode(#[from] EncodeError),
    #[error("{0}")]
    Fatal(String),
}

impl LanderError {
    pub fn fatal(reason: impl fmt::Display) -> Self {
        Self::Fatal(reason.to_string())
    }
}
