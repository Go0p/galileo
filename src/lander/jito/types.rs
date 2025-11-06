use crate::config::LanderJitoStrategyKind;
use serde_json::Value;
use solana_sdk::transaction::VersionedTransaction;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct StrategyEndpoint {
    pub label: String,
    pub url: String,
    pub kind: LanderJitoStrategyKind,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct BundleSubmission {
    pub label: String,
    pub strategy: LanderJitoStrategyKind,
    pub endpoint: Url,
    pub payload: Value,
    pub bundle_hint: Option<String>,
    pub raw_transactions: Vec<VersionedTransaction>,
}

impl LanderJitoStrategyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanderJitoStrategyKind::Uuid => "uuid",
            LanderJitoStrategyKind::MultiIps => "multi_ips",
            LanderJitoStrategyKind::Forward => "forward",
        }
    }
}

pub(crate) fn endpoint_label(kind: LanderJitoStrategyKind, index: usize, url: &str) -> String {
    format!("{}#{index}:{}", kind.as_str(), url)
}
