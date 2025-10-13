use std::borrow::Cow;
use std::future::Future;

use tracing::Level;

use super::types::{LatencyGuard, LatencyMetadata};

#[allow(dead_code)]
pub fn guard(operation: impl Into<Cow<'static, str>>) -> LatencyGuard {
    LatencyGuard::new(operation, Level::INFO, LatencyMetadata::default())
}

#[allow(dead_code)]
pub fn guard_with_metadata(
    operation: impl Into<Cow<'static, str>>,
    metadata: LatencyMetadata,
) -> LatencyGuard {
    LatencyGuard::new(operation, Level::INFO, metadata)
}

#[allow(dead_code)]
pub fn guard_with_level(
    operation: impl Into<Cow<'static, str>>,
    level: Level,
    metadata: LatencyMetadata,
) -> LatencyGuard {
    LatencyGuard::new(operation, level, metadata)
}

#[allow(dead_code)]
pub async fn measure_future<Fut, T>(
    operation: impl Into<Cow<'static, str>>,
    metadata: LatencyMetadata,
    fut: Fut,
) -> T
where
    Fut: Future<Output = T>,
{
    let guard = LatencyGuard::new(operation, Level::INFO, metadata);
    let output = fut.await;
    guard.finish();
    output
}

#[allow(dead_code)]
pub async fn measure_result<Fut, T, E>(
    operation: impl Into<Cow<'static, str>>,
    metadata: LatencyMetadata,
    fut: Fut,
) -> Result<T, E>
where
    Fut: Future<Output = Result<T, E>>,
{
    let guard = LatencyGuard::new(operation, Level::INFO, metadata);
    let res = fut.await;
    guard.finish();
    res
}
