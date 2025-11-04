use anyhow::{Result, anyhow};
use futures::{Stream, TryStreamExt};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use yellowstone_grpc_proto::geyser::geyser_client::GeyserClient;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions, SubscribeUpdate,
    SubscribeUpdateTransactionInfo, subscribe_update,
};
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;
use yellowstone_grpc_proto::tonic::service::{Interceptor, interceptor::InterceptedService};
use yellowstone_grpc_proto::tonic::transport::{Channel, Endpoint};
use yellowstone_grpc_proto::tonic::{Request, Status};

pub struct YellowstoneTransactionClient {
    client: GeyserClient<InterceptedService<Channel, TokenInterceptor>>,
}

impl YellowstoneTransactionClient {
    pub async fn connect(endpoint: String, token: Option<AsciiMetadataValue>) -> Result<Self> {
        let endpoint = Endpoint::from_shared(endpoint)
            .map_err(|err| anyhow!("Yellowstone endpoint 无效: {err}"))?
            .tcp_nodelay(true);
        let channel = endpoint
            .connect()
            .await
            .map_err(|err| anyhow!("连接 Yellowstone gRPC 失败: {err}"))?;
        let interceptor = TokenInterceptor { token };
        Ok(Self {
            client: GeyserClient::with_interceptor(channel, interceptor),
        })
    }

    pub async fn subscribe_transactions(
        &mut self,
        wallet: Pubkey,
    ) -> Result<impl Stream<Item = Result<SubscribeUpdate, Status>>> {
        let mut request = SubscribeRequest::default();
        let mut tx_filter = SubscribeRequestFilterTransactions::default();
        tx_filter.account_required.push(wallet.to_string());
        request.commitment = Some(CommitmentLevel::Processed as i32);
        request.transactions.insert(wallet.to_string(), tx_filter);

        let (sender, receiver) = mpsc::channel(4);
        sender
            .send(request)
            .await
            .map_err(|_| anyhow!("发送订阅请求失败"))?;
        drop(sender);

        let response = self
            .client
            .subscribe(Request::new(ReceiverStream::new(receiver)))
            .await
            .map_err(|err| anyhow!("订阅 Yellowstone 失败: {err}"))?;

        Ok(response.into_inner().map_err(Into::into))
    }
}

#[derive(Clone)]
struct TokenInterceptor {
    token: Option<AsciiMetadataValue>,
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut request: Request<()>) -> std::result::Result<Request<()>, Status> {
        if let Some(token) = &self.token {
            request.metadata_mut().insert("x-token", token.clone());
        }
        Ok(request)
    }
}

pub fn parse_transaction_update(
    update: &SubscribeUpdate,
) -> Option<SubscribeUpdateTransactionInfo> {
    match &update.update_oneof {
        Some(subscribe_update::UpdateOneof::Transaction(tx)) => tx.transaction.clone(),
        _ => None,
    }
}
