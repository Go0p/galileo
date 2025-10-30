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
    AppConfig, BotConfig, ConfigError, EngineConfig, GlobalConfig, IntermediumConfig,
    JupiterConfig, JupiterEngineConfig, LaunchOverrides, LoggingProfile, YellowstoneConfig,
    load_config,
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

    let base = fmt()
        .with_timer(offset_timer.clone())
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(true)
        .with_level(true);

    if config.json {
        base.json()
            .with_current_span(false)
            .with_span_list(false)
            .with_env_filter(filter)
            .try_init()
            .map_err(|err| anyhow!(err.to_string()))?;
    } else {
        base.with_env_filter(filter)
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

pub fn resolve_jupiter_base_url(_bot: &BotConfig, jupiter: &JupiterConfig) -> String {
    let host = sanitize_jupiter_host(&jupiter.core.host);
    format!("http://{}:{}", host, jupiter.core.port)
}

pub fn resolve_jupiter_api_proxy(engine: &EngineConfig) -> Option<String> {
    engine
        .jupiter
        .api_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

pub fn resolve_global_http_proxy(global: &GlobalConfig) -> Option<String> {
    global
        .proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
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
    engine: &JupiterEngineConfig,
    intermedium: &IntermediumConfig,
) -> LaunchOverrides {
    let mut overrides = LaunchOverrides::default();

    let mut mint_set: BTreeSet<String> = intermedium.mints.iter().cloned().collect();
    for mint in &intermedium.disable_mints {
        mint_set.remove(mint);
    }

    if !mint_set.is_empty() {
        overrides.filter_markets_with_mints = mint_set.into_iter().collect();
    }

    overrides.exclude_dex_program_ids = Vec::new();
    overrides.include_dex_program_ids = engine.args_included_dexes.clone();

    overrides
}

pub fn resolve_jupiter_defaults(
    mut jupiter: JupiterConfig,
    global: &GlobalConfig,
) -> Result<JupiterConfig> {
    if jupiter.core.rpc_url.trim().is_empty() {
        if let Some(primary) = global.primary_rpc_url() {
            jupiter.core.rpc_url = primary.to_string();
        }
    }

    if jupiter.core.rpc_url.trim().is_empty() {
        return Err(anyhow!(
            "未配置 Jupiter RPC：请在 jupiter.toml 或 galileo.yaml 的 global.rpc_urls 中设置 RPC 端点"
        ));
    }

    if jupiter.core.secondary_rpc_urls.is_empty() {
        for url in global.rpc_urls() {
            if url != &jupiter.core.rpc_url {
                jupiter.core.secondary_rpc_urls.push(url.clone());
            }
        }
    }

    let needs_yellowstone = jupiter
        .launch
        .yellowstone
        .as_ref()
        .map(|cfg| cfg.endpoint.trim().is_empty())
        .unwrap_or(true);

    if needs_yellowstone {
        if let Some(endpoint) = &global.yellowstone_grpc_url {
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
                jupiter.launch.yellowstone = Some(YellowstoneConfig {
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
    fn test_resolve_jupiter_api_proxy_none_on_empty() {
        let engine = EngineConfig {
            jupiter: JupiterEngineConfig {
                enable: true,
                api_proxy: Some("   ".to_string()),
                ..JupiterEngineConfig::default()
            },
            ..EngineConfig::default()
        };
        assert!(resolve_jupiter_api_proxy(&engine).is_none());
    }

    #[test]
    fn test_resolve_jupiter_api_proxy_trims_value() {
        let engine = EngineConfig {
            jupiter: JupiterEngineConfig {
                enable: false,
                api_proxy: Some("  http://127.0.0.1:8888  ".to_string()),
                ..JupiterEngineConfig::default()
            },
            ..EngineConfig::default()
        };
        assert_eq!(
            resolve_jupiter_api_proxy(&engine).as_deref(),
            Some("http://127.0.0.1:8888")
        );
    }

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
}

pub fn resolve_rpc_client(global: &GlobalConfig) -> Result<ResolvedRpcClient> {
    let endpoints = if !global.rpc_urls().is_empty() {
        global.rpc_urls().to_vec()
    } else {
        Vec::new()
    };
    let rotator = Arc::new(RpcEndpointRotator::new(endpoints)?);
    let primary_url = rotator.primary_url().to_string();

    let proxy = resolve_global_http_proxy(global);
    let mut builder = reqwest::Client::builder()
        .default_headers(rpc_default_headers(&primary_url))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(30));

    if let Some(proxy_url) = proxy.as_ref() {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        info!(
            target: "rpc::client",
            proxy = %proxy_url,
            url = %primary_url,
            "RPC 客户端将通过 global.proxy 访问"
        );
        builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
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

    if rotator.endpoints().len() > 1 {
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
