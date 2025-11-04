use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc;
use tokio::time::{Interval, interval};
use tracing::warn;

use crate::dexes::clmm::{RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAdapter};
use crate::dexes::dlmm::{METEORA_DLMM_PROGRAM_ID, MeteoraDlmmAdapter};
use crate::dexes::framework::{DexMarketMeta, DexMetaProvider, SwapFlow};
use crate::dexes::humidifi::{HUMIDIFI_PROGRAM_ID, HumidiFiAdapter};
use crate::dexes::obric_v2::{OBRIC_V2_PROGRAM_ID, ObricV2Adapter};
use crate::dexes::saros::{SAROS_PROGRAM_ID, SarosAdapter};
use crate::dexes::solfi_v2::{SOLFI_V2_PROGRAM_ID, SolFiV2Adapter};
use crate::dexes::tessera_v::{TESSERA_V_PROGRAM_ID, TesseraVAdapter};
use crate::dexes::whirlpool::{ORCA_WHIRLPOOL_PROGRAM_ID, WhirlpoolAdapter};
use crate::dexes::zerofi::{ZEROFI_PROGRAM_ID, ZeroFiAdapter};
use crate::strategy::types::{BlindAsset, BlindDex, BlindMarketMeta, BlindStep};

use super::observer::{
    DeactivateReason, PoolCatalog, PoolCatalogEvent, PoolKey, PoolProfile, PoolStatsSnapshot,
};

#[derive(Debug)]
pub enum DynamicRouteUpdate {
    Activated {
        profile: Arc<PoolProfile>,
        stats: PoolStatsSnapshot,
        steps: Vec<BlindStep>,
    },
    Retired {
        key: PoolKey,
        _reason: DeactivateReason,
    },
}

pub fn spawn_dynamic_worker(
    catalog: Arc<PoolCatalog>,
    rpc_client: Arc<RpcClient>,
    decay_duration: Duration,
) -> mpsc::UnboundedReceiver<DynamicRouteUpdate> {
    let mut receiver = catalog.subscribe();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut decay_interval = if decay_duration.is_zero() {
        None
    } else {
        Some(interval(decay_duration))
    };

    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = maybe_tick(&mut decay_interval) => {
                    catalog.enforce_decay();
                }
                event = receiver.recv() => {
                    match event {
                        Ok(event) => {
                            if let Err(err) = handle_event(&tx, event, Arc::clone(&rpc_client)).await {
                                warn!(
                                    target: "pure_blind::dynamic",
                                    error = %err,
                                    "处理池子事件失败"
                                );
                            }
                        }
                        Err(err) => {
                            match err {
                                tokio::sync::broadcast::error::RecvError::Closed => break,
                                tokio::sync::broadcast::error::RecvError::Lagged(skipped) => {
                                    warn!(
                                        target: "pure_blind::dynamic",
                                        skipped,
                                        "动态池子事件滞后，跳过 {} 条记录",
                                        skipped
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    rx
}

async fn maybe_tick(interval: &mut Option<Interval>) {
    if let Some(interval) = interval {
        interval.tick().await;
    } else {
        tokio::task::yield_now().await;
    }
}

async fn handle_event(
    tx: &mpsc::UnboundedSender<DynamicRouteUpdate>,
    event: PoolCatalogEvent,
    rpc_client: Arc<RpcClient>,
) -> Result<()> {
    match event {
        PoolCatalogEvent::Activated { profile, stats } => {
            match build_steps(Arc::clone(&profile), rpc_client).await {
                Ok(steps) => {
                    let update = DynamicRouteUpdate::Activated {
                        profile,
                        stats,
                        steps,
                    };
                    let _ = tx.send(update);
                }
                Err(err) => {
                    warn!(
                        target: "pure_blind::dynamic",
                        error = %err,
                        dex = profile.key.dex_label,
                        pool = ?profile.key.pool_address,
                        "池子激活但构造动态路线失败"
                    );
                }
            }
        }
        PoolCatalogEvent::Deactivated {
            profile,
            reason: _reason,
            ..
        } => {
            let update = DynamicRouteUpdate::Retired {
                key: profile.key.clone(),
                _reason,
            };
            let _ = tx.send(update);
        }
    }
    Ok(())
}

async fn build_steps(
    profile: Arc<PoolProfile>,
    rpc_client: Arc<RpcClient>,
) -> Result<Vec<BlindStep>> {
    let pool_address = profile
        .key
        .pool_address
        .ok_or_else(|| anyhow!("pool profile 缺少池子地址"))?;
    let program = profile
        .key
        .dex_program
        .ok_or_else(|| anyhow!("pool profile 缺少 DEX 程序号"))?;

    let account = rpc_client
        .get_account(&pool_address)
        .await
        .with_context(|| format!("获取池子账户失败: {pool_address}"))?;

    let resolved = resolve_market_meta(&rpc_client, program, pool_address, account).await?;
    let (flow, input_asset, output_asset) =
        resolve_flow(&profile, &resolved.base_asset, &resolved.quote_asset)?;

    let step = BlindStep {
        dex: resolved.dex,
        market: pool_address,
        base: resolved.base_asset.clone(),
        quote: resolved.quote_asset.clone(),
        input: input_asset,
        output: output_asset,
        meta: resolved.meta.clone(),
        flow,
    };

    Ok(vec![step])
}

struct ResolvedMeta {
    dex: BlindDex,
    meta: BlindMarketMeta,
    base_asset: BlindAsset,
    quote_asset: BlindAsset,
}

async fn resolve_market_meta(
    rpc_client: &RpcClient,
    program: Pubkey,
    market: Pubkey,
    account: Account,
) -> Result<ResolvedMeta> {
    if program == ZEROFI_PROGRAM_ID {
        let adapter = ZeroFiAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("ZeroFi 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::ZeroFi,
            meta: BlindMarketMeta::ZeroFi(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == SAROS_PROGRAM_ID {
        let adapter = SarosAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("Saros 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::Saros,
            meta: BlindMarketMeta::Saros(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == HUMIDIFI_PROGRAM_ID {
        let adapter = HumidiFiAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("HumidiFi 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::HumidiFi,
            meta: BlindMarketMeta::HumidiFi(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == OBRIC_V2_PROGRAM_ID {
        let adapter = ObricV2Adapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("ObricV2 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::ObricV2,
            meta: BlindMarketMeta::ObricV2(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == RAYDIUM_CLMM_PROGRAM_ID {
        let adapter = RaydiumClmmAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("RaydiumClmm 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::RaydiumClmm,
            meta: BlindMarketMeta::RaydiumClmm(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == METEORA_DLMM_PROGRAM_ID {
        let adapter = MeteoraDlmmAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("MeteoraDlmm 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::MeteoraDlmm,
            meta: BlindMarketMeta::MeteoraDlmm(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == ORCA_WHIRLPOOL_PROGRAM_ID {
        let adapter = WhirlpoolAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("Whirlpool 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::Whirlpool,
            meta: BlindMarketMeta::Whirlpool(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == SOLFI_V2_PROGRAM_ID {
        let adapter = SolFiV2Adapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("SolFiV2 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::SolFiV2,
            meta: BlindMarketMeta::SolFiV2(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    if program == TESSERA_V_PROGRAM_ID {
        let adapter = TesseraVAdapter::shared();
        let meta = adapter
            .fetch_market_meta(rpc_client, market, &account)
            .await
            .context("TesseraV 市场解码失败")?;
        return Ok(ResolvedMeta {
            dex: BlindDex::TesseraV,
            meta: BlindMarketMeta::TesseraV(meta.clone()),
            base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
            quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
        });
    }

    Err(anyhow!("不支持的 DEX 程序 {}", program))
}

fn resolve_flow(
    profile: &PoolProfile,
    base_asset: &BlindAsset,
    quote_asset: &BlindAsset,
) -> Result<(SwapFlow, BlindAsset, BlindAsset)> {
    let input_mint = profile
        .key
        .input_mint
        .ok_or_else(|| anyhow!("pool profile 缺少输入 mint"))?;
    let output_mint = profile
        .key
        .output_mint
        .ok_or_else(|| anyhow!("pool profile 缺少输出 mint"))?;

    if input_mint == base_asset.mint && output_mint == quote_asset.mint {
        return Ok((
            SwapFlow::BaseToQuote,
            base_asset.clone(),
            quote_asset.clone(),
        ));
    }

    if input_mint == quote_asset.mint && output_mint == base_asset.mint {
        return Ok((
            SwapFlow::QuoteToBase,
            quote_asset.clone(),
            base_asset.clone(),
        ));
    }

    Err(anyhow!(
        "pool profile 输入/输出 mint 与市场资产不匹配 (input={}, output={}, base={}, quote={})",
        input_mint,
        output_mint,
        base_asset.mint,
        quote_asset.mint,
    ))
}
