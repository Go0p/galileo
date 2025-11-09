use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::net::IpAddr;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesUnordered};
use solana_compute_budget_interface as compute_budget;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use tokio::sync::{Mutex, watch};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::api::titan::{
    Instruction as TitanInstruction, SwapRoute, TitanError,
    manager::{TitanLeg, TitanQuoteUpdate, TitanSubscriptionConfig, subscribe_quote_stream},
};
use crate::cache::cached_associated_token_address;
use crate::engine::multi_leg::leg::LegProvider;
use crate::engine::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
    SignerRewrite,
};
use crate::engine::titan::subscription::TitanSubscriptionPlan;
use crate::monitoring::short_mint_str;
use crate::network::{IpLeaseHandle, IpLeaseOutcome};
use crate::strategy::types::TradePair;

/// Titan 报价源抽象，便于在单元测试中注入 mock。
#[async_trait]
pub trait TitanQuoteSource: Send + Sync {
    async fn quote(
        &self,
        intent: &QuoteIntent,
        side: LegSide,
        local_ip: Option<IpAddr>,
    ) -> Result<TitanQuote, TitanLegError>;
}

/// Titan 报价结构，封装选定的 SwapRoute 及相关上下文。
#[derive(Debug, Clone)]
pub struct TitanQuote {
    pub route: SwapRoute,
    pub provider: String,
    pub quote_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TitanLegProvider<S> {
    descriptor: LegDescriptor,
    source: Arc<S>,
    placeholder_signer: Pubkey,
    use_wsol: bool,
    allowed_programs: Arc<HashSet<Pubkey>>,
    filter_other_instructions: bool,
}

impl<S> TitanLegProvider<S> {
    pub fn new(
        source: S,
        side: LegSide,
        placeholder_signer: Pubkey,
        use_wsol: bool,
        allowed_programs: HashSet<Pubkey>,
        filter_other_instructions: bool,
    ) -> Self {
        Self {
            descriptor: LegDescriptor::new(AggregatorKind::Titan, side),
            source: Arc::new(source),
            placeholder_signer,
            use_wsol,
            allowed_programs: Arc::new(allowed_programs),
            filter_other_instructions,
        }
    }

    fn rewrite_placeholder_accounts(&self, plan: &mut LegPlan, payer: Pubkey, route: &SwapRoute) {
        if payer == self.placeholder_signer {
            return;
        }

        plan.signer_rewrite = Some(SignerRewrite {
            original: self.placeholder_signer,
            replacement: payer,
        });

        let rewrites =
            collect_placeholder_ata_rewrites(self.placeholder_signer, payer, route, plan);

        for (from, to) in rewrites {
            if plan
                .account_rewrites
                .iter()
                .any(|existing| existing.0 == from && existing.1 == to)
            {
                continue;
            }
            plan.account_rewrites.push((from, to));
        }
    }
}

/// 基于 Titan WebSocket 推流的报价源实现。
#[derive(Clone, Debug)]
pub struct TitanWsQuoteSource {
    config: Arc<TitanSubscriptionConfig>,
    streams: Arc<Mutex<HashMap<StreamKey, Arc<TitanStreamState>>>>,
    first_quote_timeout: Option<Duration>,
    push_stride: NonZeroU64,
    stride_wait_timeout: Option<Duration>,
}

impl TitanWsQuoteSource {
    pub fn new(
        config: TitanSubscriptionConfig,
        first_quote_timeout: Option<Duration>,
        push_stride: NonZeroU64,
        stride_wait_timeout: Option<Duration>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            streams: Arc::new(Mutex::new(HashMap::new())),
            first_quote_timeout,
            push_stride,
            stride_wait_timeout,
        }
    }

    fn select_best_route(update: &TitanQuoteUpdate) -> Option<(String, SwapRoute)> {
        update
            .quotes
            .quotes
            .iter()
            .max_by_key(|(_, route)| route.out_amount)
            .map(|(provider, route)| (provider.clone(), route.clone()))
    }

    async fn stream_state(
        &self,
        pair: TradePair,
        amount: u64,
        local_ip: IpAddr,
    ) -> Result<Arc<TitanStreamState>, TitanLegError> {
        let key = StreamKey::new(pair, amount, local_ip);

        if let Some(existing) = self.streams.lock().await.get(&key) {
            return Ok(existing.clone());
        }
        let (sender, receiver) = watch::channel(StreamSnapshot::empty());
        let state = Arc::new(TitanStreamState::new(receiver));

        {
            let mut guard = self.streams.lock().await;
            guard.insert(key.clone(), state.clone());
        }

        let config = Arc::clone(&self.config);
        let streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            if let Err(err) = Self::run_stream(config, key.clone(), local_ip, sender).await {
                debug!(
                    target: "multi_leg::titan",
                    error = %err,
                    amount = key.amount,
                    input = %key.pair.input_pubkey,
                    output = %key.pair.output_pubkey,
                    ip = %key.ip,
                    "Titan 推流处理失败，即将移除缓存"
                );
            }

            streams.lock().await.remove(&key);
        });

        Ok(state)
    }

    async fn accept_snapshot(
        &self,
        cursor: &TitanStreamCursor,
        snapshot: &StreamSnapshot,
    ) -> Option<TitanQuote> {
        let (Some(seq), Some(quote)) = (snapshot.seq, snapshot.quote.clone()) else {
            return None;
        };

        if self.push_stride.get() == 1 {
            cursor.force_consume(seq).await;
            return Some(quote);
        }

        if cursor.try_consume_with_stride(seq, self.push_stride).await {
            Some(quote)
        } else {
            None
        }
    }

    async fn force_accept_snapshot(
        &self,
        cursor: &TitanStreamCursor,
        snapshot: &StreamSnapshot,
    ) -> Option<TitanQuote> {
        let (Some(seq), Some(quote)) = (snapshot.seq, snapshot.quote.clone()) else {
            return None;
        };
        cursor.force_consume(seq).await;
        Some(quote)
    }

    async fn run_stream(
        config: Arc<TitanSubscriptionConfig>,
        key: StreamKey,
        local_ip: IpAddr,
        sender: watch::Sender<StreamSnapshot>,
    ) -> Result<(), TitanLegError> {
        let idle_timeout = config.idle_resubscribe_timeout;
        let base_display = short_mint_str(key.pair.input_mint.as_str());
        let quote_display = short_mint_str(key.pair.output_mint.as_str());
        loop {
            let mut stream = subscribe_quote_stream(
                (*config).clone(),
                &key.pair,
                TitanLeg::Forward,
                key.amount,
                Some(local_ip),
            )
            .await
            .map_err(TitanLegError::from)?;

            debug!(
                target: "multi_leg::titan",
                amount = key.amount,
                base_mint = %base_display,
                quote_mint = %quote_display,
                ip = %key.ip,
                "已建立 Titan 报价流"
            );

            let mut idle_triggered = false;
            while let Some(update) = {
                let recv_future = stream.recv();
                match idle_timeout {
                    Some(timeout) => match tokio::time::timeout(timeout, recv_future).await {
                        Ok(item) => item,
                        Err(_) => {
                            idle_triggered = true;
                            warn!(
                                target: "multi_leg::titan",
                                amount = key.amount,
                                base_mint = %base_display,
                                quote_mint = %quote_display,
                                ip = %key.ip,
                                timeout_ms = timeout.as_millis() as u64,
                                "Titan 推流在指定时间内无更新，重新订阅"
                            );
                            None
                        }
                    },
                    None => recv_future.await,
                }
            } {
                if let Some((provider, route)) = Self::select_best_route(&update) {
                    debug!(
                        target: "multi_leg::titan",
                        provider = provider,
                        out_amount = route.out_amount,
                        in_amount = route.in_amount,
                        slippage_bps = route.slippage_bps,
                        ip = %key.ip,
                        "Titan 报价更新"
                    );
                    let quote = TitanQuote {
                        route,
                        provider,
                        quote_id: Some(update.quotes.id.clone()),
                    };
                    let snapshot = StreamSnapshot::with_quote(update.seq, quote);
                    if sender.send(snapshot).is_err() {
                        break;
                    }
                }
            }

            if idle_triggered {
                continue;
            }

            break;
        }

        let _ = sender.send(StreamSnapshot::empty());
        Ok(())
    }

    pub async fn ensure_stream(
        &self,
        pair: TradePair,
        amount: u64,
        local_ip: IpAddr,
    ) -> Result<(), TitanLegError> {
        let _ = self.stream_state(pair, amount, local_ip).await?;
        Ok(())
    }

    pub async fn bootstrap_plan(&self, plan: &TitanSubscriptionPlan) -> Result<(), TitanLegError> {
        if plan.is_empty() {
            return Ok(());
        }
        let mut tasks = FuturesUnordered::new();
        for entry in plan.entries() {
            info!(
                target: "multi_leg::titan",
                ip = %entry.ip,
                base_mint = %short_mint_str(entry.pair.input_mint.as_str()),
                quote_mint = %short_mint_str(entry.pair.output_mint.as_str()),
                amount = entry.amount,
                "Titan 预热订阅"
            );
            tasks.push(self.ensure_stream(entry.pair.clone(), entry.amount, entry.ip));
        }
        while let Some(result) = tasks.next().await {
            result?;
        }
        Ok(())
    }
}

#[async_trait]
impl TitanQuoteSource for TitanWsQuoteSource {
    async fn quote(
        &self,
        intent: &QuoteIntent,
        side: LegSide,
        local_ip: Option<IpAddr>,
    ) -> Result<TitanQuote, TitanLegError> {
        if side != LegSide::Buy {
            return Err(TitanLegError::Source("Titan 仅支持买腿报价".to_string()));
        }

        let trade_pair = TradePair::from_pubkeys(intent.input_mint, intent.output_mint);
        let amount = intent.amount;
        let local_ip = local_ip.ok_or_else(|| {
            TitanLegError::Source("Titan 推流未建立且缺少 IP 资源，无法初始化".to_string())
        })?;
        let key = StreamKey::new(trade_pair.clone(), amount, local_ip);
        let state = self.stream_state(trade_pair, amount, local_ip).await?;
        let mut cursor = state.cursor();

        if let Some(quote) = self.accept_snapshot(&cursor, &cursor.snapshot()).await {
            return Ok(quote);
        }

        loop {
            let snapshot = cursor.snapshot();
            if let Some(quote) = self.accept_snapshot(&cursor, &snapshot).await {
                return Ok(quote);
            }

            let wait_duration = if snapshot.seq.is_none() {
                self.first_quote_timeout
            } else {
                self.stride_wait_timeout
            };

            let wait_future = cursor.wait_for_change();
            match wait_duration {
                Some(wait) if wait > Duration::ZERO => match timeout(wait, wait_future).await {
                    Ok(Ok(())) => continue,
                    Ok(Err(_)) => break,
                    Err(_) => {
                        if snapshot.seq.is_none() {
                            let timeout_ms = wait.as_millis() as u64;
                            warn!(
                                target: "multi_leg::titan",
                                amount,
                                input = %intent.input_mint,
                                output = %intent.output_mint,
                                timeout_ms,
                                "Titan 首次报价超时"
                            );
                            return Err(TitanLegError::Source(format!(
                                "Titan 推流首次报价超时 {}ms",
                                timeout_ms
                            )));
                        }

                        if let Some(quote) = self.force_accept_snapshot(&cursor, &snapshot).await {
                            return Ok(quote);
                        }
                        continue;
                    }
                },
                _ => {
                    if wait_future.await.is_err() {
                        break;
                    }
                }
            }
        }

        self.streams.lock().await.remove(&key);
        Err(TitanLegError::Source(
            "Titan 推流结束但未获得有效路线".to_string(),
        ))
    }
}

#[derive(Debug, Error)]
pub enum TitanLegError {
    #[error("Titan 报价源错误: {0}")]
    Source(String),
    #[error("Titan API 错误: {0}")]
    Api(#[from] TitanError),
}

#[async_trait]
impl<S> LegProvider for TitanLegProvider<S>
where
    S: TitanQuoteSource + Send + Sync + Debug + 'static,
{
    type QuoteResponse = TitanQuote;
    type BuildError = TitanLegError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    fn summarize_quote(&self, quote: &Self::QuoteResponse) -> LegQuote {
        let mut summary = LegQuote::new(
            quote.route.in_amount,
            quote.route.out_amount,
            quote.route.slippage_bps,
        );
        summary.provider = Some(quote.provider.clone());
        summary.quote_id = quote.quote_id.clone();
        summary.context_slot = quote.route.context_slot;
        summary.expires_at_ms = quote.route.expires_at_ms;
        summary.expires_after_slot = quote.route.expires_after_slot;
        summary
    }

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::QuoteResponse, Self::BuildError> {
        debug!(
            target: "multi_leg::titan",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            side = %self.descriptor.side,
            "请求 Titan 报价"
        );
        let local_ip = lease.map(|handle| handle.ip());
        let result = self
            .source
            .quote(intent, self.descriptor.side, local_ip)
            .await;
        if let Err(err) = &result {
            if let Some(handle) = lease {
                tracing::warn!(
                    target = "multi_leg::titan",
                    error = %err,
                    "Titan 报价失败，标记网络错误"
                );
                handle.mark_outcome(IpLeaseOutcome::NetworkError);
            }
        } else if let Ok(quote) = &result {
            debug!(
                target: "multi_leg::titan",
                provider = quote.provider,
                quote_id = quote.quote_id.as_deref().unwrap_or("-"),
                amount_in = quote.route.in_amount,
                amount_out = quote.route.out_amount,
                slippage_bps = quote.route.slippage_bps,
                "Titan 报价就绪"
            );
        }
        result
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
        _lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::Plan, Self::BuildError> {
        let mut requested_limit = quote.route.compute_units_safe.map(|value| value as u32);
        if requested_limit.is_none() {
            requested_limit = quote.route.compute_units.map(|value| value as u32);
        }
        let mut requested_price = None;
        let mut compute_budget_instructions = Vec::new();
        let mut other_instructions = Vec::new();

        for ix in quote.route.instructions.iter().map(convert_instruction) {
            if ix.program_id == compute_budget::id() {
                if let Some(value) = parse_compute_unit_limit(&ix) {
                    requested_limit = Some(value);
                }
                if let Some(price) = parse_compute_unit_price(&ix) {
                    requested_price = Some(price);
                }
                if !self.filter_other_instructions {
                    compute_budget_instructions.push(ix);
                }
                continue;
            }

            if self.filter_other_instructions && !self.allowed_programs.contains(&ix.program_id) {
                continue;
            }

            other_instructions.push(ix);
        }

        let mut plan = LegPlan {
            descriptor: self.descriptor.clone(),
            quote: self.summarize_quote(quote),
            instructions: other_instructions,
            compute_budget_instructions,
            address_lookup_table_addresses: quote.route.address_lookup_tables.clone(),
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports: None,
            blockhash: None,
            raw_transaction: None,
            signer_rewrite: None,
            account_rewrites: Vec::new(),
            requested_compute_unit_limit: None,
            requested_compute_unit_price_micro_lamports: None,
            requested_tip_lamports: None,
        };

        if let Some(limit) = requested_limit {
            plan.requested_compute_unit_limit = Some(limit);
        }
        if let Some(price) = requested_price {
            plan.requested_compute_unit_price_micro_lamports = Some(price);
        }

        self.rewrite_placeholder_accounts(&mut plan, context.payer, &quote.route);
        strip_redundant_ata_creations(&mut plan, context.payer);
        if self.use_wsol {
            strip_wsol_wrapping_instructions(&mut plan, context.payer);
        }

        Ok(plan)
    }
}

fn convert_instruction(ix: &TitanInstruction) -> Instruction {
    let accounts = ix
        .accounts
        .iter()
        .map(|meta| AccountMeta {
            pubkey: meta.pubkey,
            is_signer: meta.signer,
            is_writable: meta.writable,
        })
        .collect::<Vec<_>>();
    Instruction {
        program_id: ix.program_id,
        accounts,
        data: ix.data.clone(),
    }
}

fn parse_compute_unit_limit(ix: &Instruction) -> Option<u32> {
    if ix.program_id != compute_budget::id() {
        return None;
    }
    let data = ix.data.as_slice();
    if data.first().copied()? == 2 && data.len() >= 5 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&data[1..5]);
        return Some(u32::from_le_bytes(buf));
    }
    None
}

fn parse_compute_unit_price(ix: &Instruction) -> Option<u64> {
    if ix.program_id != compute_budget::id() {
        return None;
    }
    let data = ix.data.as_slice();
    if data.first().copied()? == 3 && data.len() >= 9 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&data[1..9]);
        return Some(u64::from_le_bytes(buf));
    }
    None
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StreamKey {
    pair: TradePair,
    amount: u64,
    ip: IpAddr,
}

impl StreamKey {
    fn new(pair: TradePair, amount: u64, ip: IpAddr) -> Self {
        Self { pair, amount, ip }
    }
}

#[derive(Debug)]
struct TitanStreamState {
    receiver: watch::Receiver<StreamSnapshot>,
    last_consumed: Arc<Mutex<Option<u64>>>,
}

impl TitanStreamState {
    fn new(receiver: watch::Receiver<StreamSnapshot>) -> Self {
        Self {
            receiver,
            last_consumed: Arc::new(Mutex::new(None)),
        }
    }

    fn cursor(&self) -> TitanStreamCursor {
        TitanStreamCursor {
            receiver: self.receiver.clone(),
            last_consumed: Arc::clone(&self.last_consumed),
        }
    }
}

#[derive(Debug, Clone)]
struct StreamSnapshot {
    seq: Option<u64>,
    quote: Option<TitanQuote>,
}

impl StreamSnapshot {
    fn empty() -> Self {
        Self {
            seq: None,
            quote: None,
        }
    }

    fn with_quote(seq: u64, quote: TitanQuote) -> Self {
        Self {
            seq: Some(seq),
            quote: Some(quote),
        }
    }
}

struct TitanStreamCursor {
    receiver: watch::Receiver<StreamSnapshot>,
    last_consumed: Arc<Mutex<Option<u64>>>,
}

impl TitanStreamCursor {
    fn snapshot(&self) -> StreamSnapshot {
        self.receiver.borrow().clone()
    }

    async fn wait_for_change(&mut self) -> Result<(), watch::error::RecvError> {
        self.receiver.changed().await
    }

    async fn try_consume_with_stride(&self, seq: u64, stride: NonZeroU64) -> bool {
        let mut guard = self.last_consumed.lock().await;
        match *guard {
            None => {
                *guard = Some(seq);
                true
            }
            Some(prev) if seq.saturating_sub(prev) >= stride.get() => {
                *guard = Some(seq);
                true
            }
            _ => false,
        }
    }

    async fn force_consume(&self, seq: u64) {
        let mut guard = self.last_consumed.lock().await;
        *guard = Some(seq);
    }
}

fn collect_placeholder_ata_rewrites(
    placeholder: Pubkey,
    actual: Pubkey,
    route: &SwapRoute,
    plan: &LegPlan,
) -> Vec<(Pubkey, Pubkey)> {
    let mut rewrites = Vec::new();
    let mints = collect_candidate_mints(route);

    let token_programs = [
        Pubkey::new_from_array(spl_token::id().to_bytes()),
        Pubkey::new_from_array(spl_token_2022::id().to_bytes()),
    ];

    for mint in mints {
        for token_program in &token_programs {
            let original = cached_associated_token_address(&placeholder, &mint, token_program);
            if !plan_uses_account(plan, &original) {
                continue;
            }
            let replacement = cached_associated_token_address(&actual, &mint, token_program);
            rewrites.push((original, replacement));
        }
    }

    rewrites
}

fn strip_wsol_wrapping_instructions(plan: &mut LegPlan, payer: Pubkey) {
    const WSOL_MINT_STR: &str = "So11111111111111111111111111111111111111112";
    let wsol_mint = Pubkey::from_str(WSOL_MINT_STR).expect("valid WSOL mint");
    let spl_token_program_id =
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").expect("token program");
    let system_program_id =
        Pubkey::from_str("11111111111111111111111111111111").expect("system program");
    let wsol_ata = cached_associated_token_address(&payer, &wsol_mint, &spl_token_program_id);

    plan.instructions.retain(|ix| {
        !is_wsol_wrap_or_sync(ix, payer, wsol_ata, system_program_id, spl_token_program_id)
    });
}

fn is_sync_native_instruction(data: &[u8]) -> bool {
    matches!(data.first(), Some(&17))
}

fn is_wsol_wrap_or_sync(
    ix: &Instruction,
    payer: Pubkey,
    wsol_ata: Pubkey,
    system_program_id: Pubkey,
    spl_token_program_id: Pubkey,
) -> bool {
    if ix.program_id == system_program_id {
        return ix.accounts.len() >= 2
            && ix.accounts[0].pubkey == payer
            && ix.accounts[1].pubkey == wsol_ata;
    }

    if ix.program_id == spl_token_program_id
        && is_sync_native_instruction(&ix.data)
        && ix.accounts.first().map(|meta| meta.pubkey) == Some(wsol_ata)
    {
        return true;
    }

    ix.accounts
        .iter()
        .any(|meta| meta.pubkey == payer && meta.is_signer)
        && ix.accounts.iter().any(|meta| meta.pubkey == wsol_ata)
        && ix.data.get(0).copied().unwrap_or_default() == 12
}

fn strip_redundant_ata_creations(plan: &mut LegPlan, payer: Pubkey) {
    let ata_program_id = Pubkey::new_from_array(spl_associated_token_account::id().to_bytes());
    plan.instructions.retain(|ix| {
        if ix.program_id != ata_program_id {
            return true;
        }
        let opcode = ix.data.first().copied().unwrap_or_default();
        let is_create = opcode == 0 || opcode == 1;
        if !is_create {
            return true;
        }
        ix.accounts
            .first()
            .map(|meta| meta.pubkey != payer)
            .unwrap_or(true)
    });
}

fn collect_candidate_mints(route: &SwapRoute) -> HashSet<Pubkey> {
    let mut set = HashSet::new();
    for step in &route.steps {
        set.insert(step.input_mint);
        set.insert(step.output_mint);
        if let Some(fee_mint) = step.fee_mint {
            set.insert(fee_mint);
        }
    }
    set
}

fn plan_uses_account(plan: &LegPlan, target: &Pubkey) -> bool {
    plan.compute_budget_instructions
        .iter()
        .any(|ix| instruction_uses_account(ix, target))
        || plan
            .instructions
            .iter()
            .any(|ix| instruction_uses_account(ix, target))
}

fn instruction_uses_account(ix: &Instruction, target: &Pubkey) -> bool {
    ix.accounts.iter().any(|meta| meta.pubkey == *target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::titan::types::AccountMeta as TitanAccountMeta;
    use crate::engine::multi_leg::types::QuoteIntent;
    use solana_sdk::pubkey::Pubkey;
    use tokio::sync::Mutex;

    #[derive(Clone, Default, Debug)]
    struct MockSource {
        quote: Arc<Mutex<Option<TitanQuote>>>,
    }

    impl MockSource {
        fn with_quote(route: SwapRoute) -> Self {
            let quote = TitanQuote {
                route,
                provider: "Titan".to_string(),
                quote_id: Some("mock".into()),
            };
            Self {
                quote: Arc::new(Mutex::new(Some(quote))),
            }
        }
    }

    #[async_trait]
    impl TitanQuoteSource for MockSource {
        async fn quote(
            &self,
            _intent: &QuoteIntent,
            _side: LegSide,
            _local_ip: Option<IpAddr>,
        ) -> Result<TitanQuote, TitanLegError> {
            self.quote
                .lock()
                .await
                .take()
                .ok_or_else(|| TitanLegError::Source("no quote".into()))
        }
    }

    fn build_route() -> SwapRoute {
        let payer = Pubkey::new_unique();
        let account = Pubkey::new_unique();
        let compute_ix =
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);

        SwapRoute {
            in_amount: 100,
            out_amount: 99,
            slippage_bps: 10,
            platform_fee: None,
            steps: Vec::new(),
            instructions: vec![
                TitanInstruction {
                    program_id: compute_ix.program_id,
                    accounts: Vec::new(),
                    data: compute_ix.data,
                },
                TitanInstruction {
                    program_id: Pubkey::new_unique(),
                    accounts: vec![
                        TitanAccountMeta {
                            pubkey: payer,
                            signer: true,
                            writable: true,
                        },
                        TitanAccountMeta {
                            pubkey: account,
                            signer: false,
                            writable: false,
                        },
                    ],
                    data: vec![1, 2, 3],
                },
            ],
            address_lookup_tables: vec![Pubkey::new_unique()],
            context_slot: None,
            time_taken_ns: None,
            expires_at_ms: None,
            expires_after_slot: None,
            compute_units: None,
            compute_units_safe: None,
            transaction: None,
            reference_id: None,
        }
    }

    #[tokio::test]
    async fn titan_leg_provider_converts_instructions() {
        let placeholder = Pubkey::new_unique();
        let mut route = build_route();
        if let Some(ix) = route.instructions.get_mut(1) {
            if let Some(meta) = ix.accounts.get_mut(0) {
                meta.pubkey = placeholder;
            }
        }
        let intent = QuoteIntent::new(Pubkey::new_unique(), Pubkey::new_unique(), 100, 50);
        let provider = TitanLegProvider::new(
            MockSource::with_quote(route),
            LegSide::Buy,
            placeholder,
            false,
            {
                let mut allowed = HashSet::new();
                allowed.insert(TITAN_PROGRAM_ID);
                allowed
            },
            false,
        );

        let quote = provider.quote(&intent, None).await.expect("quote");
        let mut context = LegBuildContext::default();
        context.payer = Pubkey::new_unique();
        let plan = provider
            .build_plan(&quote, &context, None)
            .await
            .expect("plan");

        assert_eq!(plan.compute_budget_instructions.len(), 1);
        assert_eq!(plan.instructions.len(), 1);
        assert_eq!(plan.address_lookup_table_addresses.len(), 1);
        assert!(plan.raw_transaction.is_none());
        let rewrite = plan.signer_rewrite.expect("signer rewrite");
        assert_eq!(rewrite.original, placeholder);
        assert_eq!(rewrite.replacement, context.payer);
    }
}
pub(crate) const TITAN_PROGRAM_ID: Pubkey =
    pubkey!("T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT");
pub(crate) const METIS_PROGRAM_ID: Pubkey =
    pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub(crate) const OKX_PROGRAM_ID: Pubkey =
    pubkey!("proVF4pMXVaYqmy4NjniPh4pqKNfMmsihgd4wdkCX3u");
