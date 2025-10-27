use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use anyhow::{Result, anyhow};
use clap::Parser;
use tracing::{debug, info, warn};

mod api;
mod cache;
mod cli;
mod config;
mod dexes;
mod engine;
mod flashloan;
mod intermedium;
mod jupiter;
mod lander;
mod monitoring;
mod multi_leg;
mod pure_blind;
mod rpc;
mod strategy;
mod txs;

use crate::cli::args::Cli;
use crate::cli::context::{init_tracing, load_configuration};
use crate::config::{AppConfig, CpuAffinityConfig, loader::ConfigError as LoaderConfigError};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
async fn async_entry(cli: Cli, config: AppConfig) -> Result<()> {
    crate::cli::run(cli, config).await
}

#[derive(Debug)]
struct RuntimeOptions {
    plan: Option<Arc<AffinityPlan>>,
    worker_threads: Option<usize>,
    max_blocking_threads: Option<usize>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            plan: None,
            worker_threads: None,
            max_blocking_threads: None,
        }
    }
}

#[derive(Debug)]
struct AffinityPlan {
    cores: Vec<usize>,
    cursor: AtomicUsize,
    wrapped_notified: AtomicBool,
}

impl AffinityPlan {
    fn new(cores: Vec<usize>) -> Self {
        Self {
            cores,
            cursor: AtomicUsize::new(0),
            wrapped_notified: AtomicBool::new(false),
        }
    }

    fn core_count(&self) -> usize {
        self.cores.len()
    }

    fn cores(&self) -> &[usize] {
        &self.cores
    }

    fn bind_main_thread(&self) {
        self.bind("main");
    }

    fn bind_worker_thread(&self) {
        self.bind("worker");
    }

    fn bind(&self, role: &str) {
        if let Some(core) = self.next_core() {
            let core_id = core_affinity::CoreId { id: core };
            if core_affinity::set_for_current(core_id) {
                debug!(
                    target: "runtime::affinity",
                    role,
                    core_id = core,
                    "线程已绑定到指定 CPU 核"
                );
            } else {
                warn!(
                    target: "runtime::affinity",
                    role,
                    core_id = core,
                    "尝试设置线程 CPU 亲和性失败，该线程可能仍受调度器分配"
                );
            }
        } else {
            warn!(
                target: "runtime::affinity",
                role,
                "未找到可用的核心分配，CPU 绑定已跳过"
            );
        }
    }

    fn next_core(&self) -> Option<usize> {
        if self.cores.is_empty() {
            return None;
        }
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed);
        let len = self.cores.len();
        if len == 0 {
            return None;
        }
        if idx >= len && !self.wrapped_notified.swap(true, Ordering::Relaxed) {
            warn!(
                target: "runtime::affinity",
                len,
                "Tokio 线程数量超过可用核心数，将循环复用核心"
            );
        }
        Some(self.cores[idx % len])
    }
}

fn bootstrap() -> Result<(Cli, AppConfig, RuntimeOptions)> {
    let cli = Cli::parse();
    let config = match load_configuration(cli.config.clone()) {
        Ok(config) => config,
        Err(LoaderConfigError::WalletEncrypted { encrypted }) => {
            println!(
                "🔐 已将私钥加密写入 `{}`，配置项已清空。请确认已提交或备份后，再次启动 Galileo。",
                encrypted.display()
            );
            std::process::exit(0);
        }
        Err(err) => return Err(err.into()),
    };
    init_tracing(&config.galileo.global.logging)?;

    let runtime_opts = prepare_runtime_options(&config.galileo.bot.cpu_affinity)?;

    Ok((cli, config, runtime_opts))
}

fn prepare_runtime_options(config: &CpuAffinityConfig) -> Result<RuntimeOptions> {
    if !config.enable {
        return Ok(RuntimeOptions::default());
    }

    if config.worker_cores.is_empty() {
        return Err(anyhow!(
            "bot.cpu_affinity.enable = true 时必须至少指定一个 worker_cores"
        ));
    }

    let available_core_ids = core_affinity::get_core_ids();
    if available_core_ids.is_none() {
        let message = "当前系统不支持查询 CPU 亲和性，无法启用 bot CPU 绑定";
        if config.strict {
            return Err(anyhow!(message));
        }
        warn!(
            target: "runtime::affinity",
            "{message}",
            message = message
        );
        return Ok(RuntimeOptions::default());
    }

    let available: HashSet<usize> = available_core_ids
        .unwrap()
        .into_iter()
        .map(|core| core.id)
        .collect();

    let mut selected = Vec::new();
    let mut skipped = Vec::new();
    let mut seen = HashSet::new();
    for &core in &config.worker_cores {
        if !seen.insert(core) {
            continue;
        }
        if available.contains(&core) {
            selected.push(core);
        } else {
            skipped.push(core);
        }
    }

    if selected.is_empty() {
        if config.strict {
            return Err(anyhow!(
                "bot.cpu_affinity.worker_cores 中没有任何有效核心，且 strict = true"
            ));
        }
        warn!(
            target: "runtime::affinity",
            cores = ?config.worker_cores,
            "未找到可用核心，CPU 绑定已禁用"
        );
        return Ok(RuntimeOptions::default());
    }

    if !skipped.is_empty() {
        warn!(
            target: "runtime::affinity",
            skipped = ?skipped,
            "以下核心在当前机器上不可用，将被忽略"
        );
    }

    let plan = Arc::new(AffinityPlan::new(selected));
    info!(
        target: "runtime::affinity",
        cores = ?plan.cores(),
        "启用 bot CPU 亲和性"
    );

    let max_blocking_threads = config.max_blocking_threads.filter(|value| {
        if *value == 0 {
            warn!(
                target: "runtime::affinity",
                "max_blocking_threads 配置为 0 无效，将忽略该值"
            );
            false
        } else {
            true
        }
    });

    Ok(RuntimeOptions {
        worker_threads: Some(plan.core_count()),
        plan: Some(plan),
        max_blocking_threads,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_affinity_requires_core_list_when_enabled() {
        let mut cfg = CpuAffinityConfig::default();
        cfg.enable = true;

        let err = prepare_runtime_options(&cfg).unwrap_err();
        assert!(
            err.to_string().contains("worker_cores"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn cpu_affinity_accepts_available_core() {
        let mut cfg = CpuAffinityConfig::default();
        cfg.enable = true;

        if let Some(cores) = core_affinity::get_core_ids() {
            let first = cores
                .first()
                .expect("system exposes at least one CPU core")
                .id;
            cfg.worker_cores = vec![first];
            let opts = prepare_runtime_options(&cfg).expect("valid config should succeed");
            assert!(opts.plan.is_some());
            assert_eq!(opts.worker_threads, Some(1));
        } else {
            // 平台不支持 CPU 亲和性，测试直接跳过。
            return;
        }
    }
}

#[cfg(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
))]
fn main() -> Result<()> {
    let (cli, config, opts) = bootstrap()?;

    if let Some(plan) = opts.plan.as_ref() {
        plan.bind_main_thread();
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async_entry(cli, config))
}

#[cfg(not(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
)))]
fn main() -> Result<()> {
    let (cli, config, opts) = bootstrap()?;

    if let Some(plan) = opts.plan.as_ref() {
        plan.bind_main_thread();
    }

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(worker_threads) = opts.worker_threads {
        builder.worker_threads(worker_threads);
    }

    if let Some(limit) = opts.max_blocking_threads {
        builder.max_blocking_threads(limit);
    }

    if let Some(plan) = opts.plan.clone() {
        builder.on_thread_start(move || {
            plan.bind_worker_thread();
        });
    }

    let runtime = builder.enable_all().build()?;
    runtime.block_on(async_entry(cli, config))
}
