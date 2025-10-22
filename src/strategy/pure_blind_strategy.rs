use std::str::FromStr;

use anyhow::{Result, bail};
use rand::seq::SliceRandom;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::config;
use crate::dexes::{
    framework::{DexMarketMeta, DexMetaProvider},
    humidifi::{HUMIDIFI_PROGRAM_ID, HumidiFiAdapter},
    obric_v2::{OBRIC_V2_PROGRAM_ID, ObricV2Adapter},
    solfi_v2::{SOLFI_V2_PROGRAM_ID, SolFiV2Adapter},
    tessera_v::{TESSERA_V_PROGRAM_ID, TesseraVAdapter},
    zerofi::{ZEROFI_PROGRAM_ID, ZeroFiAdapter},
};
use crate::engine::{Action, EngineError, EngineResult, StrategyContext};

use super::types::{
    BlindDex, BlindMarketMeta, BlindOrder, BlindRoutePlan, BlindStep, BlindSwapDirection,
};
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
            let buy_market = Pubkey::from_str(route.buy_market.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "blind_strategy.pure_routes.buy_market `{}` 解析失败: {err}",
                    route.buy_market
                ))
            })?;
            let sell_market = Pubkey::from_str(route.sell_market.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "blind_strategy.pure_routes.sell_market `{}` 解析失败: {err}",
                    route.sell_market
                ))
            })?;

            let accounts = self
                .rpc_client
                .get_multiple_accounts(&[buy_market, sell_market])
                .await
                .map_err(EngineError::Rpc)?;

            let buy_account = accounts
                .get(0)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| EngineError::InvalidConfig(format!("市场 {buy_market} 不存在")))?;
            let sell_account = accounts
                .get(1)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| EngineError::InvalidConfig(format!("市场 {sell_market} 不存在")))?;

            let buy_meta = self.resolve_market_meta(buy_market, buy_account).await?;
            let sell_meta = self.resolve_market_meta(sell_market, sell_account).await?;

            if buy_meta.base_mint != sell_meta.base_mint
                || buy_meta.quote_mint != sell_meta.quote_mint
            {
                return Err(EngineError::InvalidConfig(format!(
                    "市场 {buy_market} 与 {sell_market} 的基础/计价代币不一致",
                )));
            }

            let forward = vec![
                build_blind_step(&sell_meta, sell_market, BlindSwapDirection::BaseToQuote),
                build_blind_step(&buy_meta, buy_market, BlindSwapDirection::QuoteToBase),
            ];

            let reverse = vec![
                build_blind_step(&buy_meta, buy_market, BlindSwapDirection::BaseToQuote),
                build_blind_step(&sell_meta, sell_market, BlindSwapDirection::QuoteToBase),
            ];

            plans.push(BlindRoutePlan { forward, reverse });
        }

        Ok(plans)
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
                base_mint: meta.base_mint(),
                quote_mint: meta.quote_mint(),
                base_token_program: meta.base_token_program(),
                quote_token_program: meta.quote_token_program(),
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
                base_mint: meta.base_mint(),
                quote_mint: meta.quote_mint(),
                base_token_program: meta.base_token_program(),
                quote_token_program: meta.quote_token_program(),
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
                base_mint: meta.base_mint(),
                quote_mint: meta.quote_mint(),
                base_token_program: meta.base_token_program(),
                quote_token_program: meta.quote_token_program(),
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
                base_mint: meta.base_mint(),
                quote_mint: meta.quote_mint(),
                base_token_program: meta.base_token_program(),
                quote_token_program: meta.quote_token_program(),
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
                base_mint: meta.base_mint(),
                quote_mint: meta.quote_mint(),
                base_token_program: meta.base_token_program(),
                quote_token_program: meta.quote_token_program(),
                meta: BlindMarketMeta::ObricV2(meta),
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
    base_mint: Pubkey,
    quote_mint: Pubkey,
    base_token_program: Pubkey,
    quote_token_program: Pubkey,
    meta: BlindMarketMeta,
}

fn build_blind_step(
    resolved: &ResolvedMarketMeta,
    market: Pubkey,
    direction: BlindSwapDirection,
) -> BlindStep {
    BlindStep {
        dex: resolved.dex,
        market,
        base_mint: resolved.base_mint,
        quote_mint: resolved.quote_mint,
        base_token_program: resolved.base_token_program,
        quote_token_program: resolved.quote_token_program,
        meta: resolved.meta.clone(),
        direction,
    }
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
