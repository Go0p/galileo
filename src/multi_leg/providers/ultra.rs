use async_trait::async_trait;
use solana_compute_budget_interface as compute_budget;
use solana_message::{MessageHeader, VersionedMessage, compiled_instruction::CompiledInstruction};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use thiserror::Error;
use tracing::debug;

use crate::api::ultra::{
    UltraApiClient, UltraError,
    order::{OrderRequest, OrderResponse, Router},
};
use crate::config::{UltraQuoteConfig, UltraSwapConfig};
use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::transaction::decoder::{DecodeTxError, decode_base64_transaction};
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};

/// Ultra API 的腿适配器，负责发起 `/order` 报价并将返回的 base64 交易
/// 转换为统一的 `LegPlan`。
#[derive(Clone, Debug)]
pub struct UltraLegProvider {
    client: UltraApiClient,
    descriptor: LegDescriptor,
    quote_config: UltraQuoteConfig,
    #[allow(dead_code)]
    swap_config: UltraSwapConfig,
}

impl UltraLegProvider {
    pub fn new(
        client: UltraApiClient,
        side: LegSide,
        quote_config: UltraQuoteConfig,
        swap_config: UltraSwapConfig,
    ) -> Self {
        Self {
            descriptor: LegDescriptor::new(AggregatorKind::Ultra, side),
            client,
            quote_config,
            swap_config,
        }
    }

    fn build_order_request(&self, intent: &QuoteIntent) -> Result<OrderRequest, UltraLegError> {
        let mut request = OrderRequest::new(intent.input_mint, intent.output_mint, intent.amount);

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

    fn decode_leg_plan(&self, quote: &OrderResponse) -> Result<LegPlan, UltraLegError> {
        let encoded = quote.transaction.as_str();
        let tx = decode_base64_transaction(encoded)?;
        let instructions = convert_instructions(&tx.message)?;
        let (compute_budget_instructions, mut other_instructions): (Vec<_>, Vec<_>) = instructions
            .into_iter()
            .partition(|ix| ix.program_id == compute_budget::id());

        // Ultra 返回的指令顺序已经完成 setup/swap/cleanup 排列，暂不额外处理。
        let address_lookup_table_addresses = collect_lookup_addresses(&tx.message);
        let blockhash = Some(*tx.message.recent_blockhash());
        let mut quote_meta = LegQuote::new(quote.in_amount, quote.out_amount, quote.slippage_bps);
        quote_meta.min_out_amount = Some(quote.other_amount_threshold);
        quote_meta.request_id = Some(quote.request_id.clone());
        quote_meta.quote_id = quote.quote_id.clone();
        quote_meta.provider = Some(quote.router.clone());
        quote_meta.expires_at_ms = quote
            .expire_at
            .as_ref()
            .and_then(|value| value.parse::<u64>().ok());

        Ok(LegPlan {
            descriptor: self.descriptor.clone(),
            quote: quote_meta,
            instructions: other_instructions.drain(..).collect(),
            compute_budget_instructions,
            address_lookup_table_addresses,
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports: Some(quote.prioritization_fee_lamports),
            blockhash,
            raw_transaction: Some(tx),
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
    #[error("暂不支持解析含有地址查找表的 Ultra 交易 (lookups = {count})")]
    AddressLookupUnsupported { count: usize },
}

#[async_trait]
impl LegProvider for UltraLegProvider {
    type QuoteResponse = OrderResponse;
    type BuildError = UltraLegError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn quote(&self, intent: &QuoteIntent) -> Result<Self::QuoteResponse, Self::BuildError> {
        let request = self.build_order_request(intent)?;
        debug!(
            target: "multi_leg::ultra",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            "请求 Ultra /order 报价"
        );
        Ok(self.client.order(&request).await?)
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        _context: &LegBuildContext,
    ) -> Result<Self::Plan, Self::BuildError> {
        self.decode_leg_plan(quote)
    }
}

fn convert_instructions(message: &VersionedMessage) -> Result<Vec<Instruction>, UltraLegError> {
    match message {
        VersionedMessage::Legacy(legacy) => convert_compiled_instructions(
            &legacy.instructions,
            &legacy.account_keys,
            &legacy.header,
        ),
        VersionedMessage::V0(v0) => {
            if !v0.address_table_lookups.is_empty() {
                return Err(UltraLegError::AddressLookupUnsupported {
                    count: v0.address_table_lookups.len(),
                });
            }
            convert_compiled_instructions(&v0.instructions, &v0.account_keys, &v0.header)
        }
    }
}

fn convert_compiled_instructions(
    compiled: &[CompiledInstruction],
    account_keys: &[Pubkey],
    header: &MessageHeader,
) -> Result<Vec<Instruction>, UltraLegError> {
    compiled
        .iter()
        .map(|ix| convert_single_instruction(ix, account_keys, header))
        .collect()
}

fn convert_single_instruction(
    ix: &CompiledInstruction,
    account_keys: &[Pubkey],
    header: &MessageHeader,
) -> Result<Instruction, UltraLegError> {
    let program_index = ix.program_id_index as usize;
    let program_id = account_keys.get(program_index).ok_or_else(|| {
        UltraLegError::Instruction(format!(
            "program index {program_index} 超出 account_keys 长度 {}",
            account_keys.len()
        ))
    })?;

    let mut accounts = Vec::with_capacity(ix.accounts.len());
    for account_index in &ix.accounts {
        let idx = *account_index as usize;
        let key = account_keys.get(idx).ok_or_else(|| {
            UltraLegError::Instruction(format!(
                "account index {idx} 超出 account_keys 长度 {}",
                account_keys.len()
            ))
        })?;
        accounts.push(AccountMeta {
            pubkey: *key,
            is_signer: is_signer(idx, header),
            is_writable: is_writable(idx, header, account_keys.len()),
        });
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: ix.data.clone(),
    })
}

fn is_signer(index: usize, header: &MessageHeader) -> bool {
    index < header.num_required_signatures as usize
}

fn is_writable(index: usize, header: &MessageHeader, total_keys: usize) -> bool {
    let num_required_signatures = header.num_required_signatures as usize;
    let writable_signed =
        num_required_signatures.saturating_sub(header.num_readonly_signed_accounts as usize);
    if index < num_required_signatures {
        return index < writable_signed;
    }

    let num_unsigned = total_keys.saturating_sub(num_required_signatures);
    let writable_unsigned =
        num_unsigned.saturating_sub(header.num_readonly_unsigned_accounts as usize);
    let unsigned_index = index.saturating_sub(num_required_signatures);
    unsigned_index < writable_unsigned
}

fn collect_lookup_addresses(message: &VersionedMessage) -> Vec<Pubkey> {
    match message {
        VersionedMessage::Legacy(_) => Vec::new(),
        VersionedMessage::V0(v0) => v0
            .address_table_lookups
            .iter()
            .map(|lookup| lookup.account_key)
            .collect(),
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
        instruction::Instruction as SdkInstruction,
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
        );

        let tx = build_test_transaction();
        let order = build_test_order_response(&tx);
        let plan = provider
            .build_plan(&order, &LegBuildContext::default())
            .await
            .expect("plan");

        assert_eq!(plan.instructions.len(), 1);
        assert_eq!(plan.compute_budget_instructions.len(), 1);
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
