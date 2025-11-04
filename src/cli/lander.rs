use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use std::sync::Arc;

use tracing::info;

use crate::api::dflow::SwapInstructionsResponse as DflowSwapInstructionsResponse;
use crate::cache::AltCache;
use crate::cli::args::{LanderCmd, LanderSendArgs};
use crate::cli::context::{
    override_proxy_selection, resolve_global_http_proxy, resolve_proxy_profile, resolve_rpc_client,
};
use crate::config;
use crate::config::AppConfig;
use crate::config::launch::resources::{
    build_http_client_pool, build_http_client_with_options, build_ip_allocator,
    build_rpc_client_pool,
};
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
    let resolved_rpc = resolve_rpc_client(&config.galileo.global, None)?;
    let rpc_client = resolved_rpc.client.clone();
    let identity = EngineIdentity::from_private_key(&config.galileo.private_key)
        .map_err(|err| anyhow!(err))?;

    let builder_config = BuilderConfig::new(memo).with_yellowstone(
        config.galileo.global.yellowstone_grpc_url.clone(),
        config.galileo.global.yellowstone_grpc_token.clone(),
        config.galileo.bot.get_block_hash_by_grpc,
    );
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let rpc_client_pool =
        build_rpc_client_pool(resolved_rpc.endpoints.clone(), global_proxy.clone());
    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;
    let builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
        AltCache::new(),
        false,
    );
    let lander_proxy = resolve_proxy_profile(&config.galileo.global, "lander");
    let effective_proxy =
        override_proxy_selection(None, lander_proxy.clone(), global_proxy.clone());
    let submission_client =
        build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
    let submission_client_pool = build_http_client_pool(effective_proxy.clone(), false, None);
    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client,
        Some(Arc::clone(&submission_client_pool)),
        false,
        config.galileo.bot.enable_simulation,
    );

    let preferred: Vec<String> = if !args.landers.is_empty() {
        args.landers.clone()
    } else {
        config.galileo.blind_strategy.enable_landers.clone()
    };
    let default_landers = ["rpc"];
    let lander_stack = lander_factory
        .build_stack(
            lander_settings,
            preferred.as_slice(),
            &default_landers,
            0,
            ip_allocator,
        )
        .map_err(|err| anyhow!(err))?;

    let raw = tokio::fs::read_to_string(&args.instructions).await?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let instructions = DflowSwapInstructionsResponse::try_from(value)
        .map_err(|err| anyhow!("解析 Swap 指令失败: {err}"))?;
    let instructions_variant = SwapInstructionsVariant::Dflow(instructions);

    let prepared = builder
        .build(&identity, &instructions_variant, args.tip_lamports, None)
        .await
        .map_err(|err| anyhow!(err))?;

    let dispatch_strategy = lander_settings.sending_strategy;
    let planner = TxVariantPlanner::new();
    let variant_layout = lander_stack.variant_layout(dispatch_strategy);
    let plan = planner.plan(
        dispatch_strategy,
        std::slice::from_ref(&prepared),
        &variant_layout,
    );

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
