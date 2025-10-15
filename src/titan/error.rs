#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TitanError {
    #[error("Titan WebSocket endpoint not configured")]
    MissingEndpoint,
    #[error("Titan authentication token missing")]
    MissingAuthToken,
    #[error("Titan WebSocket handshake failed: {0}")]
    Handshake(String),
    #[error("Titan WebSocket transport error: {0}")]
    Transport(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Titan WebSocket stream closed unexpectedly")]
    ConnectionClosed,
    #[error("Titan message serialization error: {0}")]
    Serialization(#[from] rmp_serde::decode::Error),
    #[error("Titan message encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("Titan gzip error: {0}")]
    Gzip(#[from] std::io::Error),
    #[error("Titan response error: code={code} message={message}")]
    Response { code: i32, message: String },
    #[error("Titan stream error: {message} (code {code:?})")]
    Stream { code: Option<i32>, message: String },
    #[error("Titan timeout waiting for response")]
    Timeout,
    #[error("Titan protocol violation: {0}")]
    Protocol(String),
}
