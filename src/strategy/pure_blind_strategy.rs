use std::str::FromStr;

use anyhow::{Result, bail};
use rand::seq::SliceRandom;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::config;
use crate::dexes::{
    clmm::{RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAdapter},
    dlmm::{METEORA_DLMM_PROGRAM_ID, MeteoraDlmmAdapter},
    framework::{DexMarketMeta, DexMetaProvider, SwapFlow},
    humidifi::{HUMIDIFI_PROGRAM_ID, HumidiFiAdapter},
    obric_v2::{OBRIC_V2_PROGRAM_ID, ObricV2Adapter},
    solfi_v2::{SOLFI_V2_PROGRAM_ID, SolFiV2Adapter},
    tessera_v::{TESSERA_V_PROGRAM_ID, TesseraVAdapter},
    whirlpool::{ORCA_WHIRLPOOL_PROGRAM_ID, WhirlpoolAdapter},
    zerofi::{ZEROFI_PROGRAM_ID, ZeroFiAdapter},
};
use crate::engine::{Action, EngineError, EngineResult, StrategyContext};

use super::types::{BlindAsset, BlindDex, BlindMarketMeta, BlindOrder, BlindRoutePlan, BlindStep};
use super::{Strategy, StrategyEvent};

/// 纯盲发路由构建器：按配置解析盲发市场并生成双向路由。
pub struct PureBlindRouteBuilder<'a> {
    config: &'a config::BlindStrategyConfig,
    rpc_client: &'a RpcClient,
}

impl<'a> PureBlindRouteBuilder<'a> {
    pub fn new(config: &'a config::BlindStrategyConfig, rpc_client: &'a RpcClient) -> Self {
        Self { config, rpc_client }
    }

    pub async fn build(&self) -> EngineResult<Vec<BlindRoutePlan>> {
        if self.config.pure_routes.is_empty() {
            return Err(EngineError::InvalidConfig(
                "pure_mode 已开启，但 blind_strategy.pure_routes 为空".into(),
            ));
        }

        let mut plans = Vec::with_capacity(self.config.pure_routes.len());

        for route in &self.config.pure_routes {
            plans.push(self.build_plan(route).await?);
        }

        Ok(plans)
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn build_plan(
        &self,
        route: &config::PureBlindRouteConfig,
    ) -> EngineResult<BlindRoutePlan> {
        let route_label = route.name.as_deref().unwrap_or("<pure_route>");

        if route.legs.len() < 2 {
            return Err(EngineError::InvalidConfig(format!(
                "pure_routes `{route_label}` 至少需要 2 条腿"
            )));
        }

        let mut markets = Vec::with_capacity(route.legs.len());
        for (idx, leg) in route.legs.iter().enumerate() {
            let market = Pubkey::from_str(leg.market.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_routes `{route_label}` 第 {idx} 条腿 market `{}` 解析失败: {err}",
                    leg.market
                ))
            })?;
            markets.push(market);
        }

        let accounts = self
            .rpc_client
            .get_multiple_accounts(&markets)
            .await
            .map_err(EngineError::Rpc)?;

        let mut resolved = Vec::with_capacity(markets.len());
        for (idx, market) in markets.iter().enumerate() {
            let account = accounts
                .get(idx)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| {
                    EngineError::InvalidConfig(format!(
                        "pure_routes `{route_label}` 第 {idx} 条腿 market `{market}` 不存在"
                    ))
                })?;

            let meta = self.resolve_market_meta(*market, account).await?;
            resolved.push(meta);
        }

        let forward = Self::build_closed_loop(&resolved).ok_or_else(|| {
            EngineError::InvalidConfig(format!(
                "pure_routes `{route_label}` 无法推导闭环资产流，请检查腿顺序与市场配置"
            ))
        })?;
        let reverse = Self::build_reverse_steps(&forward);

        Ok(BlindRoutePlan { forward, reverse })
    }

    fn build_closed_loop(resolved: &[ResolvedMarketMeta]) -> Option<Vec<BlindStep>> {
        if resolved.len() < 2 {
            return None;
        }

        Self::try_orientation(resolved, SwapFlow::BaseToQuote)
            .or_else(|| Self::try_orientation(resolved, SwapFlow::QuoteToBase))
    }

    fn try_orientation(
        resolved: &[ResolvedMarketMeta],
        first_flow: SwapFlow,
    ) -> Option<Vec<BlindStep>> {
        let mut steps = Vec::with_capacity(resolved.len());

        let first = resolved.first()?;
        let (first_input, first_output) = Self::assets_for(first, first_flow);
        let mut current_asset = first_input.clone();
        let origin_asset = current_asset.clone();

        steps.push(BlindStep {
            dex: first.dex,
            market: first.market,
            base: first.base_asset.clone(),
            quote: first.quote_asset.clone(),
            input: first_input.clone(),
            output: first_output.clone(),
            meta: first.meta.clone(),
            flow: first_flow,
        });

        current_asset = first_output;

        for meta in resolved.iter().skip(1) {
            let flow = if current_asset == meta.base_asset {
                SwapFlow::BaseToQuote
            } else if current_asset == meta.quote_asset {
                SwapFlow::QuoteToBase
            } else {
                return None;
            };

            let (input_asset, output_asset) = Self::assets_for(meta, flow);
            if input_asset != current_asset {
                return None;
            }

            steps.push(BlindStep {
                dex: meta.dex,
                market: meta.market,
                base: meta.base_asset.clone(),
                quote: meta.quote_asset.clone(),
                input: input_asset.clone(),
                output: output_asset.clone(),
                meta: meta.meta.clone(),
                flow,
            });

            current_asset = output_asset;
        }

        if current_asset != origin_asset {
            return None;
        }

        Some(steps)
    }

    fn build_reverse_steps(forward: &[BlindStep]) -> Vec<BlindStep> {
        forward
            .iter()
            .rev()
            .map(|step| BlindStep {
                dex: step.dex,
                market: step.market,
                base: step.base.clone(),
                quote: step.quote.clone(),
                input: step.output.clone(),
                output: step.input.clone(),
                meta: step.meta.clone(),
                flow: match step.flow {
                    SwapFlow::BaseToQuote => SwapFlow::QuoteToBase,
                    SwapFlow::QuoteToBase => SwapFlow::BaseToQuote,
                },
            })
            .collect()
    }

    fn assets_for(meta: &ResolvedMarketMeta, flow: SwapFlow) -> (BlindAsset, BlindAsset) {
        match flow {
            SwapFlow::BaseToQuote => (meta.base_asset.clone(), meta.quote_asset.clone()),
            SwapFlow::QuoteToBase => (meta.quote_asset.clone(), meta.base_asset.clone()),
        }
    }

    async fn resolve_market_meta(
        &self,
        market: Pubkey,
        account: &Account,
    ) -> EngineResult<ResolvedMarketMeta> {
        if account.owner == ZEROFI_PROGRAM_ID {
            let adapter = ZeroFiAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("ZeroFi 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::ZeroFi,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::ZeroFi(meta),
            });
        }

        if account.owner == SOLFI_V2_PROGRAM_ID {
            let adapter = SolFiV2Adapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("SolFiV2 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::SolFiV2,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::SolFiV2(meta),
            });
        }

        if account.owner == TESSERA_V_PROGRAM_ID {
            let adapter = TesseraVAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("TesseraV 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::TesseraV,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::TesseraV(meta),
            });
        }

        if account.owner == HUMIDIFI_PROGRAM_ID {
            let adapter = HumidiFiAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("HumidiFi 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::HumidiFi,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::HumidiFi(meta),
            });
        }

        if account.owner == OBRIC_V2_PROGRAM_ID {
            let adapter = ObricV2Adapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("ObricV2 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::ObricV2,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::ObricV2(meta),
            });
        }

        if account.owner == RAYDIUM_CLMM_PROGRAM_ID {
            let adapter = RaydiumClmmAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("RaydiumClmm 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::RaydiumClmm,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::RaydiumClmm(meta),
            });
        }

        if account.owner == METEORA_DLMM_PROGRAM_ID {
            let adapter = MeteoraDlmmAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("MeteoraDlmm 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::MeteoraDlmm,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::MeteoraDlmm(meta),
            });
        }

        if account.owner == ORCA_WHIRLPOOL_PROGRAM_ID {
            let adapter = WhirlpoolAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("Whirlpool 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::Whirlpool,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::Whirlpool(meta),
            });
        }

        Err(EngineError::InvalidConfig(format!(
            "纯盲发暂不支持程序 {}",
            account.owner
        )))
    }
}

#[derive(Clone)]
struct ResolvedMarketMeta {
    dex: BlindDex,
    market: Pubkey,
    base_asset: BlindAsset,
    quote_asset: BlindAsset,
    meta: BlindMarketMeta,
}

/// 纯盲发策略：不依赖报价，直接构造 route_v2 指令。
pub struct PureBlindStrategy {
    routes: Vec<BlindRoutePlan>,
}

impl PureBlindStrategy {
    pub fn new(routes: Vec<BlindRoutePlan>) -> Result<Self> {
        if routes.is_empty() {
            bail!("纯盲发模式需要至少一个盲发路由");
        }

        Ok(Self { routes })
    }
}

impl Strategy for PureBlindStrategy {
    type Event = StrategyEvent;

    fn name(&self) -> &'static str {
        "pure_blind"
    }

    fn on_market_event(&mut self, event: &Self::Event, ctx: StrategyContext<'_>) -> Action {
        match event {
            StrategyEvent::Tick(_) => {
                let trade_amounts = ctx.trade_amounts();
                if trade_amounts.is_empty() {
                    return Action::Idle;
                }

                let mut batch = Vec::with_capacity(self.routes.len() * trade_amounts.len() * 2);

                for route in &self.routes {
                    for &amount in trade_amounts {
                        batch.push(BlindOrder {
                            amount_in: amount,
                            steps: route.forward.clone(),
                        });
                        batch.push(BlindOrder {
                            amount_in: amount,
                            steps: route.reverse.clone(),
                        });
                    }
                }

                if batch.is_empty() {
                    return Action::Idle;
                }

                let mut rng = rand::rng();
                batch.shuffle(&mut rng);

                Action::DispatchBlind(batch)
            }
        }
    }
}
