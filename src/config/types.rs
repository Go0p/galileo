use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::strategy::config::StrategyConfig;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GalileoConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    pub strategy: Option<StrategyConfig>,
    #[serde(default)]
    pub request_params: JupiterRequestParamsConfig,
    #[serde(default)]
    pub intermedium: IntermediumConfig,
}

impl Default for GalileoConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            bot: BotConfig::default(),
            strategy: None,
            request_params: JupiterRequestParamsConfig::default(),
            intermedium: IntermediumConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub galileo: GalileoConfig,
    #[allow(dead_code)]
    pub lander: LanderConfig,
    pub jupiter: JupiterConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "LoggingConfig::default_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            json: false,
        }
    }
}

impl LoggingConfig {
    fn default_level() -> String {
        "info".to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    #[allow(dead_code)]
    pub rpc_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub yellowstone_grpc_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub yellowstone_grpc_token: Option<String>,
    #[serde(default = "BotConfig::default_jupiter_api_url")]
    pub jupiter_api_url: String,
    #[serde(default = "BotConfig::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default)]
    pub identity: BotIdentityConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            rpc_url: None,
            yellowstone_grpc_url: None,
            yellowstone_grpc_token: None,
            jupiter_api_url: Self::default_jupiter_api_url(),
            request_timeout_ms: Self::default_request_timeout_ms(),
            identity: BotIdentityConfig::default(),
        }
    }
}

impl BotConfig {
    fn default_jupiter_api_url() -> String {
        "http://127.0.0.1:8080".to_string()
    }

    fn default_request_timeout_ms() -> u64 {
        2_000
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotIdentityConfig {
    #[serde(default)]
    pub user_pubkey: Option<String>,
    #[serde(default)]
    pub fee_account: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub tip_account: Option<String>,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default)]
    pub use_shared_accounts: bool,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(default)]
    pub skip_user_accounts_rpc_calls: Option<bool>,
}

impl Default for BotIdentityConfig {
    fn default() -> Self {
        Self {
            user_pubkey: None,
            fee_account: None,
            tip_account: None,
            wrap_and_unwrap_sol: false,
            use_shared_accounts: false,
            compute_unit_price_micro_lamports: Some(1),
            skip_user_accounts_rpc_calls: Some(true),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterRequestParamsConfig {
    /// 当只想复用线上 Jupiter API 时，可设置 `request_params.api_url` 覆盖 `bot.jupiter_api_url`。
    #[serde(default)]
    pub api_url: Option<String>,
    /// Quote 请求允许的 DEX Program ID 列表（模板字段 `jupiter.included_dexes`）。
    /// 完整地址清单可参考 https://lite-api.jup.ag/swap/v1/program-id-to-label ，请使用 Program 地址而非名称，
    /// 多个值以逗号分隔。
    #[serde(default)]
    pub included_dex_program_ids: Vec<String>,
    /// Quote 请求需要排除的 DEX Program ID（模板字段 `jupiter.excluded_dexes`）。
    #[serde(default)]
    pub excluded_dex_program_ids: Vec<String>,
    /// Quote 接口的 `maxAccounts` 参数（模板字段 `jupiter.jup_max_accounts`）。
    /// 官方建议不要设置高于 32。数值越高可以枚举更多路线，但也更容易命中因账户太多导致的无效结果，
    /// 同时会显著拉长搜索时间。
    #[serde(default)]
    pub max_accounts: Option<u32>,
    /// 是否仅寻找两跳直连路线（模板字段 `jupiter.use_direct_route_only`）。
    /// 设为 `true` 时仅会返回 A -> B -> A 形式的路线，报价时间更短且落地更快；
    /// 设为 `false` 则允许 A -> B -> C -> A 等更复杂路径，机会更多但报价会变慢。
    #[serde(default)]
    pub use_direct_route_only: bool,
    /// 是否限制中间代币集合（模板字段 `jupiter.restrict_intermediate_tokens`）。
    /// 只在 `use_direct_route_only = false` 时生效。启用后会把中间代币限制在流动性较稳定的顶级集合，
    /// 降低高滑点风险并缩短报价时延；关闭则可以尝试更多冷门代币，但风险和耗时都会上升。
    #[serde(default = "JupiterRequestParamsConfig::default_restrict_intermediate_tokens")]
    pub restrict_intermediate_tokens: bool,
}

impl Default for JupiterRequestParamsConfig {
    fn default() -> Self {
        Self {
            api_url: None,
            included_dex_program_ids: Vec::new(),
            excluded_dex_program_ids: Vec::new(),
            max_accounts: None,
            use_direct_route_only: false,
            restrict_intermediate_tokens: Self::default_restrict_intermediate_tokens(),
        }
    }
}

impl JupiterRequestParamsConfig {
    const fn default_restrict_intermediate_tokens() -> bool {
        true
    }
}

/// 当使用 `JUPITER_URL=http://0.0.0.0:18080` 或 `JUPITER_URL=http://127.0.0.1:18080` 启动本地 Jupiter 时，
/// 机器人会先根据 `intermedium` 段获取的 mint 列表启动 JupV6 API，并把 `FILTER_MARKETS_WITH_MINTS`
/// 设置为这些 mint。mint 越多，潜在套利路径越丰富，但报价会更慢、资源占用也更高，因此需要在机会数量
/// 和性能之间权衡；如果 `JUPITER_URL` 指向线上实例，则这些设置不会影响远端服务。
#[derive(Debug, Clone, Deserialize)]
pub struct IntermediumConfig {
    /// 额外加载的本地 mint 白名单文件列表（模板字段 `intermedium.load_mints_from_files`）。
    #[serde(default)]
    pub load_mints_from_files: Vec<String>,
    /// 通过 HTTP 拉取的 mint 白名单地址（模板字段 `intermedium.load_mints_from_url`）。
    #[serde(default)]
    pub load_mints_from_url: String,
    /// 白名单可包含的最大 mint 数量（模板字段 `intermedium.max_tokens_limit`）。
    /// 数值越高可以探索更多路径，但也会拖慢报价并占用更多算力。
    #[serde(default = "IntermediumConfig::default_max_tokens_limit")]
    pub max_tokens_limit: u32,
    /// 手动维护的 mint 白名单（模板字段 `intermedium.mints`）。
    #[serde(default)]
    pub mints: Vec<String>,
    /// 需要禁用的 mint 列表（模板字段 `intermedium.disable_mints`），会从白名单中移除。
    #[serde(default)]
    pub disable_mints: Vec<String>,
}

impl Default for IntermediumConfig {
    fn default() -> Self {
        Self {
            load_mints_from_files: Vec::new(),
            load_mints_from_url: String::new(),
            max_tokens_limit: Self::default_max_tokens_limit(),
            mints: Vec::new(),
            disable_mints: Vec::new(),
        }
    }
}

impl IntermediumConfig {
    const fn default_max_tokens_limit() -> u32 {
        20
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct JupiterConfig {
    /// `[jupiter.binary]` 段：Jupiter 二进制下载来源、安装目录等设置，通常保持默认即可。
    #[serde(default)]
    pub binary: JupiterBinaryConfig,
    /// `[jupiter.launch]` 段：运行时 CLI 参数（RPC、监听地址、模式等），可在 `galileo.yaml` 中进一步覆盖。
    #[serde(default)]
    pub launch: JupiterLaunchConfig,
    /// `[jupiter.performance]` 段：线程池与性能相关参数。
    #[serde(default)]
    pub performance: JupiterPerformanceConfig,
    /// `[jupiter.tokens]` 段：代币与 DEX 过滤策略。
    #[serde(default)]
    pub tokens: JupiterTokenConfig,
    /// `[jupiter.process]` 段：进程守护与自动重启策略。
    #[serde(default)]
    pub process: JupiterProcessConfig,
    /// `[jupiter.environment]` 段：为二进制进程附加的环境变量。
    #[serde(default = "JupiterConfig::default_environment")]
    pub environment: BTreeMap<String, String>,
    /// `[jupiter.health_check]` 段：启动后的健康检查配置，确保服务可用。
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
    /// 是否禁止自动更新 Jupiter 二进制。
    #[serde(default)]
    pub disable_update: bool,
    /// Release 资产下载优先顺序。
    #[serde(default)]
    pub download_preference: Vec<String>,
    /// 附加的自定义 CLI 参数，按顺序追加到启动命令末尾。
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            binary: JupiterBinaryConfig::default(),
            launch: JupiterLaunchConfig::default(),
            performance: JupiterPerformanceConfig::default(),
            tokens: JupiterTokenConfig::default(),
            process: JupiterProcessConfig::default(),
            environment: Self::default_environment(),
            health_check: None,
            disable_update: false,
            download_preference: Vec::default(),
            extra_args: Vec::default(),
        }
    }
}

impl JupiterConfig {
    fn default_environment() -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();
        env.insert("RUST_LOG".to_string(), "info".to_string());
        env
    }

    pub fn binary_path(&self) -> PathBuf {
        self.binary.install_dir.join(&self.binary.binary_name)
    }

    pub fn effective_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        let launch = &self.launch;
        if let Some(rpc_url) = &launch.rpc_url {
            args.push("--rpc-url".to_string());
            args.push(rpc_url.clone());
        }

        if !launch.market_cache.is_empty() {
            args.push("--market-cache".to_string());
            args.push(launch.market_cache.clone());
        }

        args.push("--market-mode".to_string());
        args.push(launch.market_mode.as_str().to_string());

        args.push("--host".to_string());
        args.push(launch.host.clone());

        args.push("--port".to_string());
        args.push(launch.port.to_string());

        if let Some(metrics_port) = launch.metrics_port {
            args.push("--metrics-port".to_string());
            args.push(metrics_port.to_string());
        }

        if launch.allow_circular_arbitrage {
            args.push("--allow-circular-arbitrage".to_string());
        }

        if launch.enable_new_dexes {
            args.push("--enable-new-dexes".to_string());
        }

        if launch.expose_quote_and_simulate {
            args.push("--expose-quote-and-simulate".to_string());
        }

        if launch.enable_markets {
            args.push("--enable-markets".to_string());
        }

        if launch.enable_tokens {
            args.push("--enable-tokens".to_string());
        }

        if launch.skip_user_accounts_rpc {
            args.push("--skip-user-accounts-rpc-calls".to_string());
        }

        if let Some(yellowstone) = &launch.yellowstone {
            args.push("--yellowstone-grpc-endpoint".to_string());
            args.push(yellowstone.endpoint.clone());
            if let Some(token) = &yellowstone.x_token {
                if !token.is_empty() {
                    args.push("--yellowstone-grpc-x-token".to_string());
                    args.push(token.clone());
                }
            }
        }

        if !self.tokens.filter_markets_with_mints.is_empty() {
            args.push("--filter-markets-with-mints".to_string());
            args.push(self.tokens.filter_markets_with_mints.join(","));
        }

        if !self.tokens.exclude_dex_program_ids.is_empty() {
            args.push("--exclude-dex-program-ids".to_string());
            args.push(self.tokens.exclude_dex_program_ids.join(","));
        }

        if !self.tokens.include_dex_program_ids.is_empty() {
            args.push("--dex-program-ids".to_string());
            args.push(self.tokens.include_dex_program_ids.join(","));
        }

        let performance = &self.performance;
        let total_threads = std::cmp::max(performance.total_thread_count, 1);
        args.push("--total-thread-count".to_string());
        args.push(total_threads.to_string());

        let web_threads = std::cmp::max(
            std::cmp::min(performance.webserver_thread_count, total_threads),
            1,
        );
        args.push("--webserver-thread-count".to_string());
        args.push(web_threads.to_string());

        let update_threads = std::cmp::max(performance.update_thread_count, 1);
        args.push("--update-thread-count".to_string());
        args.push(update_threads.to_string());

        args.extend(self.extra_args.clone());

        args
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterBinaryConfig {
    /// GitHub 仓库拥有者（默认 `jup-ag`）。
    #[serde(default = "JupiterBinaryConfig::default_repo_owner")]
    pub repo_owner: String,
    /// 仓库名称，默认指向 `jupiter-swap-api`。
    #[serde(default = "JupiterBinaryConfig::default_repo_name")]
    pub repo_name: String,
    /// 安装后的二进制名称。
    #[serde(default = "JupiterBinaryConfig::default_binary_name")]
    pub binary_name: String,
    /// 二进制安装目录，默认写入 `bin/`。
    #[serde(default = "JupiterBinaryConfig::default_install_dir")]
    pub install_dir: PathBuf,
}

impl Default for JupiterBinaryConfig {
    fn default() -> Self {
        Self {
            repo_owner: Self::default_repo_owner(),
            repo_name: Self::default_repo_name(),
            binary_name: Self::default_binary_name(),
            install_dir: Self::default_install_dir(),
        }
    }
}

impl JupiterBinaryConfig {
    fn default_repo_owner() -> String {
        "jup-ag".to_string()
    }

    fn default_repo_name() -> String {
        "jupiter-swap-api".to_string()
    }

    fn default_binary_name() -> String {
        "jupiter-swap-api".to_string()
    }

    fn default_install_dir() -> PathBuf {
        PathBuf::from("bin")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterLaunchConfig {
    /// 主 RPC，用于订阅账户、抓取市场与报价数据，建议选择延迟低且稳定的节点。
    #[serde(default)]
    pub rpc_url: Option<String>,
    /// HTTP 服务监听地址；`0.0.0.0` 可供外部访问，`127.0.0.1` 仅限本机。
    #[serde(default = "JupiterLaunchConfig::default_host")]
    pub host: String,
    /// HTTP 服务端口，默认与官方脚本保持一致。
    #[serde(default = "JupiterLaunchConfig::default_port")]
    pub port: u16,
    /// Prometheus `/metrics` 端点所使用的端口。
    #[serde(default)]
    pub metrics_port: Option<u16>,
    /// 市场快照地址或本地文件路径。若本地启动缓慢，可先 `wget https://cache.jup.ag/markets?v=6 -O mainnet.json`
    /// 并指向该文件以加速冷启动。
    #[serde(default = "JupiterLaunchConfig::default_market_cache")]
    pub market_cache: String,
    /// 市场模式，支持 `remote`（固定快照）、`file`（本地文件）、`europa`（实时推送）。
    #[serde(default)]
    pub market_mode: MarketMode,
    /// 是否禁用本地 Jupiter 二进制（仅使用远端 API）。
    #[serde(default)]
    pub disable_local_binary: bool,
    /// 是否允许输入与输出 mint 相同的环形套利路径。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub allow_circular_arbitrage: bool,
    /// 是否启用最新接入的 DEX，便于第一时间覆盖行情。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_new_dexes: bool,
    /// 是否暴露 `/quote-and-simulate` 调试接口，可一次请求同时获取报价与模拟结果。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub expose_quote_and_simulate: bool,
    /// 是否在启动时加载市场数据。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_markets: bool,
    /// 是否在启动时加载代币数据。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_tokens: bool,
    /// 是否跳过用户账户相关的 RPC 调用，以减少重复拉取。
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub skip_user_accounts_rpc: bool,
    /// Yellowstone Geyser 连接配置（gRPC 入口与可选 token）。
    #[serde(default)]
    pub yellowstone: Option<YellowstoneConfig>,
}

impl Default for JupiterLaunchConfig {
    fn default() -> Self {
        Self {
            rpc_url: None,
            host: Self::default_host(),
            port: Self::default_port(),
            metrics_port: Some(18081),
            market_cache: Self::default_market_cache(),
            market_mode: MarketMode::Remote,
            disable_local_binary: false,
            allow_circular_arbitrage: Self::default_true(),
            enable_new_dexes: Self::default_true(),
            expose_quote_and_simulate: Self::default_true(),
            enable_markets: Self::default_true(),
            enable_tokens: Self::default_true(),
            skip_user_accounts_rpc: Self::default_true(),
            yellowstone: None,
        }
    }
}

impl JupiterLaunchConfig {
    fn default_true() -> bool {
        true
    }

    fn default_host() -> String {
        "0.0.0.0".to_string()
    }

    fn default_port() -> u16 {
        18_080
    }

    fn default_market_cache() -> String {
        "https://cache.jup.ag/markets?v=4".to_string()
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketMode {
    Local,
    Remote,
}

impl Default for MarketMode {
    fn default() -> Self {
        MarketMode::Remote
    }
}

impl MarketMode {
    fn as_str(&self) -> &'static str {
        match self {
            MarketMode::Local => "local",
            MarketMode::Remote => "remote",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YellowstoneConfig {
    /// Yellowstone gRPC 服务地址（允许 `https://host:port` 格式）。
    pub endpoint: String,
    /// Yellowstone 的认证 token，如无需认证可留空。
    #[serde(default)]
    pub x_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterPerformanceConfig {
    /// Jupiter 进程可使用的总线程数。Jupiter 会把线程划分为 webserver/update/router 三类；
    /// router 会使用剩余的核心来搜索每次报价的最优路径，尤其在 `use_direct_route_only = false`
    /// 时负载较高，应为其预留足够 CPU。
    #[serde(default = "JupiterPerformanceConfig::default_total_thread_count")]
    pub total_thread_count: u16,
    /// HTTP/WebServer 线程数，用于处理外部 API 请求。
    #[serde(default = "JupiterPerformanceConfig::default_webserver_thread_count")]
    pub webserver_thread_count: u16,
    /// 市场/缓存更新线程数，直接影响行情刷新速度。
    #[serde(default = "JupiterPerformanceConfig::default_update_thread_count")]
    pub update_thread_count: u16,
}

impl Default for JupiterPerformanceConfig {
    fn default() -> Self {
        Self {
            total_thread_count: Self::default_total_thread_count(),
            webserver_thread_count: Self::default_webserver_thread_count(),
            update_thread_count: Self::default_update_thread_count(),
        }
    }
}

impl JupiterPerformanceConfig {
    fn default_total_thread_count() -> u16 {
        64
    }

    fn default_webserver_thread_count() -> u16 {
        24
    }

    fn default_update_thread_count() -> u16 {
        5
    }
}

/// `[jupiter.tokens]` 段，用于限制 Jupiter 启动时加载的市场与 DEX。
/// 当 `JUPITER_URL` 指向本地且未禁用本地二进制时，`filter_markets_with_mints`
/// 通常会根据 `intermedium` 的 mint 列表生成：mint 越多机会越大，
/// 但报价与 CPU 开销也会相应上升。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct JupiterTokenConfig {
    #[serde(default)]
    pub filter_markets_with_mints: Vec<String>,
    #[serde(default)]
    pub exclude_dex_program_ids: Vec<String>,
    #[serde(default)]
    pub include_dex_program_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterProcessConfig {
    /// 定期自动重启 Jupiter 进程的时间间隔（分钟）；设置为 0 表示不自动重启。
    #[serde(default)]
    pub auto_restart_minutes: u64,
    /// 自动重启失败的最大尝试次数，防止因异常导致无限重启。
    #[serde(default = "JupiterProcessConfig::default_max_restart_attempts")]
    pub max_restart_attempts: u32,
    /// 优雅关闭等待时间（毫秒），退出时用于清理资源。
    #[serde(default = "JupiterProcessConfig::default_graceful_shutdown_timeout_ms")]
    pub graceful_shutdown_timeout_ms: u64,
}

impl Default for JupiterProcessConfig {
    fn default() -> Self {
        Self {
            auto_restart_minutes: 0,
            max_restart_attempts: Self::default_max_restart_attempts(),
            graceful_shutdown_timeout_ms: Self::default_graceful_shutdown_timeout_ms(),
        }
    }
}

impl JupiterProcessConfig {
    fn default_graceful_shutdown_timeout_ms() -> u64 {
        5_000
    }

    fn default_max_restart_attempts() -> u32 {
        3
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    /// 健康检查 URL，例如 `http://127.0.0.1:18080/health`。
    pub url: String,
    /// 健康检查超时（毫秒）。
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub timeout_ms: Option<u64>,
    /// 期望的 HTTP 状态码。
    #[serde(default)]
    pub expected_status: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderConfig {
    /// `lander.yaml` 顶层 `lander` 段，集中维护 tip/落地渠道的所有配置。
    #[serde(default)]
    pub lander: LanderSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LanderSettings {
    /// 是否记录落地器日志。
    #[serde(default)]
    pub enable_log: bool,
    /// 优先费策略，可选 `fixed`、`random` 等。
    #[serde(default = "LanderSettings::default_priority_fee_strategy")]
    pub priority_fee_strategy: String,
    /// 固定优先费（单位：微 lamports）。
    #[serde(default)]
    pub fixed_priority_fee: Option<u64>,
    /// 随机优先费范围（起止数值，单位：微 lamports）。
    #[serde(default)]
    pub random_priority_fee_range: Vec<u64>,
    /// Jito 上链通道配置。
    #[serde(default)]
    pub jito: Option<LanderJitoConfig>,
    /// Staked RPC 上链通道配置，本质上是一个质押节点。
    #[serde(default)]
    pub staked: Option<LanderEndpointConfig>,
    /// Temporal 提供商配置，需要额外发送 tip。
    #[serde(default)]
    pub temporal: Option<LanderEndpointConfig>,
    /// Astralane 提供商配置。
    #[serde(default)]
    pub astralane: Option<LanderEndpointConfig>,
}

impl Default for LanderSettings {
    fn default() -> Self {
        Self {
            enable_log: false,
            priority_fee_strategy: Self::default_priority_fee_strategy(),
            fixed_priority_fee: None,
            random_priority_fee_range: Vec::new(),
            jito: None,
            staked: None,
            temporal: None,
            astralane: None,
        }
    }
}

impl LanderSettings {
    fn default_priority_fee_strategy() -> String {
        "fixed".to_string()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderJitoConfig {
    /// Jito RPC/Tip 服务地址列表。
    #[serde(default)]
    pub endpoints: Vec<String>,
    /// 小费策略列表，支持 `static`、`static-list`、`fixed`、`fixed-list`、`floor`、`floor-list`。
    #[serde(default)]
    pub tip_strategies: Vec<String>,
    /// 静态 Tip（单位：基点）。例如 5000 表示用 50% 的利润作为 Jito 小费，数值越大成功率越高但利润越少。
    #[serde(default)]
    pub static_tip_bp: Option<u64>,
    /// 多个静态 Tip（基点），同上，可对同一机会发送不同 BP 的交易。
    #[serde(default)]
    pub static_tip_bps: Vec<u64>,
    /// 固定 Tip（单位：lamports）。
    #[serde(default)]
    pub fixed_tip: Option<u64>,
    /// 多个固定 Tip（lamports），可针对同一机会发送多笔不同固定额的小费。
    #[serde(default)]
    pub fixed_tips: Vec<u64>,
    /// Floor Tip 级别（如 `75th`）。当前楼层可通过 https://bundles.jito.wtf/api/v1/bundles/tip_floor 获取，
    /// 接口返回的数值以 SOL 为单位，需要乘以 10^9 转成 lamports。
    #[serde(default)]
    pub floor_tip: Option<String>,
    /// 多个 Floor Tip 配置，同上，可一次发送不同楼层的小费。
    #[serde(default)]
    pub floor_tips: Vec<String>,
    /// Floor Tip 的最大限制，防止小费过高。
    #[serde(default)]
    pub max_floor_tip_lamports: Option<u64>,
    /// 针对特定 UUID 的限流配置。
    #[serde(default)]
    pub uuid_config: Vec<LanderJitoUuidConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderJitoUuidConfig {
    /// Jito 提供的 UUID。
    #[serde(default)]
    pub uuid: String,
    /// 针对此 UUID 的速率限制（单位：请求次数）。
    #[serde(default)]
    pub rate_limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderEndpointConfig {
    /// 其他落地服务的可用地址列表。
    #[serde(default)]
    pub endpoints: Vec<String>,
}
