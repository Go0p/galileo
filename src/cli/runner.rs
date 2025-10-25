use std::{str::FromStr, time::Duration};

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest,
};
use crate::cli::args::{Cli, Command};
use crate::cli::context::{
    build_launch_overrides, ensure_running, init_configs, resolve_global_http_proxy,
    resolve_instruction_memo, resolve_jupiter_api_proxy, resolve_jupiter_base_url,
    resolve_jupiter_defaults, should_bypass_proxy,
};
use crate::cli::jupiter::handle_jupiter_cmd;
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{StrategyMode, run_strategy};
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
            let api_proxy = resolve_jupiter_api_proxy(&config.galileo.engine);
            let global_proxy = resolve_global_http_proxy(&config.galileo.global);
            let bypass_proxy = should_bypass_proxy(&base_url);
            let mut api_http_builder =
                reqwest::Client::builder().user_agent(crate::jupiter::updater::USER_AGENT);
            if let Some(proxy_url) = api_proxy {
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|err| anyhow!("Jupiter API 代理地址无效 {proxy_url}: {err}"))?;
                if needs_jupiter {
                    info!(
                        target: "jupiter",
                        proxy = %proxy_url,
                        "Jupiter API 请求将通过配置的代理发送"
                    );
                }
                api_http_builder = api_http_builder
                    .proxy(proxy)
                    .danger_accept_invalid_certs(true);
            } else if let Some(proxy_url) = global_proxy.clone() {
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
                if needs_jupiter {
                    info!(
                        target: "jupiter",
                        proxy = %proxy_url,
                        "Jupiter API 请求将通过 global.proxy 发送"
                    );
                }
                api_http_builder = api_http_builder
                    .proxy(proxy)
                    .danger_accept_invalid_certs(true);
            } else if bypass_proxy {
                if needs_jupiter {
                    info!(
                        target: "jupiter",
                        base_url = %base_url,
                        "Jupiter API 请求绕过 HTTP 代理"
                    );
                }
                api_http_builder = api_http_builder.no_proxy();
            }
            let api_http_client = api_http_builder.build()?;
            let manager = JupiterBinaryManager::new(
                jupiter_cfg,
                launch_overrides,
                config.galileo.bot.disable_local_binary,
                config.galileo.bot.show_jupiter_logs,
                needs_jupiter,
            )?;
            let api_client = JupiterApiClient::new(
                api_http_client,
                base_url,
                &config.galileo.bot,
                &config.galileo.global.logging,
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
            let mut api_http_builder = reqwest::Client::builder();
            let dflow_proxy = config
                .galileo
                .engine
                .dflow
                .api_proxy
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string());
            if let Some(proxy_url) = dflow_proxy {
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|err| anyhow!("DFlow API 代理地址无效 {proxy_url}: {err}"))?;
                info!(
                    target: "dflow",
                    proxy = %proxy_url,
                    "DFlow API 请求将通过配置的代理发送"
                );
                api_http_builder = api_http_builder
                    .proxy(proxy)
                    .danger_accept_invalid_certs(true);
            } else if let Some(proxy_url) = resolve_global_http_proxy(&config.galileo.global) {
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
                api_http_builder = api_http_builder
                    .proxy(proxy)
                    .danger_accept_invalid_certs(true);
            }
            let api_http_client = api_http_builder.build()?;
            let dflow_engine_cfg = &config.galileo.engine.dflow;
            let max_failures = if dflow_engine_cfg.max_consecutive_failures == 0 {
                None
            } else {
                Some(dflow_engine_cfg.max_consecutive_failures as usize)
            };
            let wait_on_429 = Duration::from_millis(dflow_engine_cfg.wait_on_429_ms);
            let api_client = DflowApiClient::new(
                api_http_client,
                quote_base,
                swap_base,
                &config.galileo.bot,
                &config.galileo.global.logging,
                max_failures,
                wait_on_429,
            );
            AggregatorContext::Dflow { api_client }
        }
        crate::config::EngineBackend::Ultra => {
            return Err(anyhow!(
                "Ultra backend 暂未支持 CLI 运行，请使用 jupiter/dflow/none"
            ));
        }
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

                let instructions = api_client.swap_instructions(&request).await?;
                println!("{}", serde_json::to_string_pretty(&instructions.raw)?);
            }
            AggregatorContext::Dflow { .. } => {
                return Err(anyhow!("暂未支持 DFlow swap-instructions 子命令"));
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
