use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

const X_CLIENT_TIMESTAMP: HeaderName = HeaderName::from_static("x-client-timestamp");
const X_CLIENT_REQUEST_ID: HeaderName = HeaderName::from_static("x-client-request-id");

const STATIC_HEADERS: &[(&str, &str)] = &[
    ("accept", "*/*"),
    ("accept-language", "zh-CN,zh;q=0.9"),
    ("origin", "https://dflow.net"),
    ("referer", "https://dflow.net/"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
    ("sec-fetch-site", "cross-site"),
    (
        "sec-ch-ua",
        r#""Google Chrome";v="141", "Not?A_Brand";v="8", "Chromium";v="141""#,
    ),
    ("sec-ch-ua-platform", r#""Windows""#),
    ("sec-ch-ua-mobile", "?0"),
    ("priority", "u=1, i"),
    (
        "user-agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
    ),
];

#[derive(Debug, Error)]
pub(super) enum XClientHeaderError {
    #[error("system time before UNIX epoch: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct XClientHeaderValues {
    pub timestamp: String,
    pub request_id: String,
}

pub(super) fn build_header_map(path: &str, body: &str) -> Result<HeaderMap, XClientHeaderError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let timestamp_ms = duration.as_millis() as u64;
    let uuid = Uuid::new_v4().to_string();
    build_header_map_with_parts(timestamp_ms, &uuid, path, body)
}

fn build_header_map_with_parts(
    timestamp_ms: u64,
    uuid: &str,
    path: &str,
    body: &str,
) -> Result<HeaderMap, XClientHeaderError> {
    let values = compute_values(timestamp_ms, uuid, path, body);
    let mut headers = HeaderMap::with_capacity(2 + STATIC_HEADERS.len());
    headers.insert(
        X_CLIENT_TIMESTAMP,
        HeaderValue::from_str(&values.timestamp)?,
    );
    headers.insert(
        X_CLIENT_REQUEST_ID,
        HeaderValue::from_str(&values.request_id)?,
    );
    append_static_headers(&mut headers);
    Ok(headers)
}

fn compute_values(timestamp_ms: u64, uuid: &str, path: &str, body: &str) -> XClientHeaderValues {
    let payload = format!("{path}5_{body}k");
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hasher.update(timestamp_ms.to_le_bytes());
    let digest = hasher.finalize();

    let mut digest_hex = String::with_capacity(30);
    for byte in digest.iter().take(15) {
        use std::fmt::Write;
        let _ = write!(&mut digest_hex, "{:02x}", byte);
    }

    debug_assert!(digest_hex.len() >= 30, "digest length must be at least 30");

    let uuid_chars: Vec<char> = uuid.chars().collect();
    debug_assert!(
        uuid_chars.len() > 19,
        "UUID string must have at least 20 characters"
    );

    let part1 = &digest_hex[0..8];
    let part2 = &digest_hex[8..12];
    let part3 = format!("{}{}", uuid_chars[14], &digest_hex[12..15]);
    let part4 = format!("{}{}", uuid_chars[19], &digest_hex[15..18]);
    let part5 = &digest_hex[18..30];

    let request_id = format!("{part1}-{part2}-{part3}-{part4}-{part5}").to_ascii_lowercase();
    let timestamp = timestamp_ms.to_string();

    XClientHeaderValues {
        timestamp,
        request_id,
    }
}

fn append_static_headers(headers: &mut HeaderMap) {
    for (name, value) in STATIC_HEADERS.iter().copied() {
        headers.insert(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
}

#[cfg(test)]
pub(super) fn compute_values_for_test(
    timestamp_ms: u64,
    uuid: &str,
    path: &str,
    body: &str,
) -> XClientHeaderValues {
    compute_values(timestamp_ms, uuid, path, body)
}

#[cfg(test)]
mod tests {
    use super::{build_header_map_with_parts, compute_values_for_test};

    #[test]
    fn generates_expected_headers_for_get() {
        let timestamp_ms = 1_708_000_000_123_u64;
        let uuid = "12345678-1234-5678-9abc-def012345678";
        let path = "/quote?foo=bar";
        let body = "";

        let values = compute_values_for_test(timestamp_ms, uuid, path, body);
        assert_eq!(values.timestamp, timestamp_ms.to_string());
        assert_eq!(values.request_id, "ccc5e450-5682-5890-96fc-8ecce499ddf7");
    }

    #[test]
    fn generates_expected_headers_for_post() {
        let timestamp_ms = 1_708_000_000_456_u64;
        let uuid = "123e4567-e89b-12d3-a456-426614174000";
        let path = "/swap-instructions";
        let body = "{\"foo\":\"bar\"}";

        let values = compute_values_for_test(timestamp_ms, uuid, path, body);
        assert_eq!(values.timestamp, timestamp_ms.to_string());
        assert_eq!(values.request_id, "3a77738f-6550-1d5c-ad91-e22de7164188");
    }

    #[test]
    fn appends_static_headers() {
        let headers = build_header_map_with_parts(
            1_708_000_000_123_u64,
            "12345678-1234-5678-9abc-def012345678",
            "/quote",
            "",
        )
        .expect("build header map");

        assert_eq!(
            headers.get("sec-fetch-mode").and_then(|v| v.to_str().ok()),
            Some("cors")
        );
        assert_eq!(
            headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.contains("Chrome/141.0.0.0")),
            Some(true)
        );
        assert_eq!(
            headers.get("origin").and_then(|v| v.to_str().ok()),
            Some("https://dflow.net")
        );
    }
}
