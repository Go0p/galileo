use std::{str::FromStr, sync::Arc};

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest,
};
use crate::api::kamino::KaminoApiClient;
use crate::api::ultra::UltraApiClient;
use crate::cli::args::{Cli, Command};
use crate::cli::context::{
    build_launch_overrides, ensure_running, init_configs, resolve_global_http_proxy,
    resolve_instruction_memo, resolve_jupiter_api_proxy, resolve_jupiter_base_url,
    resolve_jupiter_defaults, resolve_rpc_client, should_bypass_proxy,
};
use crate::cli::jupiter::handle_jupiter_cmd;
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{
    StrategyMode, build_http_client_pool, build_http_client_with_options, run_strategy,
};
use crate::cli::utils::apply_quote_defaults;
use crate::config::AppConfig;
use crate::jupiter::JupiterBinaryManager;

enum AggregatorContext {
    Jupiter {
        manager: JupiterBinaryManager,
        api_client: JupiterApiClient,
    },
    Dflow {
        api_client: DflowApiClient,
    },
    Kamino {
        api_client: KaminoApiClient,
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

    let aggregator = match config.galileo.engine.backend {
        crate::config::EngineBackend::Jupiter => {
            let jupiter_cfg =
                resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
            let needs_jupiter = command_needs_jupiter(&cli.command, &config);

            let launch_overrides =
                build_launch_overrides(&config.galileo.engine.jupiter, &config.galileo.intermedium);
            let base_url = resolve_jupiter_base_url(&config.galileo.bot, &jupiter_cfg);
            let api_proxy = resolve_jupiter_api_proxy(&config.galileo.engine)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let global_proxy = resolve_global_http_proxy(&config.galileo.global)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let bypass_proxy = should_bypass_proxy(&base_url);

            if let Some(proxy_url) = api_proxy.clone() {
                if needs_jupiter {
                    info!(
                        target: "jupiter",
                        proxy = %proxy_url,
                        "Jupiter API 请求将通过配置的代理发送"
                    );
                }
            } else if let Some(proxy_url) = global_proxy.clone() {
                if needs_jupiter {
                    info!(
                        target: "jupiter",
                        proxy = %proxy_url,
                        "Jupiter API 请求将通过 global.proxy 发送"
                    );
                }
            } else if bypass_proxy && needs_jupiter {
                info!(
                    target: "jupiter",
                    base_url = %base_url,
                    "Jupiter API 请求绕过 HTTP 代理"
                );
            }

            let user_agent = crate::jupiter::updater::USER_AGENT;
            let api_http_client = build_http_client_with_options(
                api_proxy.as_deref(),
                global_proxy.clone(),
                bypass_proxy,
                None,
                Some(user_agent),
            )?;
            let api_client_pool = build_http_client_pool(
                api_proxy.clone(),
                global_proxy.clone(),
                bypass_proxy,
                Some(user_agent.to_string()),
            );
            let manager = JupiterBinaryManager::new(
                jupiter_cfg,
                launch_overrides,
                config.galileo.bot.disable_local_binary,
                config.galileo.bot.show_jupiter_logs,
                needs_jupiter,
            )?;
            let api_client = JupiterApiClient::with_ip_pool(
                api_http_client,
                base_url,
                &config.galileo.bot,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::Jupiter {
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
                &config.galileo.bot,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::Dflow { api_client }
        }
        crate::config::EngineBackend::Kamino => {
            let kamino_cfg = &config.galileo.engine.kamino;
            if !kamino_cfg.enable {
                return Err(anyhow!(
                    "Kamino backend 已选择，但 engine.kamino.enable = false"
                ));
            }
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
                &config.galileo.bot,
                &config.galileo.global.logging,
                Some(api_client_pool),
            );
            AggregatorContext::Kamino { api_client }
        }
        crate::config::EngineBackend::Ultra => {
            let ultra_cfg = &config.galileo.engine.ultra;
            if !ultra_cfg.enable {
                return Err(anyhow!(
                    "Ultra backend 已选择，但 engine.ultra.enable = false"
                ));
            }
            let api_base = ultra_cfg
                .api_quote_base
                .as_ref()
                .ok_or_else(|| anyhow!("ultra.api_quote_base 未配置"))?
                .trim()
                .to_string();
            if api_base.is_empty() {
                return Err(anyhow!("ultra.api_quote_base 不能为空"));
            }
            let resolved_rpc = resolve_rpc_client(&config.galileo.global)?;
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
                &config.galileo.bot,
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
            if config.galileo.blind_strategy.enable {
                return Err(anyhow!(
                    "engine.backend=none 仅支持纯盲发策略，请关闭 blind_strategy.enable"
                ));
            }
            if !config.galileo.pure_blind_strategy.enable {
                warn!(
                    target: "runner",
                    "engine.backend=none 生效，但 pure_blind_strategy 未启用"
                );
            }
            AggregatorContext::None
        }
    };

    dispatch(cli.command, config, aggregator).await
}

async fn dispatch(
    command: Command,
    config: AppConfig,
    aggregator: AggregatorContext,
) -> Result<()> {
    // 统一的命令分发入口，便于后续按子命令拆分到专门模块。
    match command {
        Command::Jupiter(cmd) => match &aggregator {
            AggregatorContext::Jupiter { manager, .. } => {
                handle_jupiter_cmd(cmd, manager).await?;
            }
            AggregatorContext::Dflow { .. } => {
                return Err(anyhow!("DFlow 后端不支持 Jupiter 子命令"));
            }
            AggregatorContext::Kamino { .. } => {
                return Err(anyhow!("Kamino 后端不支持 Jupiter 子命令"));
            }
            AggregatorContext::Ultra { .. } => {
                return Err(anyhow!("Ultra 后端不支持 Jupiter 子命令"));
            }
            AggregatorContext::None => {
                return Err(anyhow!("engine.backend=none 下无法使用 Jupiter 子命令"));
            }
        },
        Command::Lander(cmd) => {
            handle_lander_cmd(
                cmd,
                &config,
                &config.lander.lander,
                resolve_instruction_memo(&config.galileo.global.instruction),
            )
            .await?;
        }
        Command::Quote(args) => match &aggregator {
            AggregatorContext::Jupiter {
                manager,
                api_client,
            } => {
                ensure_running(manager).await?;
                let input = args.parse_input_pubkey()?;
                let output = args.parse_output_pubkey()?;
                let mut request = QuoteRequest::new(input, output, args.amount, args.slippage_bps);
                request.only_direct_routes = Some(args.direct_only);
                request.restrict_intermediate_tokens = Some(!args.allow_intermediate);
                for (k, v) in args.extra {
                    request.extra_query_params.insert(k, v);
                }
                apply_quote_defaults(&mut request, &config.galileo.engine.jupiter.quote_config);

                let quote = api_client.quote(&request).await?;
                println!("{}", serde_json::to_string_pretty(&quote.raw)?);
            }
            AggregatorContext::Dflow { .. } => {
                return Err(anyhow!("暂未支持 DFlow quote 子命令"));
            }
            AggregatorContext::Kamino { .. } => {
                return Err(anyhow!("暂未支持 Kamino quote 子命令"));
            }
            AggregatorContext::Ultra { .. } => {
                return Err(anyhow!("暂未支持 Ultra quote 子命令"));
            }
            AggregatorContext::None => {
                return Err(anyhow!("engine.backend=none 下无法执行 quote 子命令"));
            }
        },
        Command::SwapInstructions(args) => match &aggregator {
            AggregatorContext::Jupiter {
                manager,
                api_client,
            } => {
                ensure_running(manager).await?;
                let quote_raw = tokio::fs::read_to_string(&args.quote_path).await?;
                let quote_value: serde_json::Value = serde_json::from_str(&quote_raw)?;
                let user = args.parse_user_pubkey()?;
                let mut request = SwapInstructionsRequest::new(quote_value, user);
                if let Some(flag) = args.wrap_sol {
                    request.wrap_and_unwrap_sol = flag;
                } else {
                    request.wrap_and_unwrap_sol = config
                        .galileo
                        .engine
                        .jupiter
                        .swap_config
                        .wrap_and_unwrap_sol;
                }
                request.use_shared_accounts = Some(args.shared_accounts);
                if let Some(ref fee) = args.fee_account {
                    request.fee_account = Some(args.parse_fee_pubkey(fee)?);
                }
                if let Some(price) = args.compute_unit_price {
                    request.compute_unit_price_micro_lamports =
                        Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
                }

                let instructions = api_client.swap_instructions(&request, None).await?;
                println!("{}", serde_json::to_string_pretty(&instructions.raw)?);
            }
            AggregatorContext::Dflow { .. } => {
                return Err(anyhow!("暂未支持 DFlow swap-instructions 子命令"));
            }
            AggregatorContext::Kamino { .. } => {
                return Err(anyhow!("暂未支持 Kamino swap-instructions 子命令"));
            }
            AggregatorContext::Ultra { .. } => {
                return Err(anyhow!("暂未支持 Ultra swap-instructions 子命令"));
            }
            AggregatorContext::None => {
                return Err(anyhow!(
                    "engine.backend=none 下无法执行 swap-instructions 子命令"
                ));
            }
        },
        Command::Run => match &aggregator {
            AggregatorContext::Jupiter {
                manager,
                api_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager,
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(&config, &backend, StrategyMode::Live).await?;
            }
            AggregatorContext::Kamino { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino { api_client };
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
            AggregatorContext::Jupiter {
                manager,
                api_client,
            } => {
                let backend = crate::cli::strategy::StrategyBackend::Jupiter {
                    manager,
                    api_client,
                };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::Dflow { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Dflow { api_client };
                run_strategy(&config, &backend, StrategyMode::DryRun).await?;
            }
            AggregatorContext::Kamino { api_client } => {
                let backend = crate::cli::strategy::StrategyBackend::Kamino { api_client };
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
    }

    Ok(())
}

trait QuoteArgsExt {
    fn parse_input_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey>;
    fn parse_output_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey>;
}

impl QuoteArgsExt for crate::cli::args::QuoteCmd {
    fn parse_input_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey> {
        solana_sdk::pubkey::Pubkey::from_str(&self.input)
            .map_err(|err| anyhow!("输入代币 Mint 无效 {}: {err}", self.input))
    }

    fn parse_output_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey> {
        solana_sdk::pubkey::Pubkey::from_str(&self.output)
            .map_err(|err| anyhow!("输出代币 Mint 无效 {}: {err}", self.output))
    }
}

trait SwapArgsExt {
    fn parse_user_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey>;
    fn parse_fee_pubkey(&self, src: &str) -> Result<solana_sdk::pubkey::Pubkey>;
}

impl SwapArgsExt for crate::cli::args::SwapInstructionsCmd {
    fn parse_user_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey> {
        solana_sdk::pubkey::Pubkey::from_str(&self.user)
            .map_err(|err| anyhow!("用户公钥无效 {}: {err}", self.user))
    }

    fn parse_fee_pubkey(&self, src: &str) -> Result<solana_sdk::pubkey::Pubkey> {
        solana_sdk::pubkey::Pubkey::from_str(src)
            .map_err(|err| anyhow!("手续费账户无效 {}: {err}", src))
    }
}

fn command_needs_jupiter(command: &Command, config: &AppConfig) -> bool {
    match command {
        Command::Run | Command::StrategyDryRun => !config.galileo.pure_blind_strategy.enable,
        Command::Jupiter(_) | Command::Quote(_) | Command::SwapInstructions(_) => true,
        _ => false,
    }
}
