use solana_client::client_error::ClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlashloanError {
    #[error("闪电贷配置缺失: {0}")]
    InvalidConfig(&'static str),
    #[error("闪电贷配置非法: {0}")]
    InvalidConfigDetail(String),
    #[error("不支持的闪电贷资产: {0}")]
    UnsupportedAsset(String),
    #[error("RPC 请求失败: {0}")]
    Rpc(#[from] ClientError),
}

pub type FlashloanResult<T> = Result<T, FlashloanError>;
