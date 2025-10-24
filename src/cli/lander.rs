use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tracing::info;

use crate::api::jupiter::SwapInstructionsResponse;
use crate::cli::args::{LanderCmd, LanderSendArgs};
use crate::cli::context::{resolve_global_http_proxy, resolve_rpc_client};
use crate::config;
use crate::config::AppConfig;
use crate::engine::{
    BuilderConfig, EngineIdentity, SwapInstructionsVariant, TransactionBuilder, TxVariantPlanner,
};
use crate::lander::{Deadline, LanderFactory};

/// Lander 子命令：用于离线重放 Swap 指令并测试落地器链路。
pub async fn handle_lander_cmd(
    cmd: LanderCmd,
    config: &AppConfig,
    lander_settings: &config::LanderSettings,
    memo: Option<String>,
) -> Result<()> {
    match cmd {
        LanderCmd::Send(args) => send_transaction(args, config, lander_settings, memo).await?,
    }
    Ok(())
}

async fn send_transaction(
    args: LanderSendArgs,
    config: &AppConfig,
    lander_settings: &config::LanderSettings,
    memo: Option<String>,
) -> Result<()> {
    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;
    identity.set_skip_user_accounts_rpc_calls(
        config
            .galileo
            .engine
            .jupiter
            .swap_config
            .skip_user_accounts_rpc_calls,
    );

    let builder_config = BuilderConfig::new(memo);
    let builder = TransactionBuilder::new(rpc_client.clone(), builder_config);

    let mut submission_builder = reqwest::Client::builder();
    if let Some(proxy_url) = resolve_global_http_proxy(&config.galileo.global) {
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        submission_builder = submission_builder
            .proxy(proxy)
            .danger_accept_invalid_certs(true);
    }
    let submission_client = submission_builder.build()?;
    let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client);

    let preferred: Vec<String> = if !args.landers.is_empty() {
        args.landers.clone()
    } else {
        config.galileo.blind_strategy.enable_landers.clone()
    };
    let default_landers = ["rpc"];
    let lander_stack = lander_factory
        .build_stack(lander_settings, preferred.as_slice(), &default_landers, 0)
        .map_err(|err| anyhow!(err))?;

    let raw = tokio::fs::read_to_string(&args.instructions).await?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let instructions = SwapInstructionsResponse::try_from(value)
        .map_err(|err| anyhow!("解析 Swap 指令失败: {err}"))?;
    let instructions_variant = SwapInstructionsVariant::Jupiter(instructions);

    let prepared = builder
        .build(&identity, &instructions_variant, args.tip_lamports)
        .await
        .map_err(|err| anyhow!(err))?;

    let dispatch_strategy = lander_settings.sending_strategy;
    let planner = TxVariantPlanner::new();
    let variant_layout = lander_stack.variant_layout(dispatch_strategy);
    let plan = planner.plan(dispatch_strategy, &prepared, &variant_layout);

    let deadline =
        Deadline::from_instant(Instant::now() + Duration::from_millis(args.deadline_ms.max(1)));
    let receipt = lander_stack
        .submit_plan(&plan, deadline, "lander-test")
        .await
        .map_err(|err| anyhow!(err))?;

    info!(
        target: "lander::cli",
        lander = receipt.lander,
        endpoint = %receipt.endpoint,
        slot = receipt.slot,
        blockhash = %receipt.blockhash,
        signature = receipt.signature.as_deref().unwrap_or("")
    );
    if let Some(signature) = receipt.signature {
        println!("{signature}");
    }

    Ok(())
}
