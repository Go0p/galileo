use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tracing::Level;

#[derive(Debug, Clone)]
pub struct LatencyMetadata {
    fields: Arc<BTreeMap<String, String>>,
}

impl LatencyMetadata {
    pub fn new(fields: BTreeMap<String, String>) -> Self {
        Self {
            fields: Arc::new(fields),
        }
    }

    pub fn empty() -> Self {
        Self {
            fields: Arc::new(BTreeMap::new()),
        }
    }

    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.fields
    }
}

impl Default for LatencyMetadata {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug)]
pub struct LatencyGuard {
    operation: Cow<'static, str>,
    start: Instant,
    level: Level,
    metadata: LatencyMetadata,
    ended: AtomicBool,
}

impl LatencyGuard {
    pub fn new(
        operation: impl Into<Cow<'static, str>>,
        level: Level,
        metadata: LatencyMetadata,
    ) -> Self {
        Self {
            operation: operation.into(),
            start: Instant::now(),
            level,
            metadata,
            ended: AtomicBool::new(false),
        }
    }

    #[allow(dead_code)]
    pub fn end_with_metadata(mut self, metadata: LatencyMetadata) -> Duration {
        self.metadata = metadata;
        self.finish()
    }

    pub fn finish(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if !self.ended.swap(true, Ordering::SeqCst) {
            log_latency(self.level, &self.operation, elapsed, &self.metadata);
        }
        elapsed
    }

    #[allow(dead_code)]
    pub fn cancel(self) {}
}

impl Drop for LatencyGuard {
    fn drop(&mut self) {
        if !self.ended.load(Ordering::SeqCst) {
            let elapsed = self.start.elapsed();
            if !self.ended.swap(true, Ordering::SeqCst) {
                log_latency(self.level, &self.operation, elapsed, &self.metadata);
            }
        }
    }
}

fn log_latency(level: Level, operation: &str, elapsed: Duration, metadata: &LatencyMetadata) {
    let elapsed_us = elapsed.as_micros();
    let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
    let elapsed_ms_display = format!("{elapsed_ms:.3}");
    let metadata_summary = if metadata.fields().is_empty() {
        None
    } else {
        Some(
            metadata
                .fields()
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(" "),
        )
    };

    macro_rules! log_event {
        ($macro:ident) => {
            if let Some(ref metadata) = metadata_summary {
                tracing::$macro!(
                    target: "latency",
                    %operation,
                    elapsed_us,
                    elapsed_ms = %elapsed_ms_display,
                    metadata = %metadata,
                    "耗时统计"
                );
            } else {
                tracing::$macro!(
                    target: "latency",
                    %operation,
                    elapsed_us,
                    elapsed_ms = %elapsed_ms_display,
                    "耗时统计"
                );
            }
        };
    }

    match level {
        Level::ERROR => log_event!(error),
        Level::WARN => log_event!(warn),
        Level::INFO => log_event!(info),
        Level::DEBUG => log_event!(debug),
        Level::TRACE => log_event!(trace),
    }
}
