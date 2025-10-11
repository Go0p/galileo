use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use tracing::error;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod jupiter;
mod metrics;

use config::{ConfigError, GalileoConfig, LoggingConfig, load_config};
use jupiter::{
    BinaryStatus, JupiterBinaryManager, JupiterError,
    client::{JupiterApiClient, QuoteRequest, SwapRequest},
};

#[derive(Parser, Debug)]
#[command(
    name = "galileo",
    version,
    about = "Jupiter self-hosted orchestration bot"
)]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to configuration file (defaults to galileo.toml or config/galileo.toml)"
    )]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the Jupiter self-hosted binary (optionally forcing an update)
    Start {
        #[arg(long, help = "Force update before starting the binary")]
        force_update: bool,
    },
    /// Stop the running Jupiter binary
    Stop,
    /// Restart the Jupiter binary
    Restart,
    /// Fetch and install the latest Jupiter binary
    Update,
    /// Print current binary status
    Status,
    /// Request a quote from the Jupiter API
    Quote(QuoteCmd),
    /// Request a swap transaction from the Jupiter API
    Swap(SwapCmd),
}

#[derive(Args, Debug)]
struct QuoteCmd {
    #[arg(long, help = "Input mint address")]
    input: String,
    #[arg(long, help = "Output mint address")]
    output: String,
    #[arg(long, help = "Amount in raw units (lamports/atoms)")]
    amount: u64,
    #[arg(
        long,
        default_value_t = 50,
        help = "Slippage tolerance in basis points"
    )]
    slippage_bps: u16,
    #[arg(long, help = "Restrict to 1-hop direct routes only")]
    direct_only: bool,
    #[arg(
        long,
        help = "Allow intermediate tokens (disables restrictIntermediateTokens flag)"
    )]
    allow_intermediate: bool,
    #[arg(
        long = "extra",
        value_parser = parse_key_val,
        help = "Additional query parameter in the form key=value",
        value_name = "KEY=VALUE"
    )]
    extra: Vec<(String, String)>,
}

#[derive(Args, Debug)]
struct SwapCmd {
    #[arg(long, help = "Path to JSON file containing quoteResponse")]
    quote_path: PathBuf,
    #[arg(long, help = "User public key for the swap request")]
    user: String,
    #[arg(long, help = "Wrap/unwrap SOL automatically")]
    wrap_sol: bool,
    #[arg(long, help = "Use Jupiter shared accounts for the swap")]
    shared_accounts: bool,
    #[arg(long, help = "Optional fee account to collect platform fees")]
    fee_account: Option<String>,
    #[arg(
        long,
        help = "Compute unit price (micro lamports) to request higher priority"
    )]
    compute_unit_price: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_configuration(cli.config.clone())?;
    init_tracing(&config.logging)?;

    let manager = JupiterBinaryManager::new(config.jupiter.clone())?;
    let api_client = JupiterApiClient::new(manager.client.clone(), &config.http);

    match cli.command {
        Command::Start { force_update } => {
            manager.start(force_update).await?;
        }
        Command::Stop => {
            manager.stop().await?;
        }
        Command::Restart => {
            manager.restart().await?;
        }
        Command::Update => {
            manager.update().await?;
        }
        Command::Status => {
            let status = manager.status().await;
            println!("{status:?}");
        }
        Command::Quote(args) => {
            ensure_running(&manager).await?;
            let mut request =
                QuoteRequest::new(args.input, args.output, args.amount, args.slippage_bps);
            request.only_direct_routes = args.direct_only;
            request.restrict_intermediate_tokens = !args.allow_intermediate;
            for (k, v) in args.extra {
                request.extra.insert(k, v);
            }

            let quote = api_client.quote(&request).await?;
            println!("{}", serde_json::to_string_pretty(&quote.raw)?);
        }
        Command::Swap(args) => {
            ensure_running(&manager).await?;
            let quote_raw = tokio::fs::read_to_string(&args.quote_path).await?;
            let quote_value: serde_json::Value = serde_json::from_str(&quote_raw)?;
            let mut request = SwapRequest::new(quote_value, args.user);
            request.wrap_and_unwrap_sol = Some(args.wrap_sol);
            request.use_shared_accounts = Some(args.shared_accounts);
            request.fee_account = args.fee_account;
            request.compute_unit_price_micro_lamports = args.compute_unit_price;

            let swap = api_client.swap(&request).await?;
            println!("{}", serde_json::to_string_pretty(&swap.raw)?);
        }
    }

    Ok(())
}

fn init_tracing(config: &LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    if config.json {
        fmt()
            .with_env_filter(filter)
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .init();
    } else {
        fmt().with_env_filter(filter).init();
    }
    Ok(())
}

fn load_configuration(path: Option<PathBuf>) -> Result<GalileoConfig, ConfigError> {
    load_config(path)
}

async fn ensure_running(manager: &JupiterBinaryManager) -> Result<(), JupiterError> {
    match manager.status().await {
        BinaryStatus::Running => Ok(()),
        status => {
            error!(
                target: "jupiter",
                ?status,
                "Jupiter binary is not running; start it with `galileo start`"
            );
            Err(JupiterError::Schema(format!(
                "binary not running, current status: {status:?}"
            )))
        }
    }
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| "expected key=value format".to_string())?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
