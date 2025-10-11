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

    pub fn cancel(self) {
        // Intentionally does nothing; dropping without logging.
    }
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
    match level {
        Level::ERROR => {
            tracing::error!(target: "latency", %operation, elapsed_us, metadata = ?metadata.fields())
        }
        Level::WARN => {
            tracing::warn!(target: "latency", %operation, elapsed_us, metadata = ?metadata.fields())
        }
        Level::INFO => {
            tracing::info!(target: "latency", %operation, elapsed_us, metadata = ?metadata.fields())
        }
        Level::DEBUG => {
            tracing::debug!(target: "latency", %operation, elapsed_us, metadata = ?metadata.fields())
        }
        Level::TRACE => {
            tracing::trace!(target: "latency", %operation, elapsed_us, metadata = ?metadata.fields())
        }
    }
}
