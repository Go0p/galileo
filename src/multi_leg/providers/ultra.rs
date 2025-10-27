use async_trait::async_trait;
use reqwest::StatusCode;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use tracing::debug;

use crate::api::jupiter::quote::RoutePlanStep;
use crate::api::ultra::{
    UltraApiClient, UltraError,
    order::{OrderRequest, OrderResponse, Router},
};
use crate::config::{UltraQuoteConfig, UltraSwapConfig};
use crate::engine::ultra::{
    UltraAdapter, UltraAdapterError, UltraContext, UltraLookupResolver, UltraLookupState,
    UltraPreparationParams,
};
use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::transaction::decoder::DecodeTxError;
use crate::multi_leg::transaction::instructions::InstructionExtractionError;
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};
use crate::network::{IpLeaseHandle, IpLeaseOutcome};

/// Ultra API 的腿适配器，负责发起 `/order` 报价并将返回的 base64 交易
/// 转换为统一的 `LegPlan`。
#[derive(Clone, Debug)]
pub struct UltraLegProvider {
    client: UltraApiClient,
    descriptor: LegDescriptor,
    quote_config: UltraQuoteConfig,
    #[allow(dead_code)]
    swap_config: UltraSwapConfig,
    actual_signer: Pubkey,
    request_taker_override: Option<Pubkey>,
}

impl UltraLegProvider {
    pub fn new(
        client: UltraApiClient,
        side: LegSide,
        quote_config: UltraQuoteConfig,
        swap_config: UltraSwapConfig,
        actual_signer: Pubkey,
        request_taker_override: Option<Pubkey>,
    ) -> Self {
        Self {
            descriptor: LegDescriptor::new(AggregatorKind::Ultra, side),
            client,
            quote_config,
            swap_config,
            actual_signer,
            request_taker_override,
        }
    }

    fn build_order_request(&self, intent: &QuoteIntent) -> Result<OrderRequest, UltraLegError> {
        let mut request = OrderRequest::new(intent.input_mint, intent.output_mint, intent.amount);
        let request_taker = self.request_taker_override.unwrap_or(self.actual_signer);
        request.taker = Some(request_taker);
        request.use_wsol = self.quote_config.use_wsol;
        if let Some(kind) = self.quote_config.broadcast_fee_type.as_ref() {
            let trimmed = kind.trim();
            if !trimmed.is_empty() {
                request.broadcast_fee_type = Some(trimmed.to_string());
            }
        } else {
            request.broadcast_fee_type = Some("exactFee".to_string());
        }
        if let Some(priority) = self.quote_config.priority_fee_lamports {
            request.priority_fee_lamports = Some(priority);
        }

        // Ultra 的 includeRouters 需要通过额外参数传递。
        if !self.quote_config.include_routers.is_empty() {
            let value = self
                .quote_config
                .include_routers
                .iter()
                .map(|label| label.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .join(",");
            request
                .extra_query_params
                .insert("routers".to_string(), value);
        }

        if !self.quote_config.exclude_routers.is_empty() {
            request.exclude_routers = self
                .quote_config
                .exclude_routers
                .iter()
                .map(|router| match router.as_str() {
                    "metis" => Ok(Router::metis()),
                    "jupiterz" => Ok(Router::jupiterz()),
                    "dflow" => Ok(Router::dflow()),
                    "okx" => Ok(Router::okx()),
                    other => Err(UltraLegError::UnsupportedRouter(other.to_string())),
                })
                .collect::<Result<_, _>>()?;
        }

        Ok(request)
    }

    async fn decode_leg_plan(
        &self,
        quote: &OrderResponse,
        context: &LegBuildContext,
    ) -> Result<LegPlan, UltraLegError> {
        let in_amount = quote
            .in_amount
            .or_else(|| extract_u64(&quote.raw, "inAmount"))
            .or_else(|| sum_route_plan_amount(&quote.route_plan, |step| step.swap_info.in_amount))
            .ok_or(UltraLegError::MissingField { field: "inAmount" })?;
        let out_amount = quote
            .out_amount
            .or_else(|| extract_u64(&quote.raw, "outAmount"))
            .or_else(|| sum_route_plan_amount(&quote.route_plan, |step| step.swap_info.out_amount))
            .ok_or(UltraLegError::MissingField { field: "outAmount" })?;
        let other_amount_threshold = quote
            .other_amount_threshold
            .or_else(|| extract_u64(&quote.raw, "otherAmountThreshold"))
            .or_else(|| sum_route_plan_amount(&quote.route_plan, |step| step.swap_info.out_amount))
            .ok_or(UltraLegError::MissingField {
                field: "otherAmountThreshold",
            })?;
        let payload = &**quote;
        let params = UltraPreparationParams::new(payload)
            .with_compute_unit_price_hint(context.compute_unit_price_micro_lamports)
            .with_taker_hint(self.request_taker_override);
        let prepared = UltraAdapter::prepare(
            params,
            UltraContext::new(self.actual_signer, UltraLookupResolver::Deferred),
        )
        .await
        .map_err(map_adapter_error)?;

        let transaction = prepared.transaction().clone();
        let blockhash = Some(*transaction.message.recent_blockhash());
        let lookup_state = prepared.lookup_state();
        let mut quote_meta = LegQuote::new(
            in_amount,
            out_amount,
            quote.slippage_bps.unwrap_or_default(),
        );
        quote_meta.min_out_amount = Some(other_amount_threshold);
        quote_meta.request_id = quote.request_id.clone();
        quote_meta.quote_id = quote.quote_id.clone();
        quote_meta.provider = quote.router.clone();
        quote_meta.expires_at_ms = quote
            .expire_at
            .as_ref()
            .and_then(|value| value.parse::<u64>().ok());

        Ok(LegPlan {
            descriptor: self.descriptor.clone(),
            quote: quote_meta,
            instructions: prepared.main_instructions().to_vec(),
            compute_budget_instructions: prepared.compute_budget_instructions().to_vec(),
            address_lookup_table_addresses: prepared.address_lookup_table_addresses().to_vec(),
            resolved_lookup_tables: prepared.resolved_lookup_tables().to_vec(),
            prioritization_fee_lamports: prepared.prioritization_fee_lamports(),
            blockhash,
            raw_transaction: Some(transaction),
            signer_rewrite: None,
            account_rewrites: prepared.account_rewrites().to_vec(),
            requested_compute_unit_limit: prepared.requested_compute_unit_limit(),
            requested_compute_unit_price_micro_lamports: prepared
                .requested_compute_unit_price_micro_lamports(),
            requested_tip_lamports: match lookup_state {
                UltraLookupState::Pending => quote.prioritization_fee_lamports,
                UltraLookupState::Resolved => None,
            },
        })
    }
}

#[derive(Debug, Error)]
pub enum UltraLegError {
    #[error("Ultra API 返回错误: {0}")]
    Api(#[from] UltraError),
    #[error("Ultra 交易解码失败: {0}")]
    Decode(#[from] DecodeTxError),
    #[error("Ultra 指令解析失败: {0}")]
    Instruction(String),
    #[error("Ultra 响应包含未支持的 router: {0}")]
    UnsupportedRouter(String),
    #[error("Ultra 响应缺少字段 `{field}`")]
    MissingField { field: &'static str },
    #[error("Ultra 交易需要 {count} 个地址查找表，但尚未解析")]
    AddressLookupPending { count: usize },
    #[error("Ultra 交易缺少地址查找表 {table}")]
    AddressLookupMissing { table: Pubkey },
    #[error("Ultra 地址查找表 {table} 索引 {index} 超出范围 (len = {len})")]
    AddressLookupIndexOutOfBounds {
        table: Pubkey,
        index: u8,
        len: usize,
    },
}

#[async_trait]
impl LegProvider for UltraLegProvider {
    type QuoteResponse = OrderResponse;
    type BuildError = UltraLegError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::QuoteResponse, Self::BuildError> {
        let request = self.build_order_request(intent)?;
        debug!(
            target: "multi_leg::ultra",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            "请求 Ultra /order 报价"
        );
        match self
            .client
            .order_with_ip(&request, lease.map(|handle| handle.ip()))
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => {
                if let Some(handle) = lease {
                    if let Some(outcome) = classify_ultra_api_error(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                Err(UltraLegError::Api(err))
            }
        }
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
        _lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::Plan, Self::BuildError> {
        self.decode_leg_plan(quote, context).await
    }
}

fn classify_ultra_api_error(err: &UltraError) -> Option<IpLeaseOutcome> {
    match err {
        UltraError::Http(inner) => classify_reqwest(inner),
        UltraError::ApiStatus { status, .. } => map_status(status),
        UltraError::Json(_) | UltraError::Schema(_) | UltraError::ClientPool(_) => {
            Some(IpLeaseOutcome::NetworkError)
        }
    }
}

fn classify_reqwest(err: &reqwest::Error) -> Option<IpLeaseOutcome> {
    if err.is_timeout() {
        return Some(IpLeaseOutcome::Timeout);
    }
    if let Some(status) = err.status() {
        if let Some(mapped) = map_status(&status) {
            return Some(mapped);
        }
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}

fn map_status(status: &StatusCode) -> Option<IpLeaseOutcome> {
    if *status == StatusCode::TOO_MANY_REQUESTS {
        return Some(IpLeaseOutcome::RateLimited);
    }
    if *status == StatusCode::REQUEST_TIMEOUT || *status == StatusCode::GATEWAY_TIMEOUT {
        return Some(IpLeaseOutcome::Timeout);
    }
    if status.is_server_error() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}
fn extract_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|entry| match entry {
        Value::String(s) => s.parse().ok(),
        Value::Number(n) => n.as_u64(),
        _ => None,
    })
}

fn sum_route_plan_amount<F>(route_plan: &[RoutePlanStep], mut extractor: F) -> Option<u64>
where
    F: FnMut(&RoutePlanStep) -> u64,
{
    if route_plan.is_empty() {
        return None;
    }

    route_plan
        .iter()
        .try_fold(0u64, |acc, step| acc.checked_add(extractor(step)))
}

fn map_adapter_error(err: UltraAdapterError) -> UltraLegError {
    match err {
        UltraAdapterError::MissingTransaction => UltraLegError::MissingField {
            field: "transaction",
        },
        UltraAdapterError::Decode(inner) => UltraLegError::Decode(inner),
        UltraAdapterError::Instruction(inner) => match inner {
            InstructionExtractionError::MissingLookupTables { count } => {
                UltraLegError::AddressLookupPending { count }
            }
            InstructionExtractionError::LookupTableNotFound { table } => {
                UltraLegError::AddressLookupMissing { table }
            }
            InstructionExtractionError::LookupIndexOutOfBounds { table, index, len } => {
                UltraLegError::AddressLookupIndexOutOfBounds { table, index, len }
            }
            other => UltraLegError::Instruction(other.to_string()),
        },
        UltraAdapterError::LookupFetch(error) => {
            UltraLegError::Instruction(format!("拉取地址查找表失败: {error}"))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BotConfig, LoggingConfig};
    use crate::multi_leg::transaction::decoder::encode_base64_transaction;
    use solana_compute_budget_interface::ComputeBudgetInstruction;
    use solana_message::Message as LegacyMessageType;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction as SdkInstruction},
        transaction::{Transaction, VersionedTransaction},
    };

    fn build_test_ultra_client() -> UltraApiClient {
        let http = reqwest::Client::new();
        UltraApiClient::new(
            http,
            "https://ultra.example.com".to_string(),
            &BotConfig::default(),
            &LoggingConfig::default(),
        )
    }

    fn build_test_order_response(tx: &VersionedTransaction) -> OrderResponse {
        use serde_json::json;
        let encoded = encode_base64_transaction(tx).expect("encode test transaction");
        let value = json!({
            "mode": "ExactIn",
            "inputMint": tx.message.static_account_keys()[0].to_string(),
            "outputMint": tx.message.static_account_keys()[0].to_string(),
            "inAmount": "10",
            "outAmount": "9",
            "otherAmountThreshold": "9",
            "swapMode": "ExactIn",
            "slippageBps": 100,
            "inUsdValue": 0,
            "outUsdValue": 0,
            "priceImpact": 0,
            "swapUsdValue": 0,
            "priceImpactPct": null,
            "routePlan": [],
            "feeMint": null,
            "feeBps": 0,
            "signatureFeeLamports": 5000,
            "prioritizationFeeLamports": 1000,
            "rentFeeLamports": 0,
            "swapType": "aggregator",
            "router": "metis",
            "transaction": encoded,
            "gasless": false,
            "requestId": "req-1",
            "totalTime": 10,
            "taker": null,
            "quoteId": null,
            "maker": null,
            "expireAt": null,
            "platformFee": null,
            "errorCode": null,
            "errorMessage": null
        });
        OrderResponse::try_from_value(value).expect("build test order response")
    }

    fn build_test_transaction() -> VersionedTransaction {
        let payer = Pubkey::new_unique();
        let swap_program = Pubkey::new_unique();
        let account = Pubkey::new_unique();

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
        let swap_ix = SdkInstruction::new_with_bytes(
            swap_program,
            &[1, 2, 3, 4],
            vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(account, false),
            ],
        );

        let message = LegacyMessageType::new(&[compute_ix, swap_ix], Some(&payer));
        VersionedTransaction::from(Transaction::new_unsigned(message))
    }

    #[tokio::test]
    async fn build_plan_extracts_compute_budget() {
        let provider = UltraLegProvider::new(
            build_test_ultra_client(),
            LegSide::Buy,
            UltraQuoteConfig::default(),
            UltraSwapConfig::default(),
            Pubkey::new_unique(),
            None,
        );

        let tx = build_test_transaction();
        let order = build_test_order_response(&tx);
        let plan = provider
            .build_plan(&order, &LegBuildContext::default())
            .await
            .expect("plan");

        assert_eq!(plan.instructions.len(), 1);
        assert_eq!(plan.compute_budget_instructions.len(), 0);
        assert!(plan.blockhash.is_some());
        assert_eq!(plan.prioritization_fee_lamports, Some(1_000));
        assert!(plan.address_lookup_table_addresses.is_empty());
        assert!(plan.raw_transaction.is_some());
    }

    #[tokio::test]
    async fn quote_request_applies_router_filters() {
        let mut quote_config = UltraQuoteConfig::default();
        quote_config.include_routers = vec!["metis".into(), "okx".into()];
        quote_config.exclude_routers = vec!["jupiterz".into()];

        let provider = UltraLegProvider::new(
            build_test_ultra_client(),
            LegSide::Sell,
            quote_config,
            UltraSwapConfig::default(),
            Pubkey::new_unique(),
            None,
        );
        let intent = QuoteIntent::new(Pubkey::new_unique(), Pubkey::new_unique(), 10, 50);
        let request = provider.build_order_request(&intent).expect("request");

        assert_eq!(
            request.extra_query_params.get("routers").unwrap(),
            "metis,okx"
        );
        assert_eq!(request.exclude_routers.len(), 1);
        assert_eq!(request.exclude_routers[0].as_str(), "jupiterz");
    }
}
