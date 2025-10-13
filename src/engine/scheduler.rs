use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Scheduler {
    delay: Duration,
}

impl Scheduler {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    pub async fn wait(&self) {
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
    }
}
