use once_cell::sync::Lazy;
use rayon::{ThreadPool, ThreadPoolBuilder};

fn configured_thread_count() -> Option<usize> {
    std::env::var("GALILEO_RAYON_THREADS").ok().and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        trimmed.parse::<usize>().ok().filter(|value| *value > 0)
    })
}

fn build_thread_pool() -> ThreadPool {
    let mut builder = ThreadPoolBuilder::new()
        .thread_name(|idx| format!("galileo-rayon-{idx}"))
        .stack_size(2 * 1024 * 1024);

    if let Some(num_threads) = configured_thread_count() {
        builder = builder.num_threads(num_threads);
    }

    builder
        .build()
        .expect("failed to initialize galileo rayon thread pool")
}

static RAYON_POOL: Lazy<ThreadPool> = Lazy::new(build_thread_pool);

/// 返回用于 CPU 密集任务的全局 rayon 线程池。
pub fn rayon_pool() -> &'static ThreadPool {
    &RAYON_POOL
}
