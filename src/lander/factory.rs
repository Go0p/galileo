use std::collections::HashSet;
use std::sync::Arc;

use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::warn;

use crate::config::LanderSettings;

use super::error::LanderError;
use super::jito::JitoLander;
use super::rpc::RpcLander;
use super::stack::{LanderStack, LanderVariant};
use super::staked::StakedLander;

#[derive(Clone)]
pub struct LanderFactory {
    rpc_client: Arc<RpcClient>,
    http_client: Client,
}

impl LanderFactory {
    pub fn new(rpc_client: Arc<RpcClient>, http_client: Client) -> Self {
        Self {
            rpc_client,
            http_client,
        }
    }

    pub fn build_stack(
        &self,
        settings: &LanderSettings,
        desired: &[String],
        fallback: &[&str],
        max_retries: usize,
    ) -> Result<LanderStack, LanderError> {
        let mut variants = Vec::new();
        let mut seen = HashSet::new();

        for name in desired
            .iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
        {
            push_variant(
                &mut variants,
                &mut seen,
                &name,
                self.instantiate(settings, &name),
            );
        }

        if variants.is_empty() {
            for name in fallback.iter().map(|s| s.trim().to_ascii_lowercase()) {
                push_variant(
                    &mut variants,
                    &mut seen,
                    &name,
                    self.instantiate(settings, &name),
                );
            }
        }

        if variants.is_empty() {
            return Err(LanderError::fatal(
                "no lander available after factory selection",
            ));
        }

        Ok(LanderStack::new(variants, max_retries))
    }

    fn instantiate(&self, settings: &LanderSettings, name: &str) -> Option<LanderVariant> {
        match name {
            "rpc" => Some(LanderVariant::Rpc(RpcLander::new(
                self.rpc_client.clone(),
                settings.skip_preflight,
                settings.max_retries,
                settings.min_context_slot,
            ))),
            "jito" => settings.jito.as_ref().and_then(|cfg| {
                let endpoints: Vec<String> = cfg
                    .endpoints
                    .iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if endpoints.is_empty() {
                    None
                } else {
                    Some(LanderVariant::Jito(JitoLander::new(
                        endpoints,
                        self.http_client.clone(),
                    )))
                }
            }),
            "staked" => settings.staked.as_ref().and_then(|cfg| {
                let endpoints: Vec<String> = cfg
                    .endpoints
                    .iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if endpoints.is_empty() {
                    None
                } else {
                    Some(LanderVariant::Staked(StakedLander::new(
                        endpoints,
                        self.http_client.clone(),
                    )))
                }
            }),
            other => {
                warn!(target: "lander::factory", lander = other, "unsupported lander requested");
                None
            }
        }
    }
}

fn push_variant(
    variants: &mut Vec<LanderVariant>,
    seen: &mut HashSet<String>,
    name: &str,
    variant: Option<LanderVariant>,
) {
    if seen.insert(name.to_owned()) {
        if let Some(v) = variant {
            variants.push(v);
        } else {
            warn!(
                target: "lander::factory",
                lander = name,
                "requested lander missing configuration"
            );
        }
    }
}
