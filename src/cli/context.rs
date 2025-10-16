use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use time::{UtcOffset, macros::format_description};
use tracing::error;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{EnvFilter, fmt};
use url::Url;

use crate::config::{
    AppConfig, BotConfig, ConfigError, GlobalConfig, IntermediumConfig, JupiterConfig,
    LaunchOverrides, LoggingProfile, RequestParamsConfig, TitanEngineConfig, YellowstoneConfig,
    load_config,
};
use crate::jupiter::{BinaryStatus, JupiterBinaryManager, JupiterError};

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

/// 确保本地 Jupiter 二进制已运行，否则提示用户先执行 `galileo jupiter start`。
pub async fn ensure_running(manager: &JupiterBinaryManager) -> Result<(), JupiterError> {
    if manager.disable_local_binary {
        return Ok(());
    }
    match manager.status().await {
        BinaryStatus::Running => Ok(()),
        status => {
            error!(
                target: "jupiter",
                ?status,
                "Jupiter 二进制未运行，请先执行 `galileo jupiter start`"
            );
            Err(JupiterError::Schema(format!(
                "二进制未运行，当前状态: {status:?}"
            )))
        }
    }
}

pub fn resolve_jupiter_base_url(_bot: &BotConfig, jupiter: &JupiterConfig) -> String {
    if let Ok(url) = std::env::var("JUPITER_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let host = sanitize_jupiter_host(&jupiter.core.host);
    format!("http://{}:{}", host, jupiter.core.port)
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

pub fn resolve_titan_ws_endpoint(titan: &TitanEngineConfig) -> Result<Option<Url>> {
    if !titan.enable {
        return Ok(None);
    }

    let jwt = titan
        .jwt
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("TITAN_JWT")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });

    let Some(jwt) = jwt else {
        return Ok(None);
    };

    let base = titan
        .ws_url
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("TITAN_WS_URL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "wss://api.titan.exchange/api/v1/ws".to_string());

    let mut url =
        Url::parse(&base).map_err(|err| anyhow!("Titan WebSocket 地址无效 {base}: {err}"))?;

    let mut params: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    params.retain(|(key, _)| key != "auth");
    params.push(("auth".to_string(), jwt));

    {
        let mut serializer = url.query_pairs_mut();
        serializer.clear();
        for (key, value) in params {
            serializer.append_pair(&key, &value);
        }
    }

    Ok(Some(url))
}

pub fn build_launch_overrides(
    params: &RequestParamsConfig,
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

    overrides.exclude_dex_program_ids = params.excluded_dexes.clone();
    overrides.include_dex_program_ids = params.included_dexes.clone();

    overrides
}

pub fn resolve_jupiter_defaults(
    mut jupiter: JupiterConfig,
    global: &GlobalConfig,
) -> Result<JupiterConfig> {
    if jupiter.core.rpc_url.trim().is_empty() {
        if let Some(global_rpc) = &global.rpc_url {
            let trimmed = global_rpc.trim();
            if !trimmed.is_empty() {
                jupiter.core.rpc_url = trimmed.to_string();
            }
        }
    }

    if jupiter.core.rpc_url.trim().is_empty() {
        return Err(anyhow!(
            "未配置 Jupiter RPC：请在 jupiter.toml 或 galileo.yaml 的 global.rpc_url 中设置 rpc_url"
        ));
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

pub fn resolve_rpc_client(
    global: &GlobalConfig,
) -> Result<Arc<solana_client::nonblocking::rpc_client::RpcClient>> {
    let url = env::var("GALILEO_RPC_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            global
                .rpc_url
                .clone()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

    Ok(Arc::new(
        solana_client::nonblocking::rpc_client::RpcClient::new(url),
    ))
}
