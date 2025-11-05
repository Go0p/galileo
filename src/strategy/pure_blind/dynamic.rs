use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::message::AddressLookupTableAccount;
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
    PoolProfile, RouteCatalog, RouteCatalogEvent, RouteDeactivateReason, RouteKey, RouteProfile,
    RouteStatsSnapshot,
};

#[derive(Debug)]
pub enum DynamicRouteUpdate {
    Activated {
        profile: Arc<RouteProfile>,
        stats: RouteStatsSnapshot,
        steps: Vec<BlindStep>,
        lookup_tables: Vec<AddressLookupTableAccount>,
    },
    Retired {
        key: RouteKey,
        _reason: RouteDeactivateReason,
    },
}

pub fn spawn_dynamic_worker(
    catalog: Arc<RouteCatalog>,
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
                                    "处理路线事件失败"
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
    event: RouteCatalogEvent,
    rpc_client: Arc<RpcClient>,
) -> Result<()> {
    match event {
        RouteCatalogEvent::Activated { profile, stats } => {
            match build_dynamic_route(Arc::clone(&profile), rpc_client).await {
                Ok(payload) => {
                    let update = DynamicRouteUpdate::Activated {
                        profile,
                        stats,
                        steps: payload.steps,
                        lookup_tables: payload.lookup_tables,
                    };
                    let _ = tx.send(update);
                }
                Err(err) => {
                    warn!(
                        target: "pure_blind::dynamic",
                        error = %err,
                        markets = ?profile.markets(),
                        "路线激活但构造动态闭环失败"
                    );
                }
            }
        }
        RouteCatalogEvent::Deactivated {
            profile, reason, ..
        } => {
            let update = DynamicRouteUpdate::Retired {
                key: profile.key.clone(),
                _reason: reason,
            };
            let _ = tx.send(update);
        }
    }
    Ok(())
}

struct DynamicRoutePayload {
    steps: Vec<BlindStep>,
    lookup_tables: Vec<AddressLookupTableAccount>,
}

async fn build_dynamic_route(
    profile: Arc<RouteProfile>,
    rpc_client: Arc<RpcClient>,
) -> Result<DynamicRoutePayload> {
    if profile.steps.is_empty() {
        return Err(anyhow!("route profile 缺少步骤"));
    }

    let mut steps = Vec::with_capacity(profile.steps.len());
    for pool_profile in profile.steps.iter() {
        let pool_address = pool_profile
            .key
            .pool_address
            .ok_or_else(|| anyhow!("route 步骤缺少池子地址"))?;
        let program = pool_profile
            .key
            .dex_program
            .ok_or_else(|| anyhow!("route 步骤缺少 DEX 程序号"))?;

        let account = rpc_client
            .get_account(&pool_address)
            .await
            .with_context(|| format!("获取池子账户失败: {pool_address}"))?;

        let resolved = resolve_market_meta(&rpc_client, program, pool_address, account).await?;
        let (flow, input_asset, output_asset) =
            resolve_flow(pool_profile, &resolved.base_asset, &resolved.quote_asset)?;

        steps.push(BlindStep {
            dex: resolved.dex,
            market: pool_address,
            base: resolved.base_asset.clone(),
            quote: resolved.quote_asset.clone(),
            input: input_asset,
            output: output_asset,
            meta: resolved.meta.clone(),
            flow,
        });
    }

    let lookup_tables = fetch_lookup_tables(&rpc_client, profile.lookup_tables.as_ref()).await?;

    Ok(DynamicRoutePayload {
        steps,
        lookup_tables,
    })
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
    if let (Some(input), Some(output)) = (&profile.input_asset, &profile.output_asset) {
        if input.mint == base_asset.mint && output.mint == quote_asset.mint {
            return Ok((
                SwapFlow::BaseToQuote,
                BlindAsset::new(input.mint, input.token_program),
                BlindAsset::new(output.mint, output.token_program),
            ));
        }
        if input.mint == quote_asset.mint && output.mint == base_asset.mint {
            return Ok((
                SwapFlow::QuoteToBase,
                BlindAsset::new(input.mint, input.token_program),
                BlindAsset::new(output.mint, output.token_program),
            ));
        }
    }

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

async fn fetch_lookup_tables(
    rpc_client: &RpcClient,
    tables: &[Pubkey],
) -> Result<Vec<AddressLookupTableAccount>> {
    if tables.is_empty() {
        return Ok(Vec::new());
    }

    let mut unique: Vec<Pubkey> = Vec::with_capacity(tables.len());
    let mut seen = HashSet::new();
    for table in tables {
        if seen.insert(*table) {
            unique.push(*table);
        }
    }

    if unique.is_empty() {
        return Ok(Vec::new());
    }

    let accounts = rpc_client
        .get_multiple_accounts(&unique)
        .await
        .context("获取地址查找表账户失败")?;

    let mut resolved = Vec::new();
    for (key, maybe_account) in unique.into_iter().zip(accounts.into_iter()) {
        let Some(account) = maybe_account else {
            continue;
        };
        let table = AddressLookupTable::deserialize(&account.data)
            .with_context(|| format!("解析地址查找表失败: {key}"))?;
        resolved.push(AddressLookupTableAccount {
            key,
            addresses: table.addresses.into_owned(),
        });
    }

    Ok(resolved)
}
