use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tracing::info;

use crate::api::SwapInstructionsResponse;
use crate::cli::args::{LanderCmd, LanderSendArgs};
use crate::cli::context::resolve_rpc_client;
use crate::config;
use crate::config::AppConfig;
use crate::engine::{BuilderConfig, EngineIdentity, TransactionBuilder};
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
        config.galileo.request_params.skip_user_accounts_rpc_calls,
    );

    let builder_config = BuilderConfig::new(memo);
    let builder = TransactionBuilder::new(rpc_client.clone(), builder_config);

    let submission_client = reqwest::Client::builder().build()?;
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

    let prepared = builder
        .build(&identity, &instructions, args.tip_lamports)
        .await
        .map_err(|err| anyhow!(err))?;

    let deadline =
        Deadline::from_instant(Instant::now() + Duration::from_millis(args.deadline_ms.max(1)));
    let receipt = lander_stack
        .submit(&prepared, deadline, "lander-test")
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
