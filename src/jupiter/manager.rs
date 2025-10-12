use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{Duration, SystemTime};

use tokio::process::Command;
use tracing::{info, warn};

use super::error::JupiterError;
use super::process::{shutdown_process, spawn_process};
use super::types::{BinaryInstall, BinaryStatus, JupiterBinaryManager, ProcessHandle, ReleaseInfo};
use super::updater::{
    USER_AGENT, download_and_install, fetch_latest_release, fetch_recent_releases,
    fetch_release_by_tag, read_version_file, select_asset_for_host, write_version_file,
};
use crate::config::{HealthCheckConfig, JupiterConfig, LaunchOverrides};
use crate::metrics::{LatencyMetadata, guard_with_metadata};

impl JupiterBinaryManager {
    pub fn new(
        config: JupiterConfig,
        launch_overrides: LaunchOverrides,
        disable_local_binary: bool,
    ) -> Result<Self, JupiterError> {
        let proxy = config.binary.proxy.clone();
        let mut builder = reqwest::Client::builder().user_agent(USER_AGENT);
        if let Some(proxy_url) = proxy.as_deref() {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }
        let client = builder.build()?;
        if let Some(proxy_url) = proxy {
            info!(
                target: "jupiter",
                %proxy_url,
                "🛡️ 使用下载代理处理 Jupiter Release 请求"
            );
        }

        Ok(Self {
            config,
            launch_overrides,
            disable_local_binary,
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

        self.update(None).await
    }

    pub async fn update(&self, version: Option<&str>) -> Result<BinaryInstall, JupiterError> {
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

        let owner = &self.config.binary.repo_owner;
        let repo = &self.config.binary.repo_name;

        let release = match version {
            Some(tag) => fetch_release_by_tag(&self.client, owner, repo, tag).await?,
            None => fetch_latest_release(&self.client, owner, repo).await?,
        };
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

        let binary_path = self.config.binary_path();
        if binary_exists(&binary_path).await {
            match read_version_file(&self.config.binary.install_dir).await {
                Some(existing_version) => {
                    if existing_version == release.tag_name && !matches!(version, Some(_)) {
                        let install = BinaryInstall {
                            version: existing_version,
                            path: canonicalize_path(binary_path).await,
                            updated_at: SystemTime::now(),
                        };
                        info!(
                            target: "jupiter",
                            version = %install.version,
                            path = %install.path.display(),
                            "local Jupiter binary already up-to-date; skipping download"
                        );
                        {
                            let mut state = self.state.lock().await;
                            state.install = Some(install.clone());
                            state.status = BinaryStatus::Stopped;
                        }
                        guard.finish();
                        return Ok(install);
                    }
                }
                None if !matches!(version, Some(_)) => {
                    let abs_path = canonicalize_path(binary_path).await;
                    info!(
                        target: "jupiter",
                        path = %abs_path.display(),
                        "found existing Jupiter binary without version metadata; skipping automatic update"
                    );
                    if let Err(err) =
                        write_version_file(&self.config.binary.install_dir, &release.tag_name).await
                    {
                        warn!(
                            target: "jupiter",
                            error = %err,
                            "failed to record Jupiter version metadata; future runs may re-check releases"
                        );
                    }
                    let install = BinaryInstall {
                        version: release.tag_name.clone(),
                        path: abs_path,
                        updated_at: SystemTime::now(),
                    };
                    info!(
                        target: "jupiter",
                        version = %install.version,
                        "提示: 如需强制同步 Release，请执行 `galileo jupiter update`"
                    );
                    {
                        let mut state = self.state.lock().await;
                        state.install = Some(install.clone());
                        state.status = BinaryStatus::Stopped;
                    }
                    guard.finish();
                    return Ok(install);
                }
                _ => {}
            }
        }

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

        if self.disable_local_binary {
            info!(
                target: "jupiter",
                "local jupiter binary disabled; skipping binary start"
            );
            self.transition(BinaryStatus::Running).await;
            return Ok(());
        }

        if force_update {
            self.update(None).await?;
        }

        let install = self.ensure_install().await?;

        {
            let state = self.state.lock().await;
            if state.process.is_some() {
                return Err(JupiterError::AlreadyRunning);
            }
        }

        self.transition(BinaryStatus::Starting).await;

        let process = spawn_process(&self.config, &self.launch_overrides, &install).await?;

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
        if self.disable_local_binary {
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

    pub async fn list_releases(&self, limit: usize) -> Result<Vec<ReleaseInfo>, JupiterError> {
        let owner = &self.config.binary.repo_owner;
        let repo = &self.config.binary.repo_name;
        let releases = fetch_recent_releases(&self.client, owner, repo, limit).await?;
        Ok(releases)
    }

    pub async fn installed_version(&self) -> Result<Option<String>, JupiterError> {
        let binary_path = self.config.binary_path();
        if !binary_path.exists() {
            return Ok(None);
        }

        let output = Command::new(&binary_path).arg("--version").output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JupiterError::Schema(format!(
                "执行 {} --version 失败: {}",
                binary_path.display(),
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(Some(format!(
                "{} --version (无输出)",
                binary_path.display()
            )));
        }

        Ok(Some(stdout))
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

async fn canonicalize_path(path: PathBuf) -> PathBuf {
    match tokio::fs::canonicalize(&path).await {
        Ok(abs) => abs,
        Err(_) => path,
    }
}
