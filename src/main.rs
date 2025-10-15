use anyhow::Result;

mod api;
mod cli;
mod config;
mod engine;
mod flashloan;
mod jupiter;
mod lander;
mod monitoring;
mod strategy;
mod tools;

#[cfg(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
))]
#[tokio::main(flavor = "current_thread")]
#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [95, 99]))]
async fn main() -> Result<()> {
    cli::run().await
}

#[cfg(not(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
)))]
#[tokio::main]
#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [95, 99]))]
async fn main() -> Result<()> {
    cli::run().await
}
