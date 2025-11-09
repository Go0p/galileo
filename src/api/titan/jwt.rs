use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use reqwest::{
    Client, Url,
    header::{
        ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, REFERER,
        USER_AGENT,
    },
};
use serde::Deserialize;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use tokio::sync::Mutex;

use crate::strategy::types::TradePair;

const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
const SEC_CH_UA_VALUE: &str = r#""Google Chrome";v="141", "Not?A_Brand";v="8", "Chromium";v="141""#;
const SEC_CH_UA_PLATFORM_VALUE: &str = r#""Windows""#;
const EXPIRY_SAFETY_SECS: u64 = 30;
const MIN_TTL_SECS: u64 = 60;

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: Instant,
}

impl CachedToken {
    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

#[derive(Debug, Deserialize)]
struct JwtApiResponse {
    token: String,
    expires_at: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Debug, Error)]
pub enum TitanJwtError {
    #[error("Titan JWT 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Titan JWT 客户端初始化失败: {0}")]
    Client(String),
    #[error("Titan JWT 响应异常: {0}")]
    InvalidResponse(String),
}

#[derive(Debug)]
pub struct TitanJwtManager {
    api_url: Url,
    address: String,
    cache: Mutex<HashMap<IpAddr, CachedToken>>,
}

impl TitanJwtManager {
    pub fn new(api_url: Url, address: String) -> Self {
        Self {
            api_url,
            address,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub async fn token_for(&self, ip: IpAddr, pair: &TradePair) -> Result<String, TitanJwtError> {
        if let Some(token) = self.cached_token(ip).await {
            return Ok(token);
        }

        let fresh = self.fetch_and_store(ip, pair).await?;
        Ok(fresh)
    }

    async fn cached_token(&self, ip: IpAddr) -> Option<String> {
        let guard = self.cache.lock().await;
        guard
            .get(&ip)
            .filter(|token| token.is_valid())
            .map(|token| token.token.clone())
    }

    async fn fetch_and_store(&self, ip: IpAddr, pair: &TradePair) -> Result<String, TitanJwtError> {
        let token = self.request_token(ip, pair).await?;
        let mut guard = self.cache.lock().await;
        guard.insert(ip, token.clone());
        Ok(token.token)
    }

    async fn request_token(
        &self,
        ip: IpAddr,
        pair: &TradePair,
    ) -> Result<CachedToken, TitanJwtError> {
        let client = Self::build_client(ip)?;
        let referer = build_referer(pair);
        let mut url = self.api_url.clone();
        {
            let mut pairs = url.query_pairs_mut();
            pairs.clear();
            pairs.append_pair("address", &self.address);
        }

        let response = client
            .get(url)
            .headers(build_headers(&referer)?)
            .send()
            .await?
            .error_for_status()?;

        let payload: JwtApiResponse = response.json().await?;
        if payload.token.trim().is_empty() {
            return Err(TitanJwtError::InvalidResponse("响应缺少 token".to_string()));
        }
        let expires_at = compute_expiry(&payload)?;
        Ok(CachedToken {
            token: payload.token,
            expires_at,
        })
    }

    fn build_client(ip: IpAddr) -> Result<Client, TitanJwtError> {
        reqwest::Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .local_address(ip)
            .timeout(Duration::from_secs(10))
            .no_proxy()
            .build()
            .map_err(|err| TitanJwtError::Client(err.to_string()))
    }
}

fn build_referer(pair: &TradePair) -> String {
    format!(
        "https://titan.exchange/swap?{}-{}",
        pair.input_mint, pair.output_mint
    )
}

fn build_headers(referer: &str) -> Result<HeaderMap, TitanJwtError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("empty"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("cors"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("sec-ch-ua"),
        HeaderValue::from_static(SEC_CH_UA_VALUE),
    );
    headers.insert(
        HeaderName::from_static("sec-ch-ua-platform"),
        HeaderValue::from_static(SEC_CH_UA_PLATFORM_VALUE),
    );
    headers.insert(
        HeaderName::from_static("sec-ch-ua-mobile"),
        HeaderValue::from_static("?0"),
    );
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=1, i"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_str(referer)
            .map_err(|err| TitanJwtError::InvalidResponse(format!("Referer 构造失败: {err}")))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    Ok(headers)
}

fn compute_expiry(payload: &JwtApiResponse) -> Result<Instant, TitanJwtError> {
    let now = Instant::now();
    if let Some(seconds) = payload.expires_in {
        let valid_secs = seconds.saturating_sub(EXPIRY_SAFETY_SECS).max(MIN_TTL_SECS);
        return Ok(now + Duration::from_secs(valid_secs));
    }

    if let Some(ref timestamp) = payload.expires_at {
        let parsed = time::OffsetDateTime::parse(timestamp, &Rfc3339)
            .map_err(|err| TitanJwtError::InvalidResponse(format!("expires_at 解析失败: {err}")))?;
        let now_utc = time::OffsetDateTime::now_utc();
        let delta = parsed - now_utc;
        if delta.is_positive() {
            let secs = delta.whole_seconds().max(MIN_TTL_SECS as i64) as u64;
            return Ok(now + Duration::from_secs(secs.saturating_sub(EXPIRY_SAFETY_SECS)));
        } else {
            return Err(TitanJwtError::InvalidResponse(
                "expires_at 已经过期".to_string(),
            ));
        }
    }

    Err(TitanJwtError::InvalidResponse(
        "响应缺少过期时间".to_string(),
    ))
}
