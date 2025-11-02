use std::sync::Arc;

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::kamino::KaminoApiClient;
use crate::api::ultra::UltraApiClient;
use crate::cli::args::{Cli, Command};
use crate::cli::context::{
    init_configs, resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{StrategyMode, run_strategy};
use crate::config::launch::resources::{build_http_client_pool, build_http_client_with_options};
use crate::config::{AppConfig, StrategyToggle};
use crate::engine::ConsoleSummarySink;

enum AggregatorContext {
    Dflow {
        api_client: DflowApiClient,
    },
    Kamino {
        api_client: KaminoApiClient,
        rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    },
    Ultra {
        api_client: UltraApiClient,
        rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    },
    None,
}

pub async fn run(
    cli: Cli,
    config: AppConfig,
    summary_sink: Option<Arc<dyn ConsoleSummarySink>>,
) -> Result<()> {
    if config.galileo.bot.prometheus.enable {
        crate::monitoring::try_init_prometheus(&config.galileo.bot.prometheus.listen)
            .map_err(|err| anyhow!(err))?;
    }

    let blind_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::BlindStrategy);
    let pure_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::PureBlindStrategy);
    let copy_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::CopyStrategy);

    let aggregator = match config.galileo.engine.backend {
        crate::config::EngineBackend::Dflow => {
            let quote_base = config
                .galileo
                .engine
                .dflow
                .api_quote_base
                .clone()
                .ok_or_else(|| anyhow!("未配置 DFlow 报价 API base_url"))?;
            let swap_base = config
                .galileo
                .engine
                .dflow
                .api_swap_base
                .clone()
                .unwrap_or_else(|| quote_base.clone());
            let dflow_proxy = config
                .galileo
                .engine
                .dflow
                .api_proxy
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let global_proxy = resolve_global_http_proxy(&config.galileo.global)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            if let Some(proxy_url) = dflow_proxy.clone() {
                info!(
                    target: "dflow",
                    proxy = %proxy_url,
                    "DFlow API 请求将通过配置的代理发送"
                );
            } else if let Some(proxy_url) = global_proxy.clone() {
                info!(
                    target: "dflow",
                    proxy = %proxy_url,
                    "DFlow API 请求将通过配置的代理发送"
                );
            }
            let api_http_client = build_http_client_with_options(
                dflow_proxy.as_deref(),
                global_proxy.clone(),
                false,
                None,
                None,
            )?;
            let api_client_pool =
                build_http_client_pool(dflow_proxy.clone(), global_proxy.clone(), false, None);
            let api_client = DflowApiClient::with_ip_pool(
                api_http_client,
                quote_base,
                swap_base,
                &config.galileo.engine.time_out,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::Dflow { api_client }
        }
        crate::config::EngineBackend::Kamino => {
            let kamino_cfg = &config.galileo.engine.kamino;
            let quote_base = kamino_cfg
                .api_quote_base
                .as_ref()
                .ok_or_else(|| anyhow!("kamino.api_quote_base 未配置"))?
                .trim()
                .to_string();
            if quote_base.is_empty() {
                return Err(anyhow!("kamino.api_quote_base 不能为空"));
            }
            let swap_base = kamino_cfg
                .api_swap_base
                .clone()
                .unwrap_or_else(|| quote_base.clone());
            let kamino_proxy = kamino_cfg
                .api_proxy
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let global_proxy = resolve_global_http_proxy(&config.galileo.global)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            if let Some(proxy_url) = kamino_proxy.clone() {
                info!(
                    target: "kamino",
                    proxy = %proxy_url,
                    "Kamino API 请求将通过配置的代理发送"
                );
            } else if let Some(proxy_url) = global_proxy.clone() {
                info!(
                    target: "kamino",
                    proxy = %proxy_url,
                    "Kamino API 请求将通过配置的代理发送"
                );
            }
            let api_http_client = build_http_client_with_options(
                kamino_proxy.as_deref(),
                global_proxy.clone(),
                false,
                None,
                None,
            )?;
            let api_client_pool =
                build_http_client_pool(kamino_proxy.clone(), global_proxy.clone(), false, None);
            let api_client = KaminoApiClient::with_ip_pool(
                api_http_client,
                quote_base,
                swap_base,
                &config.galileo.engine.time_out,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            let resolved_rpc = resolve_rpc_client(&config.galileo.global, None)?;
            let rpc_client = resolved_rpc.client.clone();
            AggregatorContext::Kamino {
                api_client,
                rpc_client,
            }
        }
        crate::config::EngineBackend::Ultra => {
            let ultra_cfg = &config.galileo.engine.ultra;
            let api_base = ultra_cfg
                .api_quote_base
                .as_ref()
                .ok_or_else(|| anyhow!("ultra.api_quote_base 未配置"))?
                .trim()
                .to_string();
            if api_base.is_empty() {
                return Err(anyhow!("ultra.api_quote_base 不能为空"));
            }
            let resolved_rpc = resolve_rpc_client(&config.galileo.global, None)?;
            let rpc_client = resolved_rpc.client.clone();
            let ultra_proxy = ultra_cfg
                .api_proxy
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let global_proxy = resolve_global_http_proxy(&config.galileo.global)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            if let Some(proxy_url) = ultra_proxy.clone() {
                info!(
                    target: "ultra",
                    proxy = %proxy_url,
                    "Ultra API 请求将通过配置的代理发送"
                );
            } else if let Some(proxy_url) = global_proxy.clone() {
                info!(
                    target: "ultra",
                    proxy = %proxy_url,
                    "Ultra API 请求将通过配置的代理发送"
                );
            }
            let http_client = build_http_client_with_options(
                ultra_proxy.as_deref(),
                global_proxy.clone(),
                false,
                None,
                None,
            )?;
            let http_pool =
                build_http_client_pool(ultra_proxy.clone(), global_proxy.clone(), false, None);
            let api_client = UltraApiClient::with_ip_pool(
                http_client,
                api_base,
                &config.galileo.engine.time_out,
                &config.galileo.global.logging,
                Some(http_pool),
            );
            AggregatorContext::Ultra {
                api_client,
                rpc_client,
            }
        }
        crate::config::EngineBackend::MultiLegs => AggregatorContext::None,
        crate::config::EngineBackend::None => {
            if blind_enabled {
                return Err(anyhow!(
                    "engine.backend=none 仅支持纯盲发或 copy 策略，请从 bot.strategies.enabled 中移除 blind_strategy"
                ));
            }
            if !pure_enabled && !copy_enabled {
                warn!(
                    target: "runner",
                    "engine.backend=none 生效，但 pure_blind_strategy 与 copy_strategy 均未启用"
                );
            }
            AggregatorContext::None
        }
    };

    dispatch(cli.command, config, aggregator, summary_sink).await
}

async fn dispatch(
    command: Command,
    config: AppConfig,
    aggregator: AggregatorContext,
    summary_sink: Option<Arc<dyn ConsoleSummarySink>>,
) -> Result<()> {
    // 统一的命令分发入口，便于后续按子命令拆分到专门模块。
    match command {
        Command::Lander(cmd) => {
            handle_lander_cmd(
                cmd,
                &config,
                &config.lander.lander,
                resolve_instruction_memo(&config.galileo.global.instruction),
            )
            .await?;
        }
        Command::Run => match &aggregator {
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(&config, &backend, StrategyMode::Live, summary_sink.clone()).await?;
            }
            AggregatorContext::Kamino {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::Live, summary_sink.clone()).await?;
            }
            AggregatorContext::Ultra {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Ultra {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::Live, summary_sink.clone()).await?;
            }
            AggregatorContext::None => {
                let backend = crate::cli::strategy::StrategyBackend::None;
                run_strategy(&config, &backend, StrategyMode::Live, summary_sink.clone()).await?;
            }
        },
        Command::StrategyDryRun => match &aggregator {
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(
                    &config,
                    &backend,
                    StrategyMode::DryRun,
                    summary_sink.clone(),
                )
                .await?;
            }
            AggregatorContext::Kamino {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(
                    &config,
                    &backend,
                    StrategyMode::DryRun,
                    summary_sink.clone(),
                )
                .await?;
            }
            AggregatorContext::Ultra {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Ultra {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(
                    &config,
                    &backend,
                    StrategyMode::DryRun,
                    summary_sink.clone(),
                )
                .await?;
            }
            AggregatorContext::None => {
                let backend = crate::cli::strategy::StrategyBackend::None;
                run_strategy(
                    &config,
                    &backend,
                    StrategyMode::DryRun,
                    summary_sink.clone(),
                )
                .await?;
            }
        },
        Command::Init(args) => {
            init_configs(args)?;
        }
        Command::Wallet(_) => {}
    }

    Ok(())
}
