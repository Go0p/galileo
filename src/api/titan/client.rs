#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use futures::{StreamExt, sink::SinkExt};
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_tungstenite::{
    WebSocketStream, connect_async_tls_with_config,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error, trace, warn};
use url::Url;

use crate::titan::error::TitanError;

use super::types::{
    ClientRequest, ListProvidersRequest, ProviderInfo, QuoteSwapStreamResponse, RequestData,
    RequestId, ResponseData, ResponseSuccess, ServerInfo, ServerMessage, StreamDataPayload,
    StreamEnd, StreamId, SwapQuoteRequest,
};

type PendingSender = oneshot::Sender<Result<ResponseSuccess, TitanError>>;
type PendingMap = HashMap<RequestId, PendingSender>;
type StreamSender = mpsc::Sender<QuoteStreamItem>;

const PROTOCOLS: [&str; 2] = ["v1.api.titan.ag+gzip", "v1.api.titan.ag"];
const USER_AGENT: &str = "galileo-bot/0.1";

/// Async WebSocket client for the Titan quote streaming API.
///
/// Typical usage:
///
/// ```ignore
/// use galileo::titan::{QuoteStreamItem, TitanWsClient};
/// use galileo::titan::types::{SwapParams, SwapQuoteRequest, TransactionParams};
/// use solana_sdk::pubkey::Pubkey;
/// use std::str::FromStr;
/// use url::Url;
///
/// # async fn demo() -> anyhow::Result<()> {
/// let endpoint = Url::parse("wss://api.titan.exchange/api/v1/ws?auth=...JWT...")?;
/// let client = TitanWsClient::connect(endpoint).await?;
/// let request = SwapQuoteRequest {
///     swap: SwapParams {
///         input_mint: Pubkey::from_str("So11111111111111111111111111111111111111112")?,
///         output_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qTz6fG2CjN7gx9ruJj8VH7P5w")?,
///         amount: 1_000_000_000,
///         swap_mode: None,
///         slippage_bps: Some(50),
///         dexes: None,
///         exclude_dexes: None,
///         only_direct_routes: None,
///         add_size_constraint: None,
///         size_constraint: None,
///         providers: None,
///         accounts_limit_total: None,
///         accounts_limit_writable: None,
///     },
///     transaction: TransactionParams {
///         user_public_key: Pubkey::from_str("Titan11111111111111111111111111111111111111")?,
///         close_input_token_account: None,
///         create_output_token_account: None,
///         fee_account: None,
///         fee_bps: None,
///         fee_from_input_mint: None,
///         output_account: None,
///     },
///     update: None,
/// };
/// let session = client.subscribe_swap_quotes(request).await?;
/// let mut stream = session.receiver;
/// while let Some(event) = stream.recv().await {
///     match event {
///         QuoteStreamItem::Update { seq, quotes } => {
///             println!("received update #{seq}: {} providers", quotes.quotes.len());
///         }
///         QuoteStreamItem::End(_) => break,
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct TitanWsClient {
    inner: Arc<TitanInner>,
    reader_handle: JoinHandle<()>,
}

#[derive(Debug)]
pub struct QuoteStreamSession {
    pub info: QuoteSwapStreamResponse,
    pub stream_id: StreamId,
    pub receiver: mpsc::Receiver<QuoteStreamItem>,
}

struct TitanInner {
    sink: Mutex<futures::stream::SplitSink<WebSocketStream<ConnectorStream>, Message>>,
    pending: Mutex<PendingMap>,
    streams: Mutex<HashMap<StreamId, StreamSender>>,
    next_request_id: tokio::sync::Mutex<RequestId>,
    use_gzip: bool,
}

type ConnectorStream = tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>;

impl TitanWsClient {
    pub async fn connect(endpoint: Url) -> Result<Self, TitanError> {
        let mut request = endpoint
            .as_str()
            .into_client_request()
            .map_err(|err| TitanError::Handshake(format!("failed to build request: {err}")))?;

        let protocols = PROTOCOLS.join(", ");
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str(&protocols).map_err(|err| {
                TitanError::Handshake(format!("invalid Sec-WebSocket-Protocol header: {err}"))
            })?,
        );
        request
            .headers_mut()
            .insert("User-Agent", HeaderValue::from_static(USER_AGENT));

        let (stream, response) = connect_async_tls_with_config(request, None, false, None)
            .await
            .map_err(TitanError::Transport)?;

        let protocol = response
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let use_gzip = protocol.contains("+gzip");

        let (sink, reader) = stream.split();

        let inner = Arc::new(TitanInner {
            sink: Mutex::new(sink),
            pending: Mutex::new(HashMap::new()),
            streams: Mutex::new(HashMap::new()),
            next_request_id: tokio::sync::Mutex::new(1),
            use_gzip,
        });

        let reader_handle = {
            let inner_clone = inner.clone();
            tokio::spawn(async move {
                if let Err(err) = reader_loop(reader, inner_clone).await {
                    warn!(target: "titan", "Titan reader loop terminated: {err:?}");
                }
            })
        };

        Ok(Self {
            inner,
            reader_handle,
        })
    }

    pub async fn get_info(&self) -> Result<ServerInfo, TitanError> {
        let response = self
            .send_request(RequestData::GetInfo(Default::default()))
            .await?;
        match response.data {
            ResponseData::GetInfo(info) => Ok(info),
            other => Err(TitanError::Protocol(format!(
                "unexpected response to GetInfo: {other:?}"
            ))),
        }
    }

    pub async fn list_providers(&self) -> Result<Vec<ProviderInfo>, TitanError> {
        let request = ListProvidersRequest {
            include_icons: Some(true),
        };
        let response = self
            .send_request(RequestData::ListProviders(request))
            .await?;
        match response.data {
            ResponseData::ListProviders(providers) => Ok(providers),
            other => Err(TitanError::Protocol(format!(
                "unexpected response to ListProviders: {other:?}"
            ))),
        }
    }

    pub async fn subscribe_swap_quotes(
        &self,
        request: SwapQuoteRequest,
    ) -> Result<QuoteStreamSession, TitanError> {
        let (tx, rx) = mpsc::channel(64);
        let response = self
            .send_stream_request(RequestData::NewSwapQuoteStream(request), tx)
            .await?;
        let stream_id = response
            .stream
            .as_ref()
            .map(|start| start.id)
            .ok_or_else(|| TitanError::Protocol("missing stream metadata in response".into()))?;
        if let ResponseData::NewSwapQuoteStream(info) = response.data {
            Ok(QuoteStreamSession {
                info,
                stream_id,
                receiver: rx,
            })
        } else {
            Err(TitanError::Protocol(
                "unexpected response payload for NewSwapQuoteStream".into(),
            ))
        }
    }

    pub async fn stop_stream(&self, stream_id: StreamId) -> Result<(), TitanError> {
        let request = RequestData::StopStream(super::types::StopStreamRequest { id: stream_id });
        let response = self.send_request(request).await?;
        match response.data {
            ResponseData::StreamStopped(stopped) if stopped.id == stream_id => Ok(()),
            ResponseData::StreamStopped(_) => Err(TitanError::Protocol(
                "StopStream response did not match requested stream".into(),
            )),
            other => Err(TitanError::Protocol(format!(
                "unexpected StopStream response payload: {other:?}"
            ))),
        }
    }

    async fn send_request(&self, data: RequestData) -> Result<ResponseSuccess, TitanError> {
        let (id, payload) = self.prepare_request(data).await?;
        let (tx, rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, tx);
        self.inner.send_message(payload).await?;

        let response = rx.await.map_err(|_| TitanError::ConnectionClosed)??;
        Ok(response)
    }

    async fn send_stream_request(
        &self,
        data: RequestData,
        sender: StreamSender,
    ) -> Result<ResponseSuccess, TitanError> {
        let (id, payload) = self.prepare_request(data).await?;
        let (tx, rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, tx);
        self.inner.send_message(payload).await?;

        let response = rx.await.map_err(|_| TitanError::ConnectionClosed)??;

        if let Some(stream) = &response.stream {
            if !matches!(stream.data_type, super::types::StreamDataType::SwapQuotes) {
                return Err(TitanError::Protocol(format!(
                    "unsupported stream type: {:?}",
                    stream.data_type
                )));
            }
            self.inner.streams.lock().await.insert(stream.id, sender);
        }

        Ok(response)
    }

    async fn prepare_request(&self, data: RequestData) -> Result<(RequestId, Vec<u8>), TitanError> {
        let mut guard = self.inner.next_request_id.lock().await;
        let id = *guard;
        *guard = guard.wrapping_add(1).max(1);
        drop(guard);

        let envelope = ClientRequest { id, data };
        let mut encoded = rmp_serde::to_vec_named(&envelope)?;
        if self.inner.use_gzip {
            encoded = gzip_encode(encoded)?;
        }
        Ok((id, encoded))
    }
}

impl TitanInner {
    async fn send_message(&self, payload: Vec<u8>) -> Result<(), TitanError> {
        let mut sink = self.sink.lock().await;
        sink.send(Message::Binary(payload.into()))
            .await
            .map_err(TitanError::Transport)
    }
}

impl Drop for TitanWsClient {
    fn drop(&mut self) {
        self.reader_handle.abort();
    }
}

async fn reader_loop(
    mut reader: futures::stream::SplitStream<WebSocketStream<ConnectorStream>>,
    inner: Arc<TitanInner>,
) -> Result<(), TitanError> {
    while let Some(message) = reader.next().await {
        match message {
            Ok(Message::Binary(data)) => {
                if let Err(err) = handle_binary_message(&inner, &data).await {
                    error!(target = "titan", "Failed to handle Titan message: {err:?}");
                }
            }
            Ok(Message::Close(frame)) => {
                warn!(target = "titan", "Titan WebSocket closed: {:?}", frame);
                break;
            }
            Ok(Message::Ping(payload)) => {
                let mut sink = inner.sink.lock().await;
                if let Err(err) = sink.send(Message::Pong(payload)).await {
                    error!(target = "titan", "Failed to send pong: {err}");
                }
            }
            Ok(Message::Pong(_)) => {
                trace!(target = "titan", "Received pong from Titan server");
            }
            Ok(Message::Frame(_)) => {
                trace!(target = "titan", "Ignoring raw frame message");
            }
            Ok(Message::Text(text)) => {
                debug!(target = "titan", "Ignoring unexpected text message: {text}");
            }
            Err(err) => {
                return Err(TitanError::Transport(err));
            }
        }
    }
    Ok(())
}

async fn handle_binary_message(inner: &Arc<TitanInner>, data: &[u8]) -> Result<(), TitanError> {
    let decoded = if inner.use_gzip {
        gzip_decode(data)?
    } else {
        data.to_vec()
    };

    let message: ServerMessage = rmp_serde::from_slice(&decoded)?;
    match message {
        ServerMessage::Response(response) => {
            let sender = inner.pending.lock().await.remove(&response.request_id);
            if let Some(sender) = sender {
                let _ = sender.send(Ok(response));
            } else {
                warn!(target = "titan", "Received response for unknown request id");
            }
        }
        ServerMessage::Error(err) => {
            let sender = inner.pending.lock().await.remove(&err.request_id);
            if let Some(sender) = sender {
                let _ = sender.send(Err(TitanError::Response {
                    code: err.code,
                    message: err.message,
                }));
            } else {
                warn!(target = "titan", "Received error for unknown request id");
            }
        }
        ServerMessage::StreamData(stream_data) => {
            let StreamDataPayload::SwapQuotes(quotes) = stream_data.payload;
            let mut streams = inner.streams.lock().await;
            if let Some(sender) = streams.get(&stream_data.id) {
                let event = QuoteStreamItem::Update {
                    seq: stream_data.seq,
                    quotes,
                };
                if sender.send(event).await.is_err() {
                    streams.remove(&stream_data.id);
                }
            } else {
                warn!(target = "titan", "Dropping data for unknown stream id");
            }
        }
        ServerMessage::StreamEnd(end) => {
            let mut streams = inner.streams.lock().await;
            if let Some(sender) = streams.remove(&end.id) {
                let event = QuoteStreamItem::End(end.clone());
                let _ = sender.send(event).await;
            }
        }
    }
    Ok(())
}

fn gzip_encode(data: Vec<u8>) -> Result<Vec<u8>, TitanError> {
    if data.is_empty() {
        return Ok(data);
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data)?;
    Ok(encoder.finish()?)
}

fn gzip_decode(data: &[u8]) -> Result<Vec<u8>, TitanError> {
    let mut decoder = GzDecoder::new(data);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[derive(Debug, Clone)]
pub enum QuoteStreamItem {
    Update {
        seq: u64,
        quotes: super::types::SwapQuotes,
    },
    End(StreamEnd),
}
