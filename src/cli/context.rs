use std::collections::BTreeSet;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::RpcClientConfig;
use solana_rpc_client::http_sender::HttpSender;
use time::{UtcOffset, macros::format_description};
use tracing::info;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::{
    AppConfig, ConfigError, DryRunConfig, GlobalConfig, IntermediumConfig, JupiterConfig,
    JupiterSelfHostedEngineConfig, LaunchOverrides, LoggingProfile, ProxyProfile, load_config,
};

#[derive(Debug)]
pub struct RpcEndpointRotator {
    endpoints: Arc<Vec<String>>,
    cursor: AtomicUsize,
}

impl RpcEndpointRotator {
    pub fn new(endpoints: Vec<String>) -> Result<Self> {
        if endpoints.is_empty() {
            return Err(anyhow!("global.rpc_urls 至少需要配置一个 RPC 端点"));
        }
        Ok(Self {
            cursor: AtomicUsize::new(0),
            endpoints: Arc::new(endpoints),
        })
    }

    pub fn primary_url(&self) -> &str {
        &self.endpoints[0]
    }

    pub fn next(&self) -> (usize, &str) {
        let len = self.endpoints.len();
        let index = if len == 1 {
            0
        } else {
            self.cursor.fetch_add(1, Ordering::Relaxed) % len
        };
        (index, self.endpoints[index].as_str())
    }

    pub fn endpoints(&self) -> &[String] {
        self.endpoints.as_slice()
    }
}

#[derive(Clone)]
pub struct ResolvedRpcClient {
    pub client: Arc<RpcClient>,
    pub endpoints: Arc<RpcEndpointRotator>,
}

#[derive(Clone, Debug, Default)]
pub struct DryRunMode {
    pub enabled: bool,
    rpc_url: Option<String>,
}

impl DryRunMode {
    pub fn from_sources(cli_enabled: bool, config: &DryRunConfig) -> Result<Self> {
        let url = config
            .rpc_url
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let enabled = cli_enabled || config.enable;
        if enabled && url.is_none() {
            return Err(anyhow!(
                "dry-run 模式启用时需提供 bot.dry_run.rpc_url，用于指向本地测试节点"
            ));
        }
        Ok(Self {
            enabled,
            rpc_url: url,
        })
    }

    pub fn rpc_override(&self) -> Option<&str> {
        if self.enabled {
            self.rpc_url.as_deref()
        } else {
            None
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 初始化 tracing，兼顾 JSON 与文本输出模式。
pub fn init_tracing(config: &crate::config::LoggingConfig) -> Result<()> {
    let mut filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    // 默认压低外部依赖的调试输出，避免日志被噪声淹没；Verbose 模式可通过配置显式覆盖。
    if matches!(config.profile, LoggingProfile::Lean) {
        const QUIET_TARGETS: &[(&str, &str)] = &[
            ("hyper", "warn"),
            ("hyper_util::client::legacy", "warn"),
            ("reqwest", "info"),
            ("tokio_util::codec", "info"),
        ];
        for (module, level) in QUIET_TARGETS {
            if !config.level.contains(module) {
                if let Ok(directive) = format!("{module}={level}").parse() {
                    filter = filter.add_directive(directive);
                }
            }
        }
    }

    if matches!(config.profile, LoggingProfile::Verbose) {
        const VERBOSE_TARGETS: &[(&str, &str)] = &[
            ("jupiter::quote", "debug"),
            ("jupiter::swap_instructions", "debug"),
            ("monitoring::quote", "info"),
            ("monitoring::swap", "info"),
        ];
        for (module, level) in VERBOSE_TARGETS {
            if let Ok(directive) = format!("{module}={level}").parse() {
                filter = filter.add_directive(directive);
            }
        }
    }

    let time_format =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
    let offset = UtcOffset::from_hms(config.timezone_offset_hours, 0, 0).map_err(|err| {
        anyhow!(
            "invalid logging timezone offset {}: {err}",
            config.timezone_offset_hours
        )
    })?;
    let offset_timer = OffsetTime::new(offset, time_format);

    let builder = fmt()
        .with_timer(offset_timer.clone())
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(true)
        .with_level(true);

    if config.json {
        builder
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .with_env_filter(filter)
            .try_init()
            .map_err(|err| anyhow!(err.to_string()))?;
    } else {
        builder
            .with_env_filter(filter)
            .event_format(fmt::format().compact())
            .try_init()
            .map_err(|err| anyhow!(err.to_string()))?;
    }
    Ok(())
}

/// 加载主配置；用于 `galileo --config` 的入口。
pub fn load_configuration(path: Option<PathBuf>) -> Result<AppConfig, ConfigError> {
    load_config(path)
}

#[derive(Debug, Clone)]
pub struct ProxySelection {
    pub url: String,
    pub per_request: bool,
}

impl ProxySelection {
    pub fn from_profile(profile: &ProxyProfile) -> Self {
        Self {
            url: profile.url.clone(),
            per_request: profile.per_request,
        }
    }
}

pub fn resolve_global_http_proxy(global: &GlobalConfig) -> Option<ProxySelection> {
    global.proxy.default_url().map(|url| ProxySelection {
        url: url.to_string(),
        per_request: false,
    })
}

pub fn resolve_proxy_profile(global: &GlobalConfig, target: &str) -> Option<ProxySelection> {
    global
        .proxy
        .resolve_for(target)
        .map(ProxySelection::from_profile)
}

pub fn override_proxy_selection(
    override_url: Option<&str>,
    module_proxy: Option<ProxySelection>,
    global_proxy: Option<ProxySelection>,
) -> Option<ProxySelection> {
    if let Some(url) = override_url {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return None;
        }
        return Some(ProxySelection {
            url: trimmed.to_string(),
            per_request: false,
        });
    }

    module_proxy.or(global_proxy)
}

pub fn rpc_default_headers(endpoint: &str) -> reqwest::header::HeaderMap {
    let mut headers = HttpSender::default_headers();

    if let Ok(url) = reqwest::Url::parse(endpoint) {
        if let Some(host) = url.host_str() {
            if host.eq_ignore_ascii_case("pump-fe.helius-rpc.com") {
                headers.insert(
                    reqwest::header::ORIGIN,
                    reqwest::header::HeaderValue::from_static("https://swap.pump.fun"),
                );
            }
        }
    }

    headers
}

pub fn should_bypass_proxy(base_url: &str) -> bool {
    if let Ok(url) = reqwest::Url::parse(base_url) {
        if let Some(host) = url.host_str() {
            if host.eq_ignore_ascii_case("localhost") {
                return true;
            }
            if let Ok(ip) = host.parse::<IpAddr>() {
                return ip.is_loopback() || ip.is_unspecified();
            }
        }
    }
    false
}

pub fn build_launch_overrides(
    intermedium: &IntermediumConfig,
    jupiter_self_hosted: &JupiterSelfHostedEngineConfig,
) -> LaunchOverrides {
    let mut overrides = LaunchOverrides::default();

    let mut mint_set: BTreeSet<String> = intermedium.mints.iter().cloned().collect();
    for mint in &intermedium.disable_mints {
        mint_set.remove(mint);
    }

    if !mint_set.is_empty() {
        overrides.filter_markets_with_mints = mint_set.into_iter().collect();
    }

    let include_set: BTreeSet<String> = jupiter_self_hosted
        .args_included_dexes
        .iter()
        .map(|dex| dex.trim().to_string())
        .filter(|dex| !dex.is_empty())
        .collect();
    if !include_set.is_empty() {
        overrides.include_dex_program_ids = include_set.into_iter().collect();
    }

    overrides
}

pub fn resolve_jupiter_base_url(jupiter: &JupiterConfig) -> String {
    let host = sanitize_jupiter_host(&jupiter.core.host);
    format!("http://{}:{}", host, jupiter.core.port)
}

pub fn resolve_self_hosted_jupiter_api_proxy(
    config: &JupiterSelfHostedEngineConfig,
) -> Option<String> {
    config
        .api_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn sanitize_jupiter_host(host: &str) -> String {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return "127.0.0.1".to_string();
    }
    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        if ip.is_unspecified() {
            return match ip {
                IpAddr::V4(_) => "127.0.0.1".to_string(),
                IpAddr::V6(_) => "::1".to_string(),
            };
        }
    }
    trimmed.to_string()
}

pub fn resolve_jupiter_defaults(
    mut jupiter: JupiterConfig,
    global: &GlobalConfig,
) -> Result<JupiterConfig> {
    if jupiter.core.rpc_url.trim().is_empty() {
        if let Some(primary) = global.rpc_urls().first() {
            jupiter.core.rpc_url = primary.to_string();
        }
    }

    if jupiter.core.rpc_url.trim().is_empty() {
        return Err(anyhow!(
            "未配置 Jupiter RPC：请在 jupiter.toml 或 galileo.yaml 的 global.rpc_urls 中设置 RPC 端点"
        ));
    }

    let needs_yellowstone = jupiter
        .launch
        .yellowstone
        .as_ref()
        .map(|cfg| cfg.endpoint.trim().is_empty())
        .unwrap_or(true);

    if needs_yellowstone {
        if let Some(endpoint) = global.yellowstone_grpc_url.as_ref() {
            let trimmed = endpoint.trim();
            if !trimmed.is_empty() {
                let token = global.yellowstone_grpc_token.as_ref().and_then(|t| {
                    let tt = t.trim();
                    if tt.is_empty() {
                        None
                    } else {
                        Some(tt.to_string())
                    }
                });
                jupiter.launch.yellowstone = Some(crate::config::YellowstoneConfig {
                    endpoint: trimmed.to_string(),
                    x_token: token,
                });
            }
        }
    }

    Ok(jupiter)
}

pub fn init_configs(args: crate::cli::args::InitCmd) -> Result<()> {
    let output_dir = match args.output {
        Some(dir) => dir,
        None => std::env::current_dir()?,
    };

    fs::create_dir_all(&output_dir)?;

    let templates: [(&str, &str); 3] = [
        (
            "galileo.yaml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/galileo.yaml")),
        ),
        (
            "lander.yaml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lander.yaml")),
        ),
        (
            "jupiter.toml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/jupiter.toml")),
        ),
    ];

    for (filename, contents) in templates {
        let target_path = output_dir.join(filename);
        if target_path.exists() && !args.force {
            println!(
                "跳过 {}（文件已存在，如需覆盖请加 --force）",
                target_path.display()
            );
            continue;
        }

        fs::write(&target_path, contents)?;
        println!("已写入 {}", target_path.display());
    }

    Ok(())
}

pub fn resolve_instruction_memo(instruction: &crate::config::InstructionConfig) -> Option<String> {
    let memo = instruction.memo.trim();
    if memo.is_empty() {
        None
    } else {
        Some(memo.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_default_headers_adds_origin_for_pump_fun() {
        let headers =
            rpc_default_headers("https://pump-fe.helius-rpc.com/?api-key=test-key-placeholder");
        let origin = headers
            .get(reqwest::header::ORIGIN)
            .and_then(|value| value.to_str().ok());
        assert_eq!(origin, Some("https://swap.pump.fun"));
    }

    #[test]
    fn test_rpc_default_headers_no_origin_on_other_hosts() {
        let headers = rpc_default_headers("https://api.mainnet-beta.solana.com");
        assert!(headers.get(reqwest::header::ORIGIN).is_none());
    }

    #[test]
    fn test_resolve_self_hosted_proxy_none_on_empty() {
        let cfg: JupiterSelfHostedEngineConfig = serde_yaml::from_str(
            r#"
api_proxy: "   "
"#,
        )
        .unwrap();
        assert!(resolve_self_hosted_jupiter_api_proxy(&cfg).is_none());
    }

    #[test]
    fn test_resolve_self_hosted_proxy_trims_value() {
        let cfg: JupiterSelfHostedEngineConfig = serde_yaml::from_str(
            r#"
api_proxy: "  http://127.0.0.1:8888  "
"#,
        )
        .unwrap();
        assert_eq!(
            resolve_self_hosted_jupiter_api_proxy(&cfg).as_deref(),
            Some("http://127.0.0.1:8888")
        );
    }
}

pub fn resolve_rpc_client(
    global: &GlobalConfig,
    override_endpoint: Option<&str>,
) -> Result<ResolvedRpcClient> {
    let endpoints = if let Some(url) = override_endpoint {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("dry-run rpc_url 不能为空"));
        }
        vec![trimmed.to_string()]
    } else if !global.rpc_urls().is_empty() {
        global.rpc_urls().to_vec()
    } else {
        Vec::new()
    };
    let rotator = Arc::new(RpcEndpointRotator::new(endpoints)?);
    let primary_url = rotator.primary_url().to_string();

    let proxy = if override_endpoint.is_some() {
        None
    } else {
        resolve_global_http_proxy(global)
    };
    let mut builder = reqwest::Client::builder()
        .default_headers(rpc_default_headers(&primary_url))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(30));

    if let Some(selection) = proxy.as_ref() {
        if selection.per_request {
            builder = builder
                .pool_max_idle_per_host(0)
                .pool_idle_timeout(Some(Duration::from_secs(0)));
        }
        let trimmed = selection.url.trim();
        if trimmed.is_empty() {
            builder = builder.no_proxy();
        } else {
            let proxy = reqwest::Proxy::all(trimmed)
                .map_err(|err| anyhow!("global.proxy 地址无效 {trimmed}: {err}"))?;
            info!(
                target: "rpc::client",
                proxy = %selection.url,
                url = %primary_url,
                "RPC 客户端将通过 global.proxy 访问"
            );
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }
    } else {
        builder = builder.no_proxy();
    }

    let client = builder
        .build()
        .map_err(|err| anyhow!("构建 RPC 客户端失败: {err}"))?;
    let sender = HttpSender::new_with_client(primary_url.clone(), client);
    let client = RpcClient::new_sender(
        sender,
        RpcClientConfig::with_commitment(solana_commitment_config::CommitmentConfig::default()),
    );

    if override_endpoint.is_some() {
        info!(
            target = "rpc::client",
            url = %primary_url,
            "dry-run 模式：所有 RPC 请求将指向此端点"
        );
    } else if rotator.endpoints().len() > 1 {
        info!(
            target: "rpc::client",
            urls = ?rotator.endpoints(),
            "已注册多个 RPC 端点"
        );
    }

    Ok(ResolvedRpcClient {
        client: Arc::new(client),
        endpoints: rotator,
    })
}
