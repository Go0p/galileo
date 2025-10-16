use std::str::FromStr;

use anyhow::{Result, anyhow};
use clap::Parser;
use tracing::info;

use crate::api::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest,
};
use crate::cli::args::{Cli, Command, DFlowProbeCmd, ToolCmd};
use crate::cli::context::{
    build_launch_overrides, ensure_running, init_configs, init_tracing, load_configuration,
    resolve_instruction_memo, resolve_jupiter_base_url, resolve_jupiter_defaults,
    should_bypass_proxy,
};
use crate::cli::jupiter::handle_jupiter_cmd;
use crate::cli::lander::handle_lander_cmd;
use crate::cli::strategy::{StrategyMode, run_strategy};
use crate::cli::utils::apply_request_defaults_to_quote;
use crate::config::AppConfig;
use crate::jupiter::JupiterBinaryManager;
use crate::tools;

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = load_configuration(cli.config.clone())?;
    init_tracing(&config.galileo.global.logging)?;

    if config.galileo.bot.prometheus.enable {
        crate::monitoring::try_init_prometheus(&config.galileo.bot.prometheus.listen)
            .map_err(|err| anyhow!(err))?;
    }

    let jupiter_cfg = resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
    let needs_jupiter = matches!(
        cli.command,
        Command::Jupiter(_) | Command::Quote(_) | Command::SwapInstructions(_)
    ) || matches!(cli.command, Command::Strategy | Command::StrategyDryRun)
        && !config.galileo.engine.titan.enable;

    let launch_overrides =
        build_launch_overrides(&config.galileo.request_params, &config.galileo.intermedium);
    let base_url = resolve_jupiter_base_url(&config.galileo.bot, &jupiter_cfg);
    let bypass_proxy = should_bypass_proxy(&base_url);
    if needs_jupiter && bypass_proxy {
        info!(
            target: "jupiter",
            base_url = %base_url,
            "Jupiter API 请求绕过 HTTP 代理"
        );
    }
    let mut api_http_builder =
        reqwest::Client::builder().user_agent(crate::jupiter::updater::USER_AGENT);
    if bypass_proxy {
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
        Command::Tools(tool) => handle_tool_cmd(tool)?,
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
            apply_request_defaults_to_quote(&mut request, &config.galileo.request_params);

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
                request.config.wrap_and_unwrap_sol = flag;
            } else if let Some(default_flag) = config.galileo.request_params.wrap_and_unwrap_sol {
                request.config.wrap_and_unwrap_sol = default_flag;
            }
            request.config.use_shared_accounts = Some(args.shared_accounts);
            if let Some(ref fee) = args.fee_account {
                request.config.fee_account = Some(args.parse_fee_pubkey(fee)?);
            }
            if let Some(price) = args.compute_unit_price {
                request.config.compute_unit_price_micro_lamports =
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

fn handle_tool_cmd(cmd: ToolCmd) -> Result<()> {
    match cmd {
        ToolCmd::DFlowProbe(DFlowProbeCmd {
            template,
            instruction_index,
        }) => tools::dflow_probe::run(&template, instruction_index),
    }
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
