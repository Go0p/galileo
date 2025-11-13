use std::sync::Arc;

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::JupiterApiClient;
use crate::api::kamino::KaminoApiClient;
use crate::api::ultra::UltraApiClient;
use crate::cli::args::{Cli, Command};
use crate::cli::context::{
    build_launch_overrides, init_configs, override_proxy_selection, resolve_global_http_proxy,
    resolve_instruction_memo, resolve_jupiter_base_url, resolve_jupiter_defaults,
    resolve_proxy_profile, resolve_rpc_client, resolve_self_hosted_jupiter_api_proxy,
    should_bypass_proxy,
};
use crate::cli::jupiter::handle_jupiter_cmd;
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{StrategyMode, run_strategy};
use crate::config::launch::resources::{build_http_client_pool, build_http_client_with_options};
use crate::config::{AppConfig, StrategyToggle};
use crate::jupiter::JupiterBinaryManager;
use crate::jupiter::error::JupiterError;

enum AggregatorContext {
    Jupiter {
        api_client: JupiterApiClient,
    },
    JupiterSelfHosted {
        manager: JupiterBinaryManager,
        api_client: JupiterApiClient,
    },
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

pub async fn run(cli: Cli, config: AppConfig) -> Result<()> {
    if config.galileo.bot.prometheus.enable {
        crate::monitoring::try_init_prometheus(&config.galileo.bot.prometheus.listen)
            .map_err(|err| anyhow!(err))?;
    }

    if let Command::Jupiter(cmd) = &cli.command {
        let jupiter_cfg = resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
        let launch_overrides = build_launch_overrides(
            &config.galileo.intermedium,
            &config.galileo.engine.jupiter_self_hosted,
        );
        let manager = JupiterBinaryManager::new(
            jupiter_cfg,
            launch_overrides,
            config.galileo.bot.binary.disable_local_binary,
            config.galileo.bot.binary.show_logs,
            true,
        )?;
        handle_jupiter_cmd(cmd.clone(), &manager).await?;
        return Ok(());
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

    let mut managed_self_hosted = maybe_start_managed_self_hosted(&cli.command, &config).await?;

    let aggregator = match config.galileo.engine.backend {
        crate::config::EngineBackend::Jupiter => {
            let jupiter_cfg = config
                .galileo
                .engine
                .jupiter
                .primary()
                .ok_or_else(|| anyhow!("缺少 Jupiter 引擎配置"))?;
            let quote_base = jupiter_cfg
                .api_quote_base
                .as_ref()
                .ok_or_else(|| anyhow!("jupiter.api_quote_base 未配置"))?
                .trim()
                .to_string();
            if quote_base.is_empty() {
                return Err(anyhow!("jupiter.api_quote_base 不能为空"));
            }
            let swap_base = jupiter_cfg
                .api_swap_base
                .clone()
                .unwrap_or_else(|| quote_base.clone());
            let proxy_override = jupiter_cfg
                .api_proxy
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let effective_proxy = override_proxy_selection(
                proxy_override,
                module_proxy.clone(),
                global_proxy.clone(),
            );

            if let Some(url) = proxy_override {
                info!(
                    target: "jupiter",
                    proxy = %url,
                    "Jupiter API 请求将通过配置的代理发送"
                );
            } else if let Some(selection) = module_proxy {
                info!(
                    target: "jupiter",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Jupiter API 请求将通过 profile 代理发送"
                );
            } else if let Some(selection) = global_proxy.clone() {
                info!(
                    target: "jupiter",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Jupiter API 请求将通过全局代理发送"
                );
            }

            let api_http_client =
                build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
            let api_client_pool = build_http_client_pool(effective_proxy.clone(), false, None);
            let api_client = JupiterApiClient::with_ip_pool(
                api_http_client,
                quote_base,
                swap_base,
                &config.galileo.engine.time_out,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::Jupiter { api_client }
        }
        crate::config::EngineBackend::JupiterSelfHosted => {
            let jupiter_cfg =
                resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
            let needs_jupiter = command_needs_jupiter(&cli.command, &config);
            let launch_overrides = build_launch_overrides(
                &config.galileo.intermedium,
                &config.galileo.engine.jupiter_self_hosted,
            );
            let base_url = resolve_jupiter_base_url(&jupiter_cfg);
            let base_endpoint = base_url.trim_end_matches('/');
            // 自托管版本提供 `/quote` 与 `/swap-instructions` 原生端点。
            let quote_url = format!("{}/quote", base_endpoint);
            let swap_url = format!("{}/swap-instructions", base_endpoint);

            let proxy_override =
                resolve_self_hosted_jupiter_api_proxy(&config.galileo.engine.jupiter_self_hosted);
            let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let effective_proxy = override_proxy_selection(
                proxy_override.as_deref(),
                module_proxy.clone(),
                global_proxy.clone(),
            );
            let mut bypass_proxy = should_bypass_proxy(&base_url);
            if proxy_override.is_some() {
                bypass_proxy = false;
            }

            if let Some(url) = proxy_override.as_deref() {
                info!(
                    target: "jupiter",
                    proxy = %url,
                    "Jupiter API 请求将通过配置的代理发送"
                );
            } else if let Some(selection) = module_proxy.as_ref() {
                info!(
                    target: "jupiter",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Jupiter API 请求将通过 profile 代理发送"
                );
            } else if let Some(selection) = global_proxy.as_ref() {
                info!(
                    target: "jupiter",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Jupiter API 请求将通过全局代理发送"
                );
            } else if bypass_proxy {
                info!(
                    target: "jupiter",
                    base_url = %base_url,
                    "Jupiter API 请求将绕过代理（本地地址）"
                );
            }

            let user_agent = crate::jupiter::updater::USER_AGENT;
            let api_http_client = build_http_client_with_options(
                effective_proxy.as_ref(),
                bypass_proxy,
                None,
                Some(user_agent),
            )?;
            let api_client_pool = build_http_client_pool(
                effective_proxy.clone(),
                bypass_proxy,
                Some(user_agent.to_string()),
            );
            let manager = JupiterBinaryManager::new(
                jupiter_cfg,
                launch_overrides,
                config.galileo.bot.binary.disable_local_binary,
                config.galileo.bot.binary.show_logs,
                needs_jupiter,
            )?;
            let api_client = JupiterApiClient::with_ip_pool(
                api_http_client,
                quote_url,
                swap_url,
                &config.galileo.engine.time_out,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::JupiterSelfHosted {
                manager,
                api_client,
            }
        }
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
            let proxy_override = dflow_proxy.as_deref();
            let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let effective_proxy = override_proxy_selection(
                proxy_override,
                module_proxy.clone(),
                global_proxy.clone(),
            );

            if let Some(url) = proxy_override {
                info!(
                    target: "dflow",
                    proxy = %url,
                    "DFlow API 请求将通过配置的代理发送"
                );
            } else if let Some(selection) = module_proxy {
                info!(
                    target: "dflow",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "DFlow API 请求将通过 profile 代理发送"
                );
            } else if let Some(selection) = global_proxy.clone() {
                info!(
                    target: "dflow",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "DFlow API 请求将通过全局代理发送"
                );
            }

            let api_http_client =
                build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
            let api_client_pool = build_http_client_pool(effective_proxy.clone(), false, None);
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
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let effective_proxy =
                override_proxy_selection(kamino_proxy, module_proxy.clone(), global_proxy.clone());

            if let Some(url) = kamino_proxy {
                info!(
                    target: "kamino",
                    proxy = %url,
                    "Kamino API 请求将通过配置的代理发送"
                );
            } else if let Some(selection) = module_proxy {
                info!(
                    target: "kamino",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Kamino API 请求将通过 profile 代理发送"
                );
            } else if let Some(selection) = global_proxy.clone() {
                info!(
                    target: "kamino",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Kamino API 请求将通过全局代理发送"
                );
            }
            let api_http_client =
                build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
            let api_client_pool = build_http_client_pool(effective_proxy.clone(), false, None);
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
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let effective_proxy =
                override_proxy_selection(ultra_proxy, module_proxy.clone(), global_proxy.clone());

            if let Some(url) = ultra_proxy {
                info!(
                    target: "ultra",
                    proxy = %url,
                    "Ultra API 请求将通过配置的代理发送"
                );
            } else if let Some(selection) = module_proxy {
                info!(
                    target: "ultra",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Ultra API 请求将通过 profile 代理发送"
                );
            } else if let Some(selection) = global_proxy.clone() {
                info!(
                    target: "ultra",
                    proxy = %selection.url,
                    per_request = selection.per_request,
                    "Ultra API 请求将通过全局代理发送"
                );
            }
            let http_client =
                build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
            let http_pool = build_http_client_pool(effective_proxy.clone(), false, None);
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

    let result = dispatch(cli.command, config, aggregator).await;

    if let Some(guard) = managed_self_hosted.take() {
        guard.shutdown().await;
    }

    result
}

async fn dispatch(
    command: Command,
    config: AppConfig,
    aggregator: AggregatorContext,
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
            AggregatorContext::Jupiter { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager: None,
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::JupiterSelfHosted {
                manager,
                api_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager: Some(manager),
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::Kamino {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::Ultra {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Ultra {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::None => {
                let backend = crate::cli::strategy::StrategyBackend::None;
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
        },
        Command::StrategyDryRun => match &aggregator {
            AggregatorContext::Jupiter { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager: None,
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::JupiterSelfHosted {
                manager,
                api_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager: Some(manager),
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::Kamino {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::Ultra {
                api_client,
                rpc_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Ultra {
                    api_client,
                    rpc_client: rpc_client.clone(),
                };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::None => {
                let backend = crate::cli::strategy::StrategyBackend::None;
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
        },
        Command::Init(args) => {
            init_configs(args)?;
        }
        Command::Wallet(_) => {}
        Command::Jupiter(_) => unreachable!("Jupiter 命令已在入口提前处理"),
    }

    Ok(())
}

fn command_needs_jupiter(command: &Command, config: &AppConfig) -> bool {
    match command {
        Command::Run | Command::StrategyDryRun => config
            .galileo
            .bot
            .strategy_enabled(StrategyToggle::BlindStrategy),
        Command::Jupiter(_) => true,
        _ => false,
    }
}

async fn maybe_start_managed_self_hosted(
    command: &Command,
    config: &AppConfig,
) -> Result<Option<ManagedSelfHostedGuard>> {
    let binary_cfg = &config.galileo.bot.binary;
    if !binary_cfg.enable_running {
        return Ok(None);
    }
    if !matches!(command, Command::Run | Command::StrategyDryRun) {
        return Ok(None);
    }
    if matches!(
        config.galileo.engine.backend,
        crate::config::EngineBackend::JupiterSelfHosted
    ) {
        return Ok(None);
    }
    if !config.galileo.engine.jupiter_self_hosted.enable {
        warn!(
            target: "jupiter",
            "bot.binary.enable_running = true 但 engine.jupiter_self_hosted.enable = false，跳过托管"
        );
        return Ok(None);
    }
    if !config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::BlindStrategy)
    {
        return Ok(None);
    }

    let jupiter_cfg = resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
    let launch_overrides = build_launch_overrides(
        &config.galileo.intermedium,
        &config.galileo.engine.jupiter_self_hosted,
    );
    let manager = JupiterBinaryManager::new(
        jupiter_cfg,
        launch_overrides,
        binary_cfg.disable_local_binary,
        binary_cfg.show_logs,
        true,
    )?;

    let started_new = match manager.start(false).await {
        Ok(()) => {
            if manager.disable_local_binary {
                info!(
                    target: "jupiter",
                    "bot.binary.disable_local_binary = true，跳过本地 Jupiter 启动"
                );
                false
            } else {
                info!(
                    target: "jupiter",
                    "bot.binary.enable_running 已启动本地 Jupiter 二进制"
                );
                true
            }
        }
        Err(JupiterError::AlreadyRunning) => {
            info!(
                target: "jupiter",
                "检测到本地 Jupiter 已运行，复用现有进程"
            );
            false
        }
        Err(err) => return Err(anyhow!(err)),
    };

    Ok(Some(ManagedSelfHostedGuard {
        manager,
        started_new,
    }))
}

struct ManagedSelfHostedGuard {
    manager: JupiterBinaryManager,
    started_new: bool,
}

impl ManagedSelfHostedGuard {
    async fn shutdown(self) {
        if !self.started_new {
            return;
        }
        if let Err(err) = self.manager.stop().await {
            warn!(
                target: "jupiter",
                error = %err,
                "bot.binary.enable_running 无法停止本地 Jupiter 二进制"
            );
        } else {
            info!(
                target: "jupiter",
                "bot.binary.enable_running 已停止本地 Jupiter 二进制"
            );
        }
    }
}
