use std::num::ParseIntError;

use anyhow::Error;
use reqwest::Error as ReqwestError;
use solana_client::client_error::ClientError;
use thiserror::Error;

use crate::api::dflow::DflowError;
use crate::api::kamino::KaminoError;
use crate::api::ultra::UltraError;
use crate::engine::plugins::flashloan::FlashloanError;
use crate::network::NetworkError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("配置缺失或非法: {0}")]
    InvalidConfig(String),
    #[error("数值解析失败: {0}")]
    ParseAmount(#[from] ParseIntError),
    #[error("DFlow API 错误: {0}")]
    Dflow(#[from] DflowError),
    #[error("Kamino API 错误: {0}")]
    Kamino(#[from] KaminoError),
    #[error("Ultra API 错误: {0}")]
    Ultra(#[from] UltraError),
    #[error("JSON 处理失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("RPC 请求失败: {0}")]
    Rpc(#[from] ClientError),
    #[error("IP 资源分配失败: {0}")]
    NetworkResource(#[from] NetworkError),
    #[error("网络请求失败: {0}")]
    Network(#[from] ReqwestError),
    #[error("闪电贷处理失败: {0}")]
    Flashloan(#[from] FlashloanError),
    #[error("交易构建失败: {0}")]
    Transaction(#[from] Error),
    #[error("落地失败: {0}")]
    Landing(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
