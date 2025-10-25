use std::sync::Arc;
use std::time::Duration;

use solana_sdk::hash::Hash;
use std::str::FromStr;
use thiserror::Error;
use tokio::sync::Mutex;
use yellowstone_grpc_proto::geyser::geyser_client::GeyserClient;
use yellowstone_grpc_proto::geyser::{CommitmentLevel, GetLatestBlockhashRequest};
use yellowstone_grpc_proto::tonic::metadata::{AsciiMetadataValue, errors::InvalidMetadataValue};
use yellowstone_grpc_proto::tonic::service::{Interceptor, interceptor::InterceptedService};
use yellowstone_grpc_proto::tonic::transport::{Channel, Endpoint, Error as TransportError};
use yellowstone_grpc_proto::tonic::{Request, Status};

use super::BlockhashSnapshot;

#[derive(Clone)]
/// 基于 Yellowstone gRPC 的区块哈希获取器。
pub struct YellowstoneBlockhashClient {
    inner: Arc<YellowstoneInner>,
}

struct YellowstoneInner {
    endpoint: Arc<str>,
    token: Option<AsciiMetadataValue>,
    commitment: CommitmentLevel,
    client: Mutex<Option<GeyserClient<InterceptedChannel>>>,
}

type InterceptedChannel = InterceptedService<Channel, TokenInterceptor>;

#[derive(Clone)]
struct TokenInterceptor {
    token: Option<AsciiMetadataValue>,
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(token) = &self.token {
            request.metadata_mut().insert("x-token", token.clone());
        }
        Ok(request)
    }
}

impl YellowstoneBlockhashClient {
    /// 创建 gRPC 区块哈希客户端。
    pub fn new(
        endpoint: String,
        token: Option<String>,
        commitment: CommitmentLevel,
    ) -> Result<Self, YellowstoneError> {
        let token = match token {
            Some(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(
                        trimmed
                            .parse::<AsciiMetadataValue>()
                            .map_err(YellowstoneError::InvalidToken)?,
                    )
                }
            }
            None => None,
        };

        Ok(Self {
            inner: Arc::new(YellowstoneInner {
                endpoint: Arc::from(endpoint),
                token,
                commitment,
                client: Mutex::new(None),
            }),
        })
    }

    /// 获取最新区块哈希。
    pub async fn latest_blockhash(&self) -> Result<BlockhashSnapshot, YellowstoneError> {
        let mut guard = self.inner.client.lock().await;
        if guard.is_none() {
            let client = self.inner.connect().await?;
            *guard = Some(client);
        }
        let client = guard.as_mut().expect("client initialized");
        let request = GetLatestBlockhashRequest {
            commitment: Some(self.inner.commitment as i32),
        };
        let response = client
            .get_latest_blockhash(request)
            .await
            .map_err(YellowstoneError::Grpc)?
            .into_inner();
        let blockhash =
            Hash::from_str(&response.blockhash).map_err(YellowstoneError::ParseBlockhash)?;
        Ok(BlockhashSnapshot {
            blockhash,
            slot: Some(response.slot),
            last_valid_block_height: Some(response.last_valid_block_height),
        })
    }
}

impl YellowstoneInner {
    async fn connect(&self) -> Result<GeyserClient<InterceptedChannel>, YellowstoneError> {
        let endpoint = Endpoint::from_shared(self.endpoint.to_string())
            .map_err(YellowstoneError::InvalidEndpoint)?
            .connect_timeout(Duration::from_secs(5))
            .tcp_nodelay(true);
        let channel = endpoint
            .connect()
            .await
            .map_err(YellowstoneError::Connect)?;
        let interceptor = TokenInterceptor {
            token: self.token.clone(),
        };
        Ok(GeyserClient::with_interceptor(channel, interceptor))
    }
}

#[derive(Debug, Error)]
pub enum YellowstoneError {
    #[error("Yellowstone endpoint 无效: {0}")]
    InvalidEndpoint(TransportError),
    #[error("连接 Yellowstone gRPC 失败: {0}")]
    Connect(TransportError),
    #[error("gRPC 调用失败: {0}")]
    Grpc(Status),
    #[error("x-token 无法解析: {0}")]
    InvalidToken(InvalidMetadataValue),
    #[error("区块哈希解析失败: {0}")]
    ParseBlockhash(solana_sdk::hash::ParseHashError),
}
