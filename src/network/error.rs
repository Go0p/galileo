#![allow(dead_code)]

use std::net::IpAddr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("网络接口枚举失败: {0}")]
    InterfaceDiscovery(#[source] std::io::Error),
    #[error("未找到可用的本地 IP")]
    NoEligibleIp,
    #[error("手动指定的 IP `{ip}` 无效: {reason}")]
    InvalidManualIp { ip: IpAddr, reason: &'static str },
    #[error("构建 IP 绑定客户端失败: {0}")]
    ClientPool(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;
