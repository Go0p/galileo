use std::collections::{HashMap, HashSet};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{error, info, warn};

use crate::config::{PureBlindAssetsConfig, PureBlindMarketCacheConfig};
use crate::dexes::clmm::RAYDIUM_CLMM_PROGRAM_ID;
use crate::dexes::dlmm::METEORA_DLMM_PROGRAM_ID;
use crate::dexes::humidifi::HUMIDIFI_PROGRAM_ID;
use crate::dexes::obric_v2::OBRIC_V2_PROGRAM_ID;
use crate::dexes::solfi_v2::SOLFI_V2_PROGRAM_ID;
use crate::dexes::tessera_v::TESSERA_V_PROGRAM_ID;
use crate::dexes::whirlpool::ORCA_WHIRLPOOL_PROGRAM_ID;
use crate::dexes::zerofi::ZEROFI_PROGRAM_ID;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MarketRecord {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub lookup_table: Option<Pubkey>,
    pub routing_group: Option<u8>,
    pub swap_account_size: Option<SwapAccountSize>,
    pub token_mints: Vec<Pubkey>,
    pub liquidity_usd: Option<f64>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SwapAccountSize {
    pub account_len: u64,
    pub account_metas_count: u64,
    pub account_compressed_count: u64,
}

pub struct MarketCacheHandle {
    data: Arc<RwLock<Vec<MarketRecord>>>,
    _task: Option<JoinHandle<()>>,
}

#[allow(dead_code)]
impl MarketCacheHandle {
    pub fn snapshot(&self) -> Vec<MarketRecord> {
        self.try_snapshot().unwrap_or_default()
    }

    pub fn try_snapshot(&self) -> Option<Vec<MarketRecord>> {
        self.data.try_read().ok().map(|guard| guard.clone())
    }
}

pub async fn init_market_cache(
    config: &PureBlindMarketCacheConfig,
    assets: &PureBlindAssetsConfig,
    client: Client,
) -> Result<MarketCacheHandle> {
    let data = Arc::new(RwLock::new(Vec::new()));
    let settings = MarketCacheSettings::from_configs(config, assets);
    if settings.local_path.as_os_str().is_empty() {
        return Err(anyhow!("pure_blind_strategy.market_cache.path 不能为空"));
    }
    let download_client = if let Some(proxy_url) = &settings.download_proxy {
        info!(
            target: "pure_blind::market_cache",
            proxy = %proxy_url,
            "纯盲发市场缓存下载客户端已启用代理"
        );
        let proxy = reqwest::Proxy::all(proxy_url)
            .with_context(|| format!("pure_blind_strategy.market_cache.proxy 无效: {proxy_url}"))?;
        reqwest::Client::builder()
            .proxy(proxy)
            .danger_accept_invalid_certs(true)
            .build()
            .with_context(|| format!("构建纯盲发市场缓存下载客户端失败 (proxy={proxy_url})"))?
    } else {
        client.clone()
    };

    let loader = MarketCacheLoader {
        settings: settings.clone(),
        client: download_client,
        data: data.clone(),
    };

    loader.load_once(true).await?;

    let task = if settings.auto_refresh_minutes > 0 {
        let mut interval = time::interval(Duration::from_secs(settings.auto_refresh_minutes * 60));
        let loader_clone = loader.clone();
        Some(tokio::spawn(async move {
            interval.tick().await; // skip immediate tick
            loop {
                interval.tick().await;
                if let Err(err) = loader_clone.load_once(true).await {
                    error!(
                        target: "pure_blind::market_cache",
                        error = %err,
                        "刷新纯盲发市场缓存失败"
                    );
                }
            }
        }))
    } else {
        None
    };

    Ok(MarketCacheHandle { data, _task: task })
}

#[derive(Clone)]
struct MarketCacheLoader {
    settings: MarketCacheSettings,
    client: Client,
    data: Arc<RwLock<Vec<MarketRecord>>>,
}

impl MarketCacheLoader {
    async fn load_once(&self, allow_download: bool) -> Result<()> {
        let should_attempt_download = allow_download || !self.settings.local_path.exists();
        let downloaded = if should_attempt_download {
            self.download_if_needed().await?
        } else {
            None
        };

        let records = if let Some(raw_bytes) = downloaded {
            let raw_records: Vec<RawMarketRecord> = serde_json::from_slice(&raw_bytes)
                .with_context(|| "解析市场缓存 JSON 失败（下载数据无法解析）")?;
            let filtered = self.filter_records(raw_records)?;
            self.write_filtered_cache(&filtered).await?;
            filtered
        } else {
            let bytes = fs::read(&self.settings.local_path).await.with_context(|| {
                format!("读取市场缓存失败: {}", self.settings.local_path.display())
            })?;
            match serde_json::from_slice::<Vec<PersistedMarketRecord>>(&bytes) {
                Ok(stored) => stored
                    .into_iter()
                    .map(MarketRecord::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|| {
                        format!(
                            "解析已过滤市场缓存失败: {}",
                            self.settings.local_path.display()
                        )
                    })?,
                Err(_) => {
                    let raw_records: Vec<RawMarketRecord> = serde_json::from_slice(&bytes)
                        .with_context(|| {
                            format!(
                                "解析市场缓存 JSON 失败: {}",
                                self.settings.local_path.display()
                            )
                        })?;
                    let filtered = self.filter_records(raw_records)?;
                    self.write_filtered_cache(&filtered).await?;
                    filtered
                }
            }
        };

        {
            let mut guard = self.data.write().await;
            *guard = records.clone();
        }

        let count = records.len();
        let path = self.settings.local_path.display().to_string();
        info!(
            target: "pure_blind::market_cache",
            path = %path,
            count,
            "纯盲发市场缓存已更新"
        );

        Ok(())
    }

    async fn download_if_needed(&self) -> Result<Option<Vec<u8>>> {
        if self.settings.download_url.is_empty() {
            if self.settings.local_path.exists() {
                return Ok(None);
            }
            return Err(anyhow!(
                "pure_blind_strategy.market_cache.path 不存在，且未配置 download_url"
            ));
        }

        let target_path = self.settings.local_path.as_path();
        info!(
            target: "pure_blind::market_cache",
            url = %self.settings.download_url,
            path = %target_path.display(),
            proxy = %self
                .settings
                .download_proxy
                .as_deref()
                .unwrap_or("<none>"),
            "开始下载纯盲发市场缓存"
        );

        let response = self
            .client
            .get(&self.settings.download_url)
            .send()
            .await
            .with_context(|| {
                format!(
                    "下载市场缓存失败: url={} path={}",
                    self.settings.download_url,
                    target_path.display()
                )
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!(
                "下载市场缓存失败: url={} status={}",
                self.settings.download_url,
                status
            ));
        }

        let total_size = response.content_length().unwrap_or_default();
        let has_known_size = total_size > 0;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let display_name = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("market-cache")
            .to_string();

        let mut pb = create_download_progress_bar(target_path, total_size);

        let mut data = Vec::with_capacity(usize::try_from(total_size).unwrap_or(0).max(1024));

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("读取市场缓存响应失败")?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            data.extend_from_slice(&chunk);
            if let Some(ref pb) = pb {
                if has_known_size {
                    pb.set_position(downloaded.min(total_size));
                } else {
                    pb.set_message(format!("下载 {display_name} ({})", HumanBytes(downloaded)));
                }
            }
        }

        if let Some(pb) = pb.take() {
            let finish_msg = if has_known_size {
                format!("下载完成 {display_name}")
            } else {
                format!("下载完成 {display_name} ({})", HumanBytes(downloaded))
            };
            pb.finish_with_message(finish_msg);
        }

        info!(
            target: "pure_blind::market_cache",
            path = %target_path.display(),
            size_bytes = downloaded,
            size_display = %HumanBytes(downloaded),
            "纯盲发市场缓存已下载"
        );

        Ok(Some(data))
    }

    fn filter_records(&self, records: Vec<RawMarketRecord>) -> Result<Vec<MarketRecord>> {
        let supported = supported_program_ids();
        let mut filtered = Vec::new();
        let mut dex_counter: HashMap<&'static str, usize> = HashMap::new();
        let mut mint_counter: HashMap<String, usize> = HashMap::new();
        let has_mint_filter =
            !(self.settings.base_mints.is_empty() && self.settings.intermediates.is_empty());
        for record in records {
            let market = match Pubkey::from_str(&record.pubkey) {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        target: "pure_blind::market_cache",
                        pubkey = record.pubkey,
                        error = %err,
                        "市场公钥解析失败，已跳过"
                    );
                    continue;
                }
            };
            let owner = match Pubkey::from_str(&record.owner) {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        target: "pure_blind::market_cache",
                        owner = record.owner,
                        market = %market,
                        error = %err,
                        "市场 owner 公钥解析失败，已跳过"
                    );
                    continue;
                }
            };

            if self.settings.exclude_other_programs && !supported.contains(&owner) {
                continue;
            }

            if self.settings.exclude_programs.contains(&owner) {
                continue;
            }

            let params = record.params.clone();

            let (routing_group, swap_account_size, token_mints, liquidity_usd, lookup_table) =
                if let Some(params) = params {
                    let lookup_table = params
                        .address_lookup_table_address
                        .as_deref()
                        .and_then(|value| Pubkey::from_str(value).ok());
                    let routing_group = params.routing_group;
                    let swap_account_size = params.swap_account_size.map(Into::into);
                    let liquidity_usd = params.liquidity_usd;
                    let mut token_mints = Vec::with_capacity(params.token_mints.len());
                    for mint in params.token_mints {
                        let trimmed = mint.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        match Pubkey::from_str(trimmed) {
                            Ok(value) => token_mints.push(value),
                            Err(err) => {
                                warn!(
                                    target: "pure_blind::market_cache",
                                    market = %market,
                                    mint = trimmed,
                                    error = %err,
                                    "市场缓存 token mint 解析失败，已跳过"
                                );
                            }
                        }
                    }
                    (
                        routing_group,
                        swap_account_size,
                        token_mints,
                        liquidity_usd,
                        lookup_table,
                    )
                } else {
                    (None, None, Vec::new(), None, None)
                };

            if has_mint_filter && !token_mints.is_empty() {
                let keep = token_mints.iter().all(|mint| {
                    self.settings.base_mints.contains(mint)
                        || self.settings.intermediates.contains(mint)
                });
                if !keep {
                    continue;
                }

                let connects_base = token_mints
                    .iter()
                    .any(|mint| self.settings.base_mints.contains(mint));
                let connects_intermediate = token_mints
                    .iter()
                    .any(|mint| self.settings.intermediates.contains(mint));
                if !connects_base && !connects_intermediate {
                    continue;
                }
            }

            filtered.push(MarketRecord {
                market,
                owner,
                lookup_table,
                routing_group,
                swap_account_size,
                token_mints: token_mints.clone(),
                liquidity_usd,
            });

            let label = dex_label_for_owner(owner);
            *dex_counter.entry(label).or_default() += 1;

            if !token_mints.is_empty() {
                let key = token_mints
                    .iter()
                    .map(|mint| mint.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                *mint_counter.entry(key).or_default() += 1;
            }
        }

        if !dex_counter.is_empty() {
            let mut summary: Vec<(&str, usize)> = dex_counter
                .iter()
                .map(|(label, count)| (*label, *count))
                .collect();
            summary.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            let formatted = summary
                .iter()
                .map(|(label, count)| format!("{label} - {count}"))
                .collect::<Vec<_>>()
                .join(", ");
            info!(
                target: "pure_blind::market_cache",
                counts = %formatted,
                "纯盲发市场缓存 DEX 分布"
            );
        }

        if !mint_counter.is_empty() {
            let mut mint_summary: Vec<(String, usize)> = mint_counter.into_iter().collect();
            mint_summary.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let formatted = mint_summary
                .iter()
                .map(|(combo, count)| format!("{combo} - {count}"))
                .collect::<Vec<_>>()
                .join(", ");
            info!(
                target: "pure_blind::market_cache",
                combos = %formatted,
                "纯盲发市场缓存 mint 组合分布"
            );
        }
        Ok(filtered)
    }

    async fn write_filtered_cache(&self, records: &[MarketRecord]) -> Result<()> {
        if let Some(parent) = self.settings.local_path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("创建市场缓存目录失败: {}", parent.display()))?;
        }

        let persisted: Vec<PersistedMarketRecord> = records.iter().map(Into::into).collect();
        let serialized = serde_json::to_vec_pretty(&persisted)
            .with_context(|| "序列化过滤后的市场缓存失败".to_string())?;

        fs::write(&self.settings.local_path, serialized)
            .await
            .with_context(|| {
                format!(
                    "写入过滤后的市场缓存失败: {}",
                    self.settings.local_path.display()
                )
            })?;

        Ok(())
    }
}

#[derive(Clone)]
struct MarketCacheSettings {
    local_path: PathBuf,
    download_url: String,
    download_proxy: Option<String>,
    auto_refresh_minutes: u64,
    exclude_other_programs: bool,
    exclude_programs: HashSet<Pubkey>,
    base_mints: HashSet<Pubkey>,
    intermediates: HashSet<Pubkey>,
}

impl MarketCacheSettings {
    fn from_configs(
        cache_cfg: &PureBlindMarketCacheConfig,
        asset_cfg: &PureBlindAssetsConfig,
    ) -> Self {
        let mut exclude_programs = HashSet::new();
        for entry in &cache_cfg.exclude_dex_program_ids {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            match Pubkey::from_str(trimmed) {
                Ok(program) => {
                    exclude_programs.insert(program);
                }
                Err(err) => {
                    warn!(
                        target: "pure_blind::market_cache",
                        program_id = trimmed,
                        error = %err,
                        "exclude_dex_program_ids 中的 program id 解析失败，已忽略"
                    );
                }
            }
        }

        let mut base_mints = HashSet::new();
        for base in &asset_cfg.base_mints {
            let trimmed = base.mint.trim();
            if trimmed.is_empty() {
                continue;
            }
            match Pubkey::from_str(trimmed) {
                Ok(mint) => {
                    base_mints.insert(mint);
                }
                Err(err) => {
                    warn!(
                        target: "pure_blind::market_cache",
                        mint = trimmed,
                        error = %err,
                        "pure_blind_strategy.assets.base_mints 公钥解析失败，已忽略"
                    );
                }
            }
        }

        let mut intermediates = HashSet::new();
        for entry in &asset_cfg.intermediates {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            match Pubkey::from_str(trimmed) {
                Ok(mint) => {
                    intermediates.insert(mint);
                }
                Err(err) => {
                    warn!(
                        target: "pure_blind::market_cache",
                        mint = trimmed,
                        error = %err,
                        "pure_blind_strategy.assets.intermediates 公钥解析失败，已忽略"
                    );
                }
            }
        }

        Self {
            local_path: PathBuf::from(cache_cfg.path.trim()),
            download_url: cache_cfg.download_url.trim().to_string(),
            download_proxy: cache_cfg
                .proxy
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            auto_refresh_minutes: cache_cfg.auto_refresh_minutes,
            exclude_other_programs: cache_cfg.exclude_other_dex_program_ids,
            exclude_programs,
            base_mints,
            intermediates,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct RawMarketRecord {
    pubkey: String,
    owner: String,
    #[serde(default)]
    params: Option<RawMarketParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedMarketRecord {
    market: String,
    owner: String,
    #[serde(default)]
    lookup_table: Option<String>,
    #[serde(default)]
    routing_group: Option<u8>,
    #[serde(default)]
    swap_account_size: Option<PersistedSwapAccountSize>,
    #[serde(default)]
    token_mints: Vec<String>,
    #[serde(default)]
    liquidity_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSwapAccountSize {
    #[serde(default)]
    account_len: u64,
    #[serde(default)]
    account_metas_count: u64,
    #[serde(default)]
    account_compressed_count: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct RawMarketParams {
    #[serde(rename = "addressLookupTableAddress")]
    #[serde(default)]
    address_lookup_table_address: Option<String>,
    #[serde(rename = "routingGroup")]
    #[serde(default)]
    routing_group: Option<u8>,
    #[serde(rename = "swapAccountSize")]
    #[serde(default)]
    swap_account_size: Option<RawSwapAccountSize>,
    #[serde(rename = "tokenMints")]
    #[serde(default)]
    token_mints: Vec<String>,
    #[serde(rename = "liquidityUsd")]
    #[serde(default)]
    liquidity_usd: Option<f64>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct RawSwapAccountSize {
    #[serde(rename = "account_len")]
    #[serde(default)]
    account_len: u64,
    #[serde(rename = "account_metas_count")]
    #[serde(default)]
    account_metas_count: u64,
    #[serde(rename = "account_compressed_count")]
    #[serde(default)]
    account_compressed_count: u64,
}

impl From<RawSwapAccountSize> for SwapAccountSize {
    fn from(value: RawSwapAccountSize) -> Self {
        Self {
            account_len: value.account_len,
            account_metas_count: value.account_metas_count,
            account_compressed_count: value.account_compressed_count,
        }
    }
}

fn supported_program_ids() -> HashSet<Pubkey> {
    HashSet::from([
        SOLFI_V2_PROGRAM_ID,
        HUMIDIFI_PROGRAM_ID,
        TESSERA_V_PROGRAM_ID,
        ZEROFI_PROGRAM_ID,
        OBRIC_V2_PROGRAM_ID,
        RAYDIUM_CLMM_PROGRAM_ID,
        METEORA_DLMM_PROGRAM_ID,
        ORCA_WHIRLPOOL_PROGRAM_ID,
    ])
}

fn dex_label_for_owner(owner: Pubkey) -> &'static str {
    if owner == ZEROFI_PROGRAM_ID {
        "ZeroFi"
    } else if owner == SOLFI_V2_PROGRAM_ID {
        "SolFi V2"
    } else if owner == TESSERA_V_PROGRAM_ID {
        "Tessera V"
    } else if owner == HUMIDIFI_PROGRAM_ID {
        "HumidiFi"
    } else if owner == OBRIC_V2_PROGRAM_ID {
        "Obric V2"
    } else if owner == RAYDIUM_CLMM_PROGRAM_ID {
        "Raydium CLMM"
    } else if owner == METEORA_DLMM_PROGRAM_ID {
        "Meteora DLMM"
    } else if owner == ORCA_WHIRLPOOL_PROGRAM_ID {
        "Whirlpool"
    } else {
        "Unknown"
    }
}

impl From<&MarketRecord> for PersistedMarketRecord {
    fn from(value: &MarketRecord) -> Self {
        Self {
            market: value.market.to_string(),
            owner: value.owner.to_string(),
            lookup_table: value.lookup_table.map(|lt| lt.to_string()),
            routing_group: value.routing_group,
            swap_account_size: value.swap_account_size.as_ref().map(Into::into),
            token_mints: value.token_mints.iter().map(ToString::to_string).collect(),
            liquidity_usd: value.liquidity_usd,
        }
    }
}

impl From<&SwapAccountSize> for PersistedSwapAccountSize {
    fn from(value: &SwapAccountSize) -> Self {
        Self {
            account_len: value.account_len,
            account_metas_count: value.account_metas_count,
            account_compressed_count: value.account_compressed_count,
        }
    }
}

impl TryFrom<PersistedMarketRecord> for MarketRecord {
    type Error = anyhow::Error;

    fn try_from(value: PersistedMarketRecord) -> Result<Self, Self::Error> {
        let market = Pubkey::from_str(&value.market)
            .with_context(|| format!("无法解析 market {}", value.market))?;
        let owner = Pubkey::from_str(&value.owner)
            .with_context(|| format!("无法解析 owner {}", value.owner))?;
        let lookup_table = match value.lookup_table {
            Some(ref lt) if lt.is_empty() => None,
            Some(lt) => {
                Some(Pubkey::from_str(&lt).with_context(|| format!("无法解析 lookup_table {lt}"))?)
            }
            None => None,
        };
        let token_mints = value
            .token_mints
            .into_iter()
            .map(|mint| {
                Pubkey::from_str(&mint).with_context(|| format!("无法解析 token_mint {mint}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let swap_account_size = value.swap_account_size.map(|size| SwapAccountSize {
            account_len: size.account_len,
            account_metas_count: size.account_metas_count,
            account_compressed_count: size.account_compressed_count,
        });

        Ok(Self {
            market,
            owner,
            lookup_table,
            routing_group: value.routing_group,
            swap_account_size,
            token_mints,
            liquidity_usd: value.liquidity_usd,
        })
    }
}

fn create_download_progress_bar(path: &Path, total_size: u64) -> Option<ProgressBar> {
    if !std::io::stderr().is_terminal() {
        return None;
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("market-cache");

    if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        let style = ProgressStyle::with_template(
            "{spinner:.green} {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar());
        pb.set_style(style);
        pb.set_message(format!("下载 {name}"));
        Some(pb)
    } else {
        let pb = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());
        pb.set_style(style);
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message(format!("下载 {name}"));
        Some(pb)
    }
}
