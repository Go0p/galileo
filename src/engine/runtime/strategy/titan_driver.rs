use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::engine::context::QuoteBatchPlan;
use crate::engine::multi_leg::providers::titan::TitanWsQuoteSource;
use crate::engine::multi_leg::types::AggregatorKind as MultiLegAggregatorKind;
use crate::engine::runtime::strategy::multi_leg::{
    MultiLegDispatchResult, build_pair_plan_requests,
};
use crate::engine::titan::subscription::{TitanPlanEntry, TitanSubscriptionPlan};
use crate::monitoring::short_mint_str;

use super::MultiLegEngineContext;

pub(super) struct TitanEventConfig {
    pub context: Arc<MultiLegEngineContext>,
    pub payer: Pubkey,
    pub compute_unit_price: Option<u64>,
    pub dex_whitelist: Vec<String>,
    pub dex_blacklist: Vec<String>,
    pub source: Arc<TitanWsQuoteSource>,
    pub plan: TitanSubscriptionPlan,
    pub sender: mpsc::Sender<MultiLegDispatchResult>,
}

pub(super) fn spawn_titan_event_driver(config: TitanEventConfig) {
    if config.plan.is_empty() {
        return;
    }

    let shared = Arc::new(DriverShared {
        context: config.context,
        payer: config.payer,
        compute_unit_price: config.compute_unit_price,
        dex_whitelist: config.dex_whitelist,
        dex_blacklist: config.dex_blacklist,
        source: config.source,
        sender: config.sender,
        batch_id: AtomicU64::new(1),
    });

    for entry in config.plan.entries() {
        let shared = Arc::clone(&shared);
        let entry = entry.clone();
        tokio::spawn(async move {
            shared.run(entry).await;
        });
    }
}

struct DriverShared {
    context: Arc<MultiLegEngineContext>,
    payer: Pubkey,
    compute_unit_price: Option<u64>,
    dex_whitelist: Vec<String>,
    dex_blacklist: Vec<String>,
    source: Arc<TitanWsQuoteSource>,
    sender: mpsc::Sender<MultiLegDispatchResult>,
    batch_id: AtomicU64,
}

impl DriverShared {
    async fn run(self: Arc<Self>, entry: TitanPlanEntry) {
        let base_display = short_mint_str(entry.pair.input_mint.as_str());
        let quote_display = short_mint_str(entry.pair.output_mint.as_str());
        let mut backoff = Duration::from_secs(1);
        loop {
            match self
                .source
                .subscribe_updates(entry.pair.clone(), entry.amount, entry.ip)
                .await
            {
                Ok(mut stream) => {
                    info!(
                        target: "multi_leg::titan",
                        base_mint = %base_display,
                        quote_mint = %quote_display,
                        amount = entry.amount,
                        ip = %entry.ip,
                        "Titan 事件驱动已建立"
                    );
                    backoff = Duration::from_secs(1);
                    loop {
                        match stream.next_quote().await {
                            Ok(_) => {
                                if let Err(err) = self.dispatch(&entry).await {
                                    warn!(
                                        target: "multi_leg::titan",
                                        error = ?err,
                                        base_mint = %base_display,
                                        quote_mint = %quote_display,
                                        amount = entry.amount,
                                        "Titan 事件触发失败，继续监听"
                                    );
                                }
                            }
                            Err(err) => {
                                warn!(
                                    target: "multi_leg::titan",
                                    error = ?err,
                                    base_mint = %base_display,
                                    quote_mint = %quote_display,
                                    amount = entry.amount,
                                    "Titan 推流事件订阅中断，将重试"
                                );
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        target: "multi_leg::titan",
                        error = ?err,
                        base_mint = %base_display,
                        quote_mint = %quote_display,
                        amount = entry.amount,
                        "Titan 推流订阅失败，将重试"
                    );
                }
            }

            sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    }

    async fn dispatch(&self, entry: &TitanPlanEntry) -> Result<(), TitanDriverError> {
        let batch_id = self.batch_id.fetch_add(1, Ordering::Relaxed);
        let batch = QuoteBatchPlan {
            batch_id,
            pair: entry.pair.clone(),
            amount: entry.amount,
            preferred_ip: Some(entry.ip),
        };
        let requests = build_pair_plan_requests(
            &self.context,
            self.payer,
            self.compute_unit_price,
            &self.dex_whitelist,
            &self.dex_blacklist,
            &batch,
            |buy, _| buy.kind == MultiLegAggregatorKind::Titan,
        );
        if requests.is_empty() {
            debug!(
                target: "multi_leg::titan",
                base_mint = %entry.pair.input_mint,
                quote_mint = %entry.pair.output_mint,
                amount = entry.amount,
                "Titan 事件对应的组合为空，跳过"
            );
            return Ok(());
        }
        let result = self
            .context
            .runtime()
            .plan_pair_batch_with_profit(requests)
            .await;
        let dispatch = MultiLegDispatchResult { batch, result };
        self.sender
            .send(dispatch)
            .await
            .map_err(|_| TitanDriverError::ChannelClosed)
    }
}

#[derive(Debug)]
enum TitanDriverError {
    ChannelClosed,
}
