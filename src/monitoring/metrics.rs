use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use metrics_exporter_prometheus::PrometheusBuilder;
use once_cell::sync::OnceCell;

static EXPORTER: OnceCell<()> = OnceCell::new();
static PROMETHEUS_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn try_init_prometheus(listen: &str) -> Result<()> {
    EXPORTER
        .get_or_try_init(|| {
            let addr: SocketAddr = listen
                .parse()
                .with_context(|| format!("invalid prometheus listen address: {listen}"))?;
            PrometheusBuilder::new()
                .with_http_listener(addr)
                .install()
                .context("failed to install prometheus exporter")?;
            PROMETHEUS_ENABLED.store(true, Ordering::Relaxed);
            Ok(())
        })
        .map(|_| ())
}

pub fn prometheus_enabled() -> bool {
    PROMETHEUS_ENABLED.load(Ordering::Relaxed)
}
