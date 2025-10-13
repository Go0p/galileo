# Galileo Jupiter Demo (Rust)

本文演示如何用 Rust 直接调用自托管的 Jupiter Quote / Swap API，并将返回的指令组装成 Versioned v0 交易后发送至 Jito。示例严格对齐 `galileo.yaml` 中的 `request_params` 与三类策略开关（`spam`、`blind`、`back_run`），并遵循“基准资产 ↔ 中间资产 ↔ 基准资产”的套利约束：`inputMint` / `outputMint` 永远选自策略配置的基准资产（例如 WSOL、USDC），`intermedium.mints` 仅用作中间流动性池。

## 请求参数对齐

| 配置路径 | 作用域 | Demo 中的字段 | 说明 |
| --- | --- | --- | --- |
| `request_params.included_dexes` | /quote, /swap-instructions | `INCLUDED_DEXES` + `QuoteRequest::dexes` | 将请求限制在白名单 Dex，满足 spam/back_run 白名单要求。 |
| `request_params.only_direct_routes` | /quote | `StrategyKind::only_direct_routes` | spam 策略强制 2-hop，其余策略沿用配置中的 `false`。 |
| `request_params.restrict_intermediate_tokens` | /quote | `REQUEST_DEFAULTS.restrict_intermediate_tokens` | blind/back_run 默认启用，保持顶级流动性中间池。 |
| `request_params.skip_user_accounts_rpc_calls` | /swap-instructions | `StrategyKind::skip_user_accounts_rpc_calls` | spam 为降低延迟可改为 `true`，其余策略延续配置。 |
| `request_params.dynamic_compute_unit_limit` | /swap-instructions | `REQUEST_DEFAULTS.dynamic_compute_unit_limit` | 三个策略都推荐开启，自动估计 CU。 |
| `global.wallet.warp_or_unwrap_sol` | /swap-instructions | `REQUEST_DEFAULTS.wrap_and_unwrap_sol` | 与机器人保持一致，确保 SOL ↔ WSOL 自行包裹。 |
| `spam.enable_landers` | Bundle 落地 | `JITO_TIP_ACCOUNT` | Demo 选用 `jito` 通道，向示例账户支付小费。 |
| `back_run.base_mints[*].min_quote_profit` 等阈值 | 策略过滤 | `StrategyKind::min_profit_lamports` | 依据策略对利润阈值做差异化设置。 |

## 策略关注点

- **基准资产循环**：quote 的输入/输出资产由策略配置决定，可形成 A→B→A 或 A→B→C→A 等多跳结构；`intermedium.mints` 定义可参与路由的中间资产集合。
- **双 quote 决策**：连续获取 A→B 与 B→A 报价，确认利润后，将第一次 quote 的原始响应（可追加 tip 调整）传入 `/swap-instructions` 获取核心 swap 指令。
- **指令后处理**：只对 Jupiter 返回的指令做补充（memo、Jito tip、闪电贷等），不修改核心 swap 序列。
- **高频特性**：quote 请求实时命中 Jupiter，不做本地缓存，以避免利润遗漏。
- **落地路径**：若配置的 `enable_landers` 为空则默认走 RPC；`staked` 等值与 RPC 共享发送流程。
- **spam**：高频直连、`onlyDirectRoutes=true`，并开启 `skip_user_accounts_rpc_calls`；重试次数由 `spam.max_retries` 控制。
- **blind**：沿用默认配置，按 `blind.base_mints[*].trade_size_range` 生成交易规模，关注三跳白名单。
- **back_run**：利润阈值与 tip 更激进，可结合后续 DEX gRPC 监听扩展。

## 依赖声明

在 `Cargo.toml` 中添加：

```toml
[dependencies]
anyhow = "1"
base64 = "0.22"
bs58 = "0.5"
dotenvy = "0.15"
reqwest = { version = "0.12", features = ["json", "gzip"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
solana-client = { version = "1.18", features = ["nonblocking"] }
solana-sdk = "1.18"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

## Rust 示例

示例读取环境变量：

- `GALILEO_PRIVATE_KEY`：对应 `global.wallet.private_key`，可为 base58 字符串或 64 字节 JSON 数组。
- `GALILEO_RPC_URL`：覆盖 `global.rpc_url`。
- `JUPITER_URL`：覆盖请求 Jupiter 的基地址。
- `GALILEO_STRATEGY`：取值 `spam`、`blind`、`back_run`，默认 `blind`。
- （可选）`GALILEO_MEMO`：追加 Memo 指令，贴合 `global.instruction.memo`。

```rust
use std::{str::FromStr, time::Instant};

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bs58;
use dotenvy::dotenv;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::v0::{AddressLookupTableAccount, Message as V0Message},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::VersionedTransaction,
};
use tokio::time::{sleep, Duration};

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const INCLUDED_DEXES: &[&str] = &[
    "9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp",
    "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH",
    "SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF",
    "SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe",
    "ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY",
];
const JITO_TIP_ACCOUNT: &str = "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY";
const BUNDLE_ENDPOINT: &str = "https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/bundles";

struct RequestDefaults {
    only_direct_routes: bool,
    restrict_intermediate_tokens: bool,
    slippage_bps: u16,
    max_accounts: u8,
    wrap_and_unwrap_sol: bool,
    compute_unit_price_micro_lamports: u64,
    dynamic_compute_unit_limit: bool,
    skip_user_accounts_rpc_calls: bool,
}

const REQUEST_DEFAULTS: RequestDefaults = RequestDefaults {
    only_direct_routes: false,
    restrict_intermediate_tokens: true,
    slippage_bps: 0,
    max_accounts: 20,
    wrap_and_unwrap_sol: true,
    compute_unit_price_micro_lamports: 0,
    dynamic_compute_unit_limit: true,
    skip_user_accounts_rpc_calls: false,
};

#[derive(Clone, Copy)]
enum StrategyKind {
    Spam,
    Blind,
    BackRun,
}

impl StrategyKind {
    fn from_env() -> Self {
        match std::env::var("GALILEO_STRATEGY")
            .unwrap_or_else(|_| "blind".to_string())
            .to_lowercase()
            .as_str()
        {
            "spam" => StrategyKind::Spam,
            "back_run" | "backrun" => StrategyKind::BackRun,
            _ => StrategyKind::Blind,
        }
    }

    fn sample_amount(&self) -> u64 {
        match self {
            StrategyKind::Spam => 5_000_000,      // 0.005 SOL
            StrategyKind::Blind => 10_000_000,    // 0.01 SOL
            StrategyKind::BackRun => 100_000_000, // 0.1 SOL
        }
    }

    fn only_direct_routes(&self, default: bool) -> bool {
        match self {
            StrategyKind::Spam => true,
            _ => default,
        }
    }

    fn skip_user_accounts_rpc_calls(&self, default: bool) -> bool {
        match self {
            StrategyKind::Spam => true,
            _ => default,
        }
    }

    fn min_profit_lamports(&self) -> u64 {
        match self {
            StrategyKind::Spam => 1_000,
            StrategyKind::Blind => 5_000,
            StrategyKind::BackRun => 100_000,
        }
    }

    fn tip_ratio(&self) -> f64 {
        match self {
            StrategyKind::Spam => 0.3,
            StrategyKind::Blind => 0.5,
            StrategyKind::BackRun => 0.6,
        }
    }

    fn loop_delay_ms(&self) -> u64 {
        match self {
            StrategyKind::Spam => 100,
            StrategyKind::Blind => 200,
            StrategyKind::BackRun => 400,
        }
    }
}

#[derive(Serialize)]
struct QuoteRequest<'a> {
    #[serde(rename = "inputMint")]
    input_mint: &'a str,
    #[serde(rename = "outputMint")]
    output_mint: &'a str,
    amount: u64,
    #[serde(rename = "onlyDirectRoutes")]
    only_direct_routes: bool,
    #[serde(rename = "restrictIntermediateTokens")]
    restrict_intermediate_tokens: bool,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "maxAccounts")]
    max_accounts: u8,
    dexes: &'a str,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let strategy = StrategyKind::from_env();
    let dexes = INCLUDED_DEXES.join(",");
    let payer = load_keypair_from_env("GALILEO_PRIVATE_KEY")?;
    let memo = std::env::var("GALILEO_MEMO").unwrap_or_default();

    let rpc_url = std::env::var("GALILEO_RPC_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "https://rpc.shyft.to?api_key=FgU1AVt-7pIiUk2j".to_string());
    let rpc = RpcClient::new(rpc_url);

    let base_url = std::env::var("JUPITER_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "http://172.22.166.244:18080".to_string());

    let client = Client::builder().build()?;
    let tip_recipient = Pubkey::from_str(JITO_TIP_ACCOUNT)?;

    loop {
        let start = Instant::now();
        let forward_amount = strategy.sample_amount();
        let quote0_params = QuoteRequest {
            input_mint: WSOL_MINT,
            output_mint: USDC_MINT,
            amount: forward_amount,
            only_direct_routes: strategy.only_direct_routes(REQUEST_DEFAULTS.only_direct_routes),
            restrict_intermediate_tokens: REQUEST_DEFAULTS.restrict_intermediate_tokens,
            slippage_bps: REQUEST_DEFAULTS.slippage_bps,
            max_accounts: REQUEST_DEFAULTS.max_accounts,
            dexes: &dexes,
        };

        let quote0: Value = client
            .get(format!("{base_url}/quote"))
            .query(&quote0_params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mid_amount = parse_amount(&quote0, "outAmount")?;
        let quote1_params = QuoteRequest {
            input_mint: USDC_MINT,
            output_mint: WSOL_MINT,
            amount: mid_amount,
            only_direct_routes: strategy.only_direct_routes(REQUEST_DEFAULTS.only_direct_routes),
            restrict_intermediate_tokens: REQUEST_DEFAULTS.restrict_intermediate_tokens,
            slippage_bps: REQUEST_DEFAULTS.slippage_bps,
            max_accounts: REQUEST_DEFAULTS.max_accounts,
            dexes: &dexes,
        };

        let quote1: Value = client
            .get(format!("{base_url}/quote"))
            .query(&quote1_params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let backward_amount = parse_amount(&quote1, "outAmount")?;
        let diff = backward_amount.saturating_sub(forward_amount);
        println!("profit candidate (lamports): {diff}");

        if diff <= strategy.min_profit_lamports() {
            sleep(Duration::from_millis(strategy.loop_delay_ms())).await;
            continue;
        }

        let tip_lamports = (diff as f64 * strategy.tip_ratio()).floor() as u64;
        let target_lamports = forward_amount + tip_lamports;
        // 以第一段 quote 为基础，仅追加 profit/tip 信息，让 `/swap-instructions` 在原始路径上输出指令
        let mut merged_quote = quote0.clone();
        merge_quotes(&mut merged_quote, &quote1, target_lamports)?;

        let swap_payload = json!({
            "userPublicKey": payer.pubkey().to_string(),
            "wrapAndUnwrapSol": REQUEST_DEFAULTS.wrap_and_unwrap_sol,
            "useSharedAccounts": false,
            "computeUnitPriceMicroLamports": REQUEST_DEFAULTS.compute_unit_price_micro_lamports,
            "dynamicComputeUnitLimit": REQUEST_DEFAULTS.dynamic_compute_unit_limit,
            "skipUserAccountsRpcCalls": strategy.skip_user_accounts_rpc_calls(
                REQUEST_DEFAULTS.skip_user_accounts_rpc_calls
            ),
            "quoteResponse": merged_quote,
        });

        let swap_response: Value = client
            .post(format!("{base_url}/swap-instructions"))
            .json(&swap_payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut instructions = Vec::new();
        let compute_unit_limit = swap_response
            .get("computeUnitLimit")
            .and_then(Value::as_u64)
            .unwrap_or(200_000);
        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            compute_unit_limit as u32,
        ));

        if REQUEST_DEFAULTS.compute_unit_price_micro_lamports > 0 {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
                REQUEST_DEFAULTS.compute_unit_price_micro_lamports,
            ));
        }

        append_instructions(&mut instructions, swap_response.get("setupInstructions"))?;

        if let Some(swap_ix) = swap_response.get("swapInstruction") {
            instructions.push(value_to_instruction(swap_ix)?);
        }

        append_instructions(&mut instructions, swap_response.get("cleanupInstructions"))?;
        append_instructions(&mut instructions, swap_response.get("addressTableLookupsAsInstructions"))?;

        if !memo.is_empty() {
            instructions.push(build_memo(&memo)?);
        }

        if tip_lamports > 0 {
            instructions.push(system_instruction::transfer(
                &payer.pubkey(),
                &tip_recipient,
                tip_lamports,
            ));
        }

        let lookup_accounts = load_lookup_tables(&rpc, swap_response.get("addressLookupTableAddresses")).await?;
        let blockhash = rpc.get_latest_blockhash().await?;

        let message = compile_message(&payer.pubkey(), &instructions, &lookup_accounts, blockhash)?;
        let mut transaction = VersionedTransaction::try_new(message, &[&payer])?;

        let serialized = transaction.serialize();
        let base58_tx = bs58::encode(serialized).into_string();
        let bundle = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [[base58_tx]]
        });

        let bundle_resp: Value = client
            .post(BUNDLE_ENDPOINT)
            .json(&bundle)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let elapsed = start.elapsed().as_millis();
        let slot = merged_quote
            .get("contextSlot")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        println!(
            "[{:?}] slot={slot} profit={} tip={} elapsed={}ms bundle_id={}",
            strategy_label(strategy),
            diff,
            tip_lamports,
            elapsed,
            bundle_resp.get("result").unwrap_or(&Value::Null)
        );

        sleep(Duration::from_millis(strategy.loop_delay_ms())).await;
    }
}

fn strategy_label(strategy: StrategyKind) -> &'static str {
    match strategy {
        StrategyKind::Spam => "spam",
        StrategyKind::Blind => "blind",
        StrategyKind::BackRun => "back_run",
    }
}

fn load_keypair_from_env(key: &str) -> Result<Keypair> {
    let raw = std::env::var(key).map_err(|_| anyhow!("{key} not set"))?;
    if raw.trim_start().starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(&raw)?;
        return Keypair::from_bytes(&bytes).map_err(|_| anyhow!("invalid keypair bytes"));
    }
    let bytes = bs58::decode(raw.trim()).into_vec()?;
    Keypair::from_bytes(&bytes).map_err(|_| anyhow!("invalid keypair base58"))
}

fn parse_amount(value: &Value, field: &str) -> Result<u64> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{} missing in quote response", field))?
        .parse::<u64>()
        .map_err(|_| anyhow!("{} not numeric", field))
}

fn merge_quotes(merged: &mut Value, backward: &Value, target_lamports: u64) -> Result<()> {
    let obj = merged
        .as_object_mut()
        .ok_or_else(|| anyhow!("quote response is not an object"))?;

    obj.insert(
        "outputMint".to_string(),
        backward
            .get("outputMint")
            .cloned()
            .ok_or_else(|| anyhow!("backward quote missing outputMint"))?,
    );
    let target = Value::String(target_lamports.to_string());
    obj.insert("outAmount".to_string(), target.clone());
    obj.insert("otherAmountThreshold".to_string(), target);
    obj.insert("priceImpactPct".to_string(), Value::String("0".to_string()));

    if let Some(route0) = obj.get_mut("routePlan").and_then(Value::as_array_mut) {
        if let Some(route1) = backward.get("routePlan").and_then(Value::as_array) {
            route0.extend(route1.iter().cloned());
        }
    }
    Ok(())
}

fn value_to_instruction(value: &Value) -> Result<Instruction> {
    let program_id = value
        .get("programId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("instruction missing programId"))?;
    let program_id = Pubkey::from_str(program_id)?;
    let accounts = value
        .get("accounts")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("instruction accounts missing"))?
        .iter()
        .map(|meta| {
            let pubkey = meta
                .get("pubkey")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("account pubkey missing"))?;
            let pubkey = Pubkey::from_str(pubkey)?;
            let is_signer = meta
                .get("isSigner")
                .and_then(Value::as_bool)
                .ok_or_else(|| anyhow!("account isSigner missing"))?;
            let is_writable = meta
                .get("isWritable")
                .and_then(Value::as_bool)
                .ok_or_else(|| anyhow!("account isWritable missing"))?;
            Ok(AccountMeta {
                pubkey,
                is_signer,
                is_writable,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let data = value
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("instruction data missing"))?;
    let data = BASE64.decode(data)?;

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

fn append_instructions(target: &mut Vec<Instruction>, value: Option<&Value>) -> Result<()> {
    if let Some(Value::Array(instructions)) = value {
        for ix in instructions {
            target.push(value_to_instruction(ix)?);
        }
    } else if let Some(single) = value {
        if !single.is_null() {
            target.push(value_to_instruction(single)?);
        }
    }
    Ok(())
}

async fn load_lookup_tables(
    rpc: &RpcClient,
    addresses: Option<&Value>,
) -> Result<Vec<AddressLookupTableAccount>> {
    let mut tables = Vec::new();
    if let Some(Value::Array(list)) = addresses {
        for address in list {
            let addr = address
                .as_str()
                .ok_or_else(|| anyhow!("lookup address not string"))?;
            let pubkey = Pubkey::from_str(addr)?;
            if let Some(table) = rpc.get_address_lookup_table(&pubkey).await? {
                tables.push(table);
            }
        }
    }
    Ok(tables)
}

fn compile_message(
    payer: &Pubkey,
    instructions: &[Instruction],
    tables: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> Result<V0Message> {
    let refs: Vec<&AddressLookupTableAccount> = tables.iter().collect();
    V0Message::try_compile(payer, instructions, &refs, blockhash)
        .map_err(|err| anyhow!("compile message failed: {err}"))
}

fn build_memo(text: &str) -> Result<Instruction> {
    let program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")?;
    Ok(Instruction {
        program_id,
        accounts: vec![],
        data: text.as_bytes().to_vec(),
    })
}
```

> **提示**：示例以纯 HTTP 周期验证为目的，真实策略可将 `StrategyKind` 替换为读取 `galileo.yaml` 的配置结构，或将利润阈值与交易规模改为实时计算（例如依据 `back_run.trade_configs` 动态生成 `amount`）。此外，`build_memo` 可根据 `back_run.trigger_memo` 或 `global.instruction.memo` 设置不同备注。最后，若需改用 `staked` 等其它落地渠道，请替换 `JITO_TIP_ACCOUNT` 并调整 bundle 提交接口。*** End Patch
