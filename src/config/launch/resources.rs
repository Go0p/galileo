use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use reqwest::Proxy;
use tracing::info;

use crate::cli::context::{RpcEndpointRotator, resolve_global_http_proxy, resolve_rpc_client};
use crate::config::{self, AppConfig};
use crate::network::{
    CooldownConfig, IpAllocator, IpBoundClientPool, IpInventory, IpInventoryConfig, NetworkError,
    ReqwestClientFactoryFn, RpcClientFactoryFn,
};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::RpcClientConfig;
use solana_rpc_client::http_sender::HttpSender;

/// 解析 RPC 设置并构造 rotator + client
#[allow(dead_code)]
pub fn build_rpc_resources(
    config: &AppConfig,
) -> Result<(Arc<RpcClient>, Arc<RpcEndpointRotator>)> {
    let resolved = resolve_rpc_client(&config.galileo.global)?;
    Ok((resolved.client.clone(), resolved.endpoints))
}

/// 构造 IP allocator
pub fn build_ip_allocator(cfg: &config::NetworkConfig) -> Result<Arc<IpAllocator>> {
    let inventory_cfg = IpInventoryConfig {
        enable_multiple_ip: cfg.enable_multiple_ip,
        manual_ips: cfg.manual_ips.clone(),
        blacklist: cfg.blacklist_ips.clone(),
        allow_loopback: cfg.allow_loopback,
    };

    let inventory =
        IpInventory::new(inventory_cfg).map_err(|err| anyhow!("初始化本地 IP 资源失败: {err}"))?;

    let cooldown = CooldownConfig {
        rate_limited_start: Duration::from_millis(cfg.cooldown_ms.rate_limited_start.max(1)),
        timeout_start: Duration::from_millis(cfg.cooldown_ms.timeout_start.max(1)),
    };

    let allocator = IpAllocator::from_inventory(
        inventory,
        cfg.per_ip_inflight_limit.map(|limit| limit as usize),
        cooldown,
    );

    let summary = allocator.summary();
    let ips: Vec<String> = allocator
        .slot_ips()
        .into_iter()
        .map(|ip| ip.to_string())
        .collect();

    info!(
        target: "network::allocator",
        total_slots = summary.total_slots,
        per_ip_limit = ?summary.per_ip_inflight_limit,
        source = ?summary.source,
        ips = ?ips,
        "IP 资源池已初始化"
    );

    Ok(Arc::new(allocator))
}

/// 构造 HTTP client pool
pub fn build_http_client_with_options(
    proxy: Option<&str>,
    global_proxy: Option<String>,
    bypass_proxy: bool,
    local_ip: Option<IpAddr>,
    user_agent: Option<&str>,
) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();

    if let Some(agent) = user_agent {
        builder = builder.user_agent(agent);
    }

    if let Some(ip) = local_ip {
        builder = builder.local_address(ip);
    }

    if bypass_proxy {
        builder = builder.no_proxy();
    } else if let Some(proxy_url) = proxy.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }) {
        let proxy = Proxy::all(&proxy_url)
            .map_err(|err| anyhow!("HTTP 代理地址无效 {proxy_url}: {err}"))?;
        builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
    } else if let Some(proxy_url) = global_proxy {
        let trimmed = proxy_url.trim().to_string();
        if !trimmed.is_empty() {
            let proxy = Proxy::all(&trimmed)
                .map_err(|err| anyhow!("global.proxy 地址无效 {trimmed}: {err}"))?;
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }
    }

    builder
        .build()
        .map_err(|err| anyhow!("构建 HTTP 客户端失败: {err}"))
}

pub fn build_http_client_pool(
    proxy: Option<String>,
    global_proxy: Option<String>,
    bypass_proxy: bool,
    user_agent: Option<String>,
) -> Arc<IpBoundClientPool<ReqwestClientFactoryFn>> {
    let proxy_clone = proxy.clone();
    let global_clone = global_proxy.clone();
    let agent_clone = user_agent.clone();
    let factory: ReqwestClientFactoryFn = Box::new(move |ip: IpAddr| {
        build_http_client_with_options(
            proxy_clone.as_deref(),
            global_clone.clone(),
            bypass_proxy,
            Some(ip),
            agent_clone.as_deref(),
        )
        .map_err(|err| NetworkError::ClientPool(err.to_string()))
    });
    Arc::new(IpBoundClientPool::new(factory))
}

/// 构造 RPC client pool
pub fn build_rpc_client_pool(
    endpoints: Arc<RpcEndpointRotator>,
    global_proxy: Option<String>,
) -> Arc<IpBoundClientPool<RpcClientFactoryFn>> {
    let cache: Arc<DashMap<(IpAddr, usize), Arc<RpcClient>>> = Arc::new(DashMap::new());
    let proxy = Arc::new(global_proxy);
    let rotator = endpoints;

    let factory: RpcClientFactoryFn = Box::new(move |ip: IpAddr| {
        let (index, endpoint) = rotator.next();
        let key = (ip, index);
        if let Some(existing) = cache.get(&key) {
            return Ok(existing.clone());
        }

        let default_headers = crate::cli::context::rpc_default_headers(endpoint);
        let mut builder = reqwest::Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(30))
            .local_address(ip);

        if let Some(proxy_url) = proxy.as_ref() {
            let trimmed = proxy_url.trim();
            if trimmed.is_empty() {
                builder = builder.no_proxy();
            } else {
                let proxy = Proxy::all(trimmed).map_err(|err| {
                    NetworkError::ClientPool(format!("global.proxy 地址无效 {trimmed}: {err}"))
                })?;
                builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
            }
        } else {
            builder = builder.no_proxy();
        }

        let client = builder.build().map_err(|err| {
            NetworkError::ClientPool(format!("构建绑定 IP 的 RPC 客户端失败: {err}"))
        })?;

        let rpc_url = endpoint.to_string();
        let sender = HttpSender::new_with_client(rpc_url.clone(), client);
        let rpc = Arc::new(RpcClient::new_sender(
            sender,
            RpcClientConfig::with_commitment(solana_commitment_config::CommitmentConfig::default()),
        ));

        if let Some(previous) = cache.insert(key, rpc.clone()) {
            return Ok(previous);
        }

        Ok(rpc)
    });

    Arc::new(IpBoundClientPool::new(factory))
}

/// 构造落地提交客户端
#[allow(dead_code)]
pub fn build_submission_client(config: &AppConfig) -> Result<reqwest::Client> {
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let mut builder = reqwest::Client::builder();
    if let Some(proxy_url) = global_proxy.as_ref() {
        let proxy = Proxy::all(proxy_url.as_str())
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
    }
    builder.build().map_err(|err| anyhow!(err))
}
