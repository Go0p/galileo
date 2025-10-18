use std::str::FromStr;

use anyhow::{Result, anyhow};
use tracing::info;

use crate::api::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest,
};
use crate::cli::args::{Cli, Command};
use crate::cli::context::{
    build_launch_overrides, ensure_running, init_configs, resolve_instruction_memo,
    resolve_jupiter_api_proxy, resolve_jupiter_base_url, resolve_jupiter_defaults,
    should_bypass_proxy,
};
use crate::cli::jupiter::handle_jupiter_cmd;
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{StrategyMode, run_strategy};
use crate::cli::utils::apply_quote_defaults;
use crate::config::AppConfig;
use crate::jupiter::JupiterBinaryManager;

pub async fn run(cli: Cli, config: AppConfig) -> Result<()> {
    if config.galileo.bot.prometheus.enable {
        crate::monitoring::try_init_prometheus(&config.galileo.bot.prometheus.listen)
            .map_err(|err| anyhow!(err))?;
    }

    let jupiter_cfg = resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
    let needs_jupiter = matches!(
        cli.command,
        Command::Jupiter(_)
            | Command::Quote(_)
            | Command::SwapInstructions(_)
            | Command::Strategy
            | Command::StrategyDryRun
    );

    let launch_overrides =
        build_launch_overrides(&config.galileo.engine.jupiter, &config.galileo.intermedium);
    let base_url = resolve_jupiter_base_url(&config.galileo.bot, &jupiter_cfg);
    let api_proxy = resolve_jupiter_api_proxy(&config.galileo.engine);
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
        api_http_builder = api_http_builder.proxy(proxy);
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

    dispatch(cli.command, config, manager, api_client).await
}

async fn dispatch(
    command: Command,
    config: AppConfig,
    manager: JupiterBinaryManager,
    api_client: JupiterApiClient,
) -> Result<()> {
    // 统一的命令分发入口，便于后续按子命令拆分到专门模块。
    match command {
        Command::Jupiter(cmd) => {
            handle_jupiter_cmd(cmd, &manager).await?;
        }
        Command::Lander(cmd) => {
            handle_lander_cmd(
                cmd,
                &config,
                &config.lander.lander,
                resolve_instruction_memo(&config.galileo.global.instruction),
            )
            .await?;
        }
        Command::Quote(args) => {
            ensure_running(&manager).await?;
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
        Command::SwapInstructions(args) => {
            ensure_running(&manager).await?;
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
        Command::Strategy => {
            run_strategy(&config, &manager, &api_client, StrategyMode::Live).await?;
        }
        Command::StrategyDryRun => {
            run_strategy(&config, &manager, &api_client, StrategyMode::DryRun).await?;
        }
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
