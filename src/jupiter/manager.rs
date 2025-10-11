use std::path::Path;
use std::process::ExitStatus;
use std::time::Duration;

use tracing::{info, warn};

use super::error::JupiterError;
use super::process::{shutdown_process, spawn_process};
use super::types::{BinaryInstall, BinaryStatus, JupiterBinaryManager, ProcessHandle};
use super::updater::{
    USER_AGENT, download_and_install, fetch_latest_release, select_asset_for_host,
};
use crate::config::{HealthCheckConfig, JupiterConfig};
use crate::metrics::{LatencyMetadata, guard_with_metadata};

impl JupiterBinaryManager {
    pub fn new(config: JupiterConfig) -> Result<Self, JupiterError> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self {
            config,
            client,
            state: Default::default(),
        })
    }

    pub async fn status(&self) -> BinaryStatus {
        let state = self.state.lock().await;
        state.status
    }

    pub async fn ensure_install(&self) -> Result<BinaryInstall, JupiterError> {
        {
            let state = self.state.lock().await;
            if let Some(install) = &state.install {
                if binary_exists(&install.path).await {
                    return Ok(install.clone());
                }
            }
        }

        if self.config.disable_update {
            let path = self.config.binary_path();
            if binary_exists(&path).await {
                return Ok(BinaryInstall {
                    version: "unknown".to_string(),
                    path,
                    updated_at: std::time::SystemTime::now(),
                });
            }
            return Err(JupiterError::BinaryMissing(path));
        }

        self.update().await
    }

    pub async fn update(&self) -> Result<BinaryInstall, JupiterError> {
        {
            let mut state = self.state.lock().await;
            state.status = BinaryStatus::Updating;
        }

        let metadata = LatencyMetadata::new(
            [("stage".to_string(), "update".to_string())]
                .into_iter()
                .collect(),
        );
        let guard = guard_with_metadata("jupiter.update", metadata);

        let release = fetch_latest_release(
            &self.client,
            &self.config.binary.repo_owner,
            &self.config.binary.repo_name,
        )
        .await?;
        info!(target: "jupiter", version = %release.tag_name, asset_count = release.assets.len(), "fetched latest release metadata");
        let asset = select_asset_for_host(&release, &self.config)?;
        info!(
            target: "jupiter",
            asset = %asset.name,
            asset_id = asset.id,
            size_bytes = asset.size,
            content_type = ?asset.content_type,
            "selected release asset"
        );
        let install =
            download_and_install(&self.client, &self.config, &asset, &release.tag_name).await?;

        {
            let mut state = self.state.lock().await;
            state.install = Some(install.clone());
            state.status = BinaryStatus::Stopped;
        }

        guard.finish();
        Ok(install)
    }

    pub async fn start(&self, force_update: bool) -> Result<(), JupiterError> {
        self.ensure_monitor_task().await;

        if self.config.launch.disable_local_binary {
            info!(
                target: "jupiter",
                "local jupiter binary disabled; skipping binary start"
            );
            self.transition(BinaryStatus::Running).await;
            return Ok(());
        }

        if force_update {
            self.update().await?;
        }

        let install = self.ensure_install().await?;

        {
            let state = self.state.lock().await;
            if state.process.is_some() {
                return Err(JupiterError::AlreadyRunning);
            }
        }

        self.transition(BinaryStatus::Starting).await;

        let process = spawn_process(&self.config, &install).await?;

        {
            let mut state = self.state.lock().await;
            state.process = Some(process);
            state.install = Some(install.clone());
            state.status = BinaryStatus::Running;
            state.restart_attempts = 0;
        }

        info!(
            target: "jupiter",
            version = %install.version,
            path = %install.path.display(),
            updated_at = ?install.updated_at,
            "jupiter binary started"
        );

        if let Some(health) = &self.config.health_check {
            if let Err(err) = self.wait_for_health(health).await {
                self.transition(BinaryStatus::Failed).await;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), JupiterError> {
        if self.config.launch.disable_local_binary {
            self.transition(BinaryStatus::Stopped).await;
            return Ok(());
        }

        let process = {
            let mut state = self.state.lock().await;
            match state.process.take() {
                Some(process) => {
                    state.status = BinaryStatus::Stopping;
                    if self.config.process.graceful_shutdown_timeout_ms > 0 {
                        info!(
                            target: "jupiter",
                            timeout_ms = self.config.process.graceful_shutdown_timeout_ms,
                            "initiating graceful shutdown"
                        );
                    }
                    process
                }
                None => return Err(JupiterError::NotRunning),
            }
        };

        shutdown_process(process).await?;

        self.transition(BinaryStatus::Stopped).await;

        Ok(())
    }

    pub async fn restart(&self) -> Result<(), JupiterError> {
        let _ = self.stop().await;
        self.start(false).await
    }

    pub async fn wait_for_health(&self, config: &HealthCheckConfig) -> Result<(), JupiterError> {
        let timeout = Duration::from_millis(config.timeout_ms.unwrap_or(5_000));
        let expected_status = config.expected_status.unwrap_or(200);
        let start = std::time::Instant::now();
        loop {
            let response = self.client.get(&config.url).timeout(timeout).send().await;

            match response {
                Ok(resp) if resp.status().as_u16() == expected_status => {
                    info!(
                        target: "jupiter",
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "health check succeeded"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    warn!(
                        target: "jupiter",
                        status = resp.status().as_u16(),
                        expected_status,
                        "health check status mismatch"
                    );
                }
                Err(err) => {
                    warn!(target: "jupiter", error = %err, "health check request failed");
                }
            }

            if start.elapsed() > timeout {
                self.transition(BinaryStatus::Failed).await;
                return Err(JupiterError::HealthCheck(format!(
                    "timed out after {:?} waiting for {}",
                    timeout, config.url
                )));
            }

            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    async fn transition(&self, status: BinaryStatus) {
        let mut state = self.state.lock().await;
        state.status = status;
    }

    async fn ensure_monitor_task(&self) {
        let should_spawn = {
            let mut state = self.state.lock().await;
            if state.monitor_spawned {
                false
            } else {
                state.monitor_spawned = true;
                true
            }
        };

        if should_spawn {
            self.spawn_monitor_task();
        }
    }

    fn spawn_monitor_task(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            manager.monitor_loop().await;
        });
    }

    async fn monitor_loop(self) {
        struct ExitEvent {
            handle: ProcessHandle,
            status: ExitStatus,
            was_user_stop: bool,
            restart_attempts: u32,
            max_restart_attempts: u32,
        }

        loop {
            let exit_event = {
                let mut state = self.state.lock().await;
                match state.process.as_mut() {
                    Some(handle) => match handle.child.try_wait() {
                        Ok(Some(status)) => {
                            let was_user_stop = matches!(state.status, BinaryStatus::Stopping);
                            let handle = state.process.take();
                            if was_user_stop {
                                state.status = BinaryStatus::Stopped;
                                state.restart_attempts = 0;
                            } else {
                                state.status = BinaryStatus::Failed;
                                state.restart_attempts = state.restart_attempts.saturating_add(1);
                            }
                            handle.map(|handle| ExitEvent {
                                handle,
                                status,
                                was_user_stop,
                                restart_attempts: state.restart_attempts,
                                max_restart_attempts: self.config.process.max_restart_attempts,
                            })
                        }
                        Ok(None) => None,
                        Err(err) => {
                            warn!(
                                target: "jupiter",
                                error = %err,
                                "failed to poll Jupiter process status"
                            );
                            None
                        }
                    },
                    None => None,
                }
            };

            if let Some(mut event) = exit_event {
                if let Some(task) = event.handle.stdout_task.take() {
                    task.abort();
                }
                if let Some(task) = event.handle.stderr_task.take() {
                    task.abort();
                }

                if event.was_user_stop {
                    info!(
                        target: "jupiter",
                        exit_code = event.status.code(),
                        success = event.status.success(),
                        "jupiter process stopped"
                    );
                } else {
                    warn!(
                        target: "jupiter",
                        exit_code = event.status.code(),
                        success = event.status.success(),
                        version = ?event.handle.version,
                        "jupiter process exited unexpectedly"
                    );
                    let auto_restart = self.config.process.auto_restart_minutes;
                    let attempts_limit = event.max_restart_attempts;
                    if auto_restart > 0
                        && (attempts_limit == 0 || event.restart_attempts <= attempts_limit)
                    {
                        let delay_secs = auto_restart.saturating_mul(60);
                        let delay = Duration::from_secs(delay_secs);
                        info!(
                            target: "jupiter",
                            delay_secs = delay.as_secs(),
                            "scheduling automatic restart"
                        );
                        tokio::time::sleep(delay).await;
                        let current_status = self.status().await;
                        if matches!(current_status, BinaryStatus::Failed) {
                            match self.start(false).await {
                                Ok(_) => {
                                    let mut state = self.state.lock().await;
                                    state.restart_attempts = 0;
                                    info!(
                                        target: "jupiter",
                                        "automatic restart succeeded"
                                    );
                                }
                                Err(err) => {
                                    warn!(
                                        target: "jupiter",
                                        error = %err,
                                        "automatic restart failed"
                                    );
                                }
                            }
                        } else {
                            info!(
                                target: "jupiter",
                                ?current_status,
                                "skipping automatic restart because status changed"
                            );
                        }
                    } else if auto_restart > 0 && attempts_limit > 0 {
                        warn!(
                            target: "jupiter",
                            attempts = event.restart_attempts,
                            max_attempts = attempts_limit,
                            "automatic restart attempts exhausted"
                        );
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

async fn binary_exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}
