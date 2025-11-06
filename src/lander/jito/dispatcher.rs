use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use reqwest::Client;
use serde_json::Value;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::VersionedTransaction;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use url::Url;

use solana_client::nonblocking::rpc_client::RpcClient;

use crate::config::{
    LanderJitoConfig, LanderJitoForwardSetting, LanderJitoStrategyKind, LanderJitoUuidSetting,
    LanderSettings,
};
use crate::engine::{JitoTipPlan, TxVariant};
use crate::lander::error::LanderError;
use crate::lander::stack::{Deadline, LanderReceipt};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};

use super::bundle::{
    build_jito_transaction, build_jsonrpc_payload, encode_transaction, has_tip_transfer,
    prepare_endpoint_url,
};
use super::dry_run::DryRunFallback;
use super::multi_ips::{MultiIpsBundle, MultiIpsStrategy};
use super::tip::TipSelector;
use super::types::{BundleSubmission, StrategyEndpoint, endpoint_label};
use super::uuid::{UuidPool, UuidTicket, UuidTicketOutcome};

const STRATEGY_METRIC_LABEL: &str = "lander::jito";

#[derive(Clone)]
pub struct JitoLander {
    client: Client,
    client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
    tip_selector: TipSelector,
    endpoints: Vec<StrategyEndpoint>,
    endpoint_lookup: HashMap<String, usize>,
    uuid_pool: Option<Arc<Mutex<UuidPool>>>,
    multi_ips: Option<MultiIpsStrategy>,
    dry_run: Option<DryRunFallback>,
}

impl JitoLander {
    #[allow(dead_code)]
    pub fn new(config: &LanderJitoConfig, client: Client) -> Self {
        Self::with_ip_pool(config, client, None)
    }

    pub fn with_ip_pool(
        config: &LanderJitoConfig,
        client: Client,
        client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
    ) -> Self {
        let tip_selector = TipSelector::from_config(config);
        let enabled = resolve_enabled_strategies(config);

        let mut endpoints = Vec::new();
        let mut uuid_pool = None;
        let mut multi_ips = None;

        for kind in enabled {
            match kind {
                LanderJitoStrategyKind::Uuid => {
                    let setting =
                        config
                            .uuid_setting
                            .clone()
                            .unwrap_or_else(|| LanderJitoUuidSetting {
                                config: config.uuid_config.clone(),
                                endpoints: config.endpoints.clone(),
                            });
                    let entry_endpoints =
                        normalize_endpoints(&setting.endpoints, &config.endpoints);
                    if !entry_endpoints.is_empty() {
                        let base = endpoints.len();
                        for (idx, url) in entry_endpoints.iter().enumerate() {
                            endpoints.push(StrategyEndpoint {
                                label: endpoint_label(kind, base + idx, url),
                                url: url.clone(),
                                kind,
                                index: base + idx,
                            });
                        }
                        if let Some(pool) = UuidPool::new(&setting.config) {
                            uuid_pool = Some(Arc::new(Mutex::new(pool)));
                        } else if uuid_pool.is_none() {
                            warn!(
                                target: STRATEGY_METRIC_LABEL,
                                "uuid 策略未配置 uuid 列表，将只发送普通请求"
                            );
                        }
                    } else {
                        warn!(
                            target: STRATEGY_METRIC_LABEL,
                            "uuid 策略未提供 endpoints，跳过"
                        );
                    }
                }
                LanderJitoStrategyKind::MultiIps => {
                    if let Some(setting) = config.multi_ips_setting.clone() {
                        let entry_endpoints =
                            normalize_endpoints(&setting.endpoints, &config.endpoints);
                        if entry_endpoints.is_empty() {
                            warn!(
                                target: STRATEGY_METRIC_LABEL,
                                "multi_ips 策略未提供 endpoints，跳过"
                            );
                        } else {
                            if let Some(strategy) = MultiIpsStrategy::new(&setting) {
                                let base = endpoints.len();
                                for (idx, url) in entry_endpoints.iter().enumerate() {
                                    endpoints.push(StrategyEndpoint {
                                        label: endpoint_label(kind, base + idx, url),
                                        url: url.clone(),
                                        kind,
                                        index: base + idx,
                                    });
                                }
                                multi_ips = Some(strategy);
                            } else {
                                warn!(
                                    target: STRATEGY_METRIC_LABEL,
                                    "multi_ips 策略初始化失败，跳过"
                                );
                            }
                        }
                    } else {
                        warn!(
                            target: STRATEGY_METRIC_LABEL,
                            "multi_ips 策略未提供配置，跳过"
                        );
                    }
                }
                LanderJitoStrategyKind::Forward => {
                    let setting = config
                        .forward_setting
                        .clone()
                        .unwrap_or_else(LanderJitoForwardSetting::default);
                    let entry_endpoints =
                        normalize_endpoints(&setting.endpoints, &config.endpoints);
                    if entry_endpoints.is_empty() {
                        warn!(
                            target: STRATEGY_METRIC_LABEL,
                            "forward 策略未提供 endpoints，跳过"
                        );
                    } else {
                        let base = endpoints.len();
                        for (idx, url) in entry_endpoints.iter().enumerate() {
                            endpoints.push(StrategyEndpoint {
                                label: endpoint_label(kind, base + idx, url),
                                url: url.clone(),
                                kind,
                                index: base + idx,
                            });
                        }
                    }
                }
            }
        }

        let endpoint_lookup = endpoints
            .iter()
            .enumerate()
            .map(|(idx, endpoint)| (endpoint.label.clone(), idx))
            .collect();

        Self {
            client,
            client_pool,
            tip_selector,
            endpoints,
            endpoint_lookup,
            uuid_pool,
            multi_ips,
            dry_run: None,
        }
    }

    pub fn with_dry_run(mut self, rpc_client: Arc<RpcClient>, settings: &LanderSettings) -> Self {
        self.dry_run = Some(DryRunFallback::new(rpc_client, settings));
        self
    }

    pub fn endpoints(&self) -> usize {
        self.endpoints.len()
    }

    pub fn endpoint_list(&self) -> Vec<String> {
        self.endpoints
            .iter()
            .map(|endpoint| endpoint.label.clone())
            .collect()
    }

    pub fn tip_strategy_label(&self) -> &'static str {
        match self.tip_selector.strategy_kind() {
            crate::config::TipStrategyKind::Fixed => "fixed",
            crate::config::TipStrategyKind::Range => "range",
            crate::config::TipStrategyKind::Stream => "stream",
            crate::config::TipStrategyKind::Api => "api",
        }
    }

    pub fn draw_tip_plan(&self) -> Option<JitoTipPlan> {
        let lamports = self.tip_selector.select_tip()?;
        if lamports == 0 {
            return None;
        }

        let Some(recipient) = super::bundle::random_tip_wallet() else {
            warn!(
                target: STRATEGY_METRIC_LABEL,
                lamports,
                "tip wallet list empty, skipping tip plan"
            );
            return None;
        };

        Some(JitoTipPlan::new(lamports, recipient))
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        endpoint: Option<&str>,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal(
                "deadline expired before jito submission",
            ));
        }

        let endpoints = self.select_endpoints(endpoint)?;
        if endpoints.is_empty() {
            return Err(LanderError::fatal("no jito endpoints available"));
        }

        let base_submission = self.build_base_submission(&variant);
        let base_tip_lamports = base_submission.tip_lamports;
        let encoded_main = encode_transaction(&base_submission.transaction)?;

        let mut submissions = Vec::new();
        for endpoint in endpoints {
            match endpoint.kind {
                LanderJitoStrategyKind::Uuid => {
                    if let Some(pool) = &self.uuid_pool {
                        let mut guard = pool.lock().await;
                        match guard.next_ticket() {
                            UuidTicketOutcome::Ticket(ticket) => {
                                if let Some(submission) = self.build_uuid_submission(
                                    endpoint,
                                    &encoded_main,
                                    &base_submission.transaction,
                                    ticket,
                                ) {
                                    submissions.push(submission);
                                }
                            }
                            UuidTicketOutcome::RateLimited { cooldown } => {
                                debug!(
                                    target: STRATEGY_METRIC_LABEL,
                                    endpoint = %endpoint.url,
                                    cooldown_ms = %cooldown.as_millis(),
                                    "uuid rate limit active, skipping endpoint"
                                );
                            }
                            UuidTicketOutcome::Empty => {
                                warn!(
                                    target: STRATEGY_METRIC_LABEL,
                                    "uuid pool 空，无法发送 bundle"
                                );
                            }
                        }
                    } else {
                        submissions.push(self.build_forward_submission(
                            endpoint,
                            &encoded_main,
                            &base_submission.transaction,
                        ));
                    }
                }
                LanderJitoStrategyKind::Forward => {
                    submissions.push(self.build_forward_submission(
                        endpoint,
                        &encoded_main,
                        &base_submission.transaction,
                    ));
                }
                LanderJitoStrategyKind::MultiIps => {
                    if let Some(strategy) = &self.multi_ips {
                        match strategy
                            .build_bundle(&variant, base_tip_lamports, tip_offset(endpoint.index))
                            .await
                        {
                            Ok(Some(bundle)) => {
                                submissions.push(self.build_multi_ips_submission(endpoint, bundle));
                            }
                            Ok(None) => {
                                debug!(
                                    target: STRATEGY_METRIC_LABEL,
                                    endpoint = %endpoint.url,
                                    "multi_ips strategy returned empty bundle"
                                );
                            }
                            Err(err) => {
                                warn!(
                                    target: STRATEGY_METRIC_LABEL,
                                    endpoint = %endpoint.url,
                                    error = %err,
                                    "multi_ips 构建 bundle 失败"
                                );
                            }
                        }
                    } else {
                        warn!(
                            target: STRATEGY_METRIC_LABEL,
                            endpoint = %endpoint.url,
                            "multi_ips strategy not initialized"
                        );
                    }
                }
            }
        }

        if submissions.is_empty() {
            return Err(LanderError::fatal("no valid jito endpoints configured"));
        }

        if let Some(dry_run) = &self.dry_run {
            let slot = variant.slot();
            let blockhash = variant.blockhash().to_string();
            let variant_id = variant.id();
            let selected = submissions
                .iter()
                .find(|submission| submission.strategy == LanderJitoStrategyKind::MultiIps)
                .or_else(|| submissions.first());
            if let Some(entry) = selected {
                return dry_run
                    .submit_transactions(
                        variant_id,
                        slot,
                        &blockhash,
                        entry.raw_transactions.as_slice(),
                        deadline,
                        Some(entry.label.as_str()),
                        local_ip,
                    )
                    .await;
            }
        }

        let client = self.http_client(local_ip)?;
        let slot = variant.slot();
        let blockhash = variant.blockhash().to_string();
        let variant_id = variant.id();

        let mut futures = FuturesUnordered::new();
        for submission in submissions {
            let payload = submission.payload.clone();
            let endpoint_url = submission.endpoint.clone();
            let strategy = submission.strategy;
            let label = submission.label.clone();
            let bundle_hint = submission.bundle_hint.clone();
            let client = client.clone();
            futures.push(async move {
                let response = client
                    .post(endpoint_url.clone())
                    .json(&payload)
                    .send()
                    .await;
                (endpoint_url, response, strategy, label, bundle_hint)
            });
        }

        while let Some((endpoint_url, response_result, strategy, label, bundle_hint)) =
            futures.next().await
        {
            let response = match response_result {
                Ok(resp) => resp,
                Err(err) => {
                    warn!(
                        target: STRATEGY_METRIC_LABEL,
                        endpoint = %endpoint_url,
                        strategy = strategy.as_str(),
                        label = %label,
                        error = %err,
                        "bundle submission network error"
                    );
                    continue;
                }
            };

            if !response.status().is_success() {
                warn!(
                    target: STRATEGY_METRIC_LABEL,
                    endpoint = %endpoint_url,
                    strategy = strategy.as_str(),
                    status = %response.status(),
                    label = %label,
                    "bundle submission returned non-success status"
                );
                continue;
            }

            let value: Value = match response.json().await {
                Ok(val) => val,
                Err(err) => {
                    warn!(
                        target: STRATEGY_METRIC_LABEL,
                        endpoint = %endpoint_url,
                        strategy = strategy.as_str(),
                        error = %err,
                        label = %label,
                        "bundle submission decode error"
                    );
                    continue;
                }
            };
            if let Some(error) = value.get("error") {
                warn!(
                        target: STRATEGY_METRIC_LABEL,
                        endpoint = %endpoint_url,
                strategy = strategy.as_str(),
                        error = %error,
                        label = %label,
                        "bundle submission returned error"
                    );
                continue;
            }

            let bundle_id = value
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let bundle_id = bundle_id.or(bundle_hint);

            let receipt = LanderReceipt {
                lander: "jito",
                endpoint: endpoint_url.to_string(),
                slot,
                blockhash: blockhash.clone(),
                signature: bundle_id,
                variant_id,
                local_ip,
            };

            info!(
                target: STRATEGY_METRIC_LABEL,
                endpoint = %endpoint_url,
                strategy = strategy.as_str(),
                label = %label,
                slot,
                blockhash = %receipt.blockhash,
                "bundle submission succeeded"
            );

            return Ok(receipt);
        }

        Err(LanderError::fatal("all jito endpoints failed submission"))
    }

    fn http_client(&self, local_ip: Option<IpAddr>) -> Result<Client, LanderError> {
        if let Some(ip) = local_ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(ip)
                    .map_err(|err| LanderError::fatal(format!("构建绑定 IP 的客户端失败: {err}")));
            }
        }
        Ok(self.client.clone())
    }

    fn select_endpoints(
        &self,
        endpoint: Option<&str>,
    ) -> Result<Vec<&StrategyEndpoint>, LanderError> {
        match endpoint
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            Some(label) => {
                if let Some(index) = self.endpoint_lookup.get(label) {
                    Ok(vec![&self.endpoints[*index]])
                } else {
                    Err(LanderError::fatal(format!(
                        "unknown jito endpoint label {label}"
                    )))
                }
            }
            None => Ok(self.endpoints.iter().collect()),
        }
    }

    fn build_uuid_submission(
        &self,
        endpoint: &StrategyEndpoint,
        encoded_main: &str,
        base_tx: &VersionedTransaction,
        ticket: UuidTicket,
    ) -> Option<BundleSubmission> {
        let bundle_hint = ticket.bundle_id.clone();
        let url = prepare_endpoint_url(endpoint, Some(&ticket))?;
        let payload = build_jsonrpc_payload(vec![encoded_main.to_string()], Some(&ticket));
        Some(BundleSubmission {
            label: endpoint.label.clone(),
            strategy: endpoint.kind,
            endpoint: url,
            payload,
            bundle_hint: Some(bundle_hint),
            raw_transactions: vec![base_tx.clone()],
        })
    }

    fn build_forward_submission(
        &self,
        endpoint: &StrategyEndpoint,
        encoded_main: &str,
        base_tx: &VersionedTransaction,
    ) -> BundleSubmission {
        let url = prepare_endpoint_url(endpoint, None)
            .unwrap_or_else(|| Url::parse(&endpoint.url).expect("valid endpoint url"));
        let payload = build_jsonrpc_payload(vec![encoded_main.to_string()], None);
        BundleSubmission {
            label: endpoint.label.clone(),
            strategy: endpoint.kind,
            endpoint: url,
            payload,
            bundle_hint: None,
            raw_transactions: vec![base_tx.clone()],
        }
    }

    fn build_multi_ips_submission(
        &self,
        endpoint: &StrategyEndpoint,
        bundle: MultiIpsBundle,
    ) -> BundleSubmission {
        let MultiIpsBundle {
            encoded_transactions,
            ephemeral_wallet,
            jito_tip_wallet,
            tip_lamports,
            main_transaction,
            tip_transaction,
        } = bundle;

        let url = prepare_endpoint_url(endpoint, None)
            .unwrap_or_else(|| Url::parse(&endpoint.url).expect("valid endpoint url"));
        let payload = build_jsonrpc_payload(encoded_transactions, None);
        debug!(
            target: STRATEGY_METRIC_LABEL,
            endpoint = %endpoint.url,
            strategy = "multi_ips",
            tip_lamports = tip_lamports,
            ephemeral_wallet = %ephemeral_wallet,
            jito_tip_wallet = %jito_tip_wallet,
            "multi_ips bundle ready"
        );
        BundleSubmission {
            label: endpoint.label.clone(),
            strategy: endpoint.kind,
            endpoint: url,
            payload,
            bundle_hint: None,
            raw_transactions: vec![main_transaction, tip_transaction],
        }
    }

    fn build_base_submission(&self, variant: &TxVariant) -> BaseSubmission {
        if let Some(plan) = variant.jito_tip_plan() {
            let payer = variant.signer().pubkey();
            let tip_embedded = has_tip_transfer(variant.instructions(), Some(plan), &payer);
            let lamports = plan.lamports;
            let recipient = plan.recipient;
            let transaction = match if tip_embedded {
                build_jito_transaction(variant, None)
            } else {
                build_jito_transaction(variant, Some((recipient, lamports)))
            } {
                Ok(tx) => tx,
                Err(err) => {
                    warn!(
                        target: STRATEGY_METRIC_LABEL,
                        error = %err,
                        tip_lamports = lamports,
                        recipient = %recipient,
                        tip_embedded,
                        "构建含 tip 交易失败，回退至原始交易"
                    );
                    variant.transaction().clone()
                }
            };
            return BaseSubmission {
                tip_lamports: lamports,
                transaction,
            };
        }

        let configured_tip = self.tip_selector.select_tip();
        let base_tip = (variant.tip_lamports() > 0).then_some(variant.tip_lamports());
        let lamports = configured_tip.or(base_tip).unwrap_or(0);
        let recipient = if lamports > 0 {
            super::bundle::random_tip_wallet()
        } else {
            None
        };

        let transaction = match (lamports, recipient) {
            (value, Some(target_wallet)) if value > 0 => {
                let plan = JitoTipPlan::new(value, target_wallet);
                let payer = variant.signer().pubkey();
                let tip_embedded = has_tip_transfer(variant.instructions(), Some(&plan), &payer);
                match if tip_embedded {
                    build_jito_transaction(variant, None)
                } else {
                    build_jito_transaction(variant, Some((target_wallet, value)))
                } {
                    Ok(tx) => {
                        debug!(
                            target: STRATEGY_METRIC_LABEL,
                            tip_lamports = value,
                            recipient = %target_wallet,
                            tip_embedded,
                            "tip 指令已准备就绪"
                        );
                        tx
                    }
                    Err(err) => {
                        warn!(
                            target: STRATEGY_METRIC_LABEL,
                            tip_lamports = value,
                            recipient = %target_wallet,
                            error = %err,
                            "构建包含 tip 的交易失败，回退为无 tip 交易"
                        );
                        build_jito_transaction(variant, None).unwrap_or_else(|fallback_err| {
                            warn!(
                                target: STRATEGY_METRIC_LABEL,
                                error = %fallback_err,
                                "重构无 tip 交易失败，使用原始交易"
                            );
                            variant.transaction().clone()
                        })
                    }
                }
            }
            (value, None) if value > 0 => {
                warn!(
                    target: STRATEGY_METRIC_LABEL,
                    tip_lamports = value,
                    "tip wallet list empty, skipping tip transaction"
                );
                build_jito_transaction(variant, None).unwrap_or_else(|err| {
                    warn!(
                        target: STRATEGY_METRIC_LABEL,
                        error = %err,
                        "重构无 tip 交易失败，使用原始交易"
                    );
                    variant.transaction().clone()
                })
            }
            _ => build_jito_transaction(variant, None).unwrap_or_else(|err| {
                warn!(
                    target: STRATEGY_METRIC_LABEL,
                    error = %err,
                    "重构无 tip 交易失败，使用原始交易"
                );
                variant.transaction().clone()
            }),
        };

        BaseSubmission {
            tip_lamports: lamports,
            transaction,
        }
    }
}

fn resolve_enabled_strategies(config: &LanderJitoConfig) -> Vec<LanderJitoStrategyKind> {
    if !config.enabled_strategys.is_empty() {
        return config.enabled_strategys.clone();
    }

    if config.uuid_setting.is_some() || !config.uuid_config.is_empty() {
        vec![LanderJitoStrategyKind::Uuid]
    } else if config.forward_setting.is_some() {
        vec![LanderJitoStrategyKind::Forward]
    } else if config.multi_ips_setting.is_some() {
        vec![LanderJitoStrategyKind::MultiIps]
    } else {
        vec![LanderJitoStrategyKind::Uuid]
    }
}

fn normalize_endpoints(primary: &[String], fallback: &[String]) -> Vec<String> {
    let source = if primary.is_empty() {
        fallback
    } else {
        primary
    };
    source
        .iter()
        .map(|endpoint| endpoint.trim().to_string())
        .filter(|endpoint| !endpoint.is_empty())
        .collect()
}

fn tip_offset(index: usize) -> i64 {
    match index % 3 {
        0 => 0,
        1 => 1,
        _ => -1,
    }
}

struct BaseSubmission {
    tip_lamports: u64,
    transaction: VersionedTransaction,
}
