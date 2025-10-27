use std::time::Duration;

/// 控制 Engine 主循环节奏。
///
/// 新设计改为：每次 tick 结束时告知下一次需要唤醒的最短延迟，
/// Scheduler 负责 sleep 相应的时间，避免所有 base mint 被最小 delay 绑死。
#[derive(Clone, Copy)]
pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self
    }

    pub async fn wait(&self, delay: Duration) {
        if delay.is_zero() {
            return;
        }
        tokio::time::sleep(delay).await;
    }
}
