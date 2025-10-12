use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{Duration, SystemTime};

use serde_json::Value;
use tokio::{fs, process::Command};
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

        let binary_path = self.config.binary_path();
        if binary_exists(&binary_path).await {
            let version = read_version_file(&self.config.binary.install_dir)
                .await
                .unwrap_or_else(|| "unknown".to_string());
            let install = BinaryInstall {
                version,
                path: canonicalize_path(binary_path).await,
                updated_at: SystemTime::now(),
            };
            {
                let mut state = self.state.lock().await;
                state.install = Some(install.clone());
                state.status = BinaryStatus::Stopped;
            }
            return Ok(install);
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
        info!(
            target: "jupiter",
            version = %release.tag_name,
            asset_count = release.assets.len(),
            "已获取最新 Release 元数据"
        );
        let asset = select_asset_for_host(&release, &self.config)?;
        info!(
            target: "jupiter",
            asset = %asset.name,
            asset_id = asset.id,
            size_bytes = asset.size,
            content_type = ?asset.content_type,
            "已选择匹配的 Release 资源"
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
                            "本地 Jupiter 二进制已是最新版本，跳过下载"
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
                        "发现已有 Jupiter 二进制但缺少版本信息，跳过自动更新"
                    );
                    if let Err(err) =
                        write_version_file(&self.config.binary.install_dir, &release.tag_name).await
                    {
                        warn!(
                            target: "jupiter",
                            error = %err,
                            "记录 Jupiter 版本信息失败，后续运行将重新检查 Release"
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
            info!(target: "jupiter", "本地 Jupiter 二进制已禁用，跳过启动");
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

        self.prepare_local_market_cache().await?;

        let effective_args = self.config.effective_args(&self.launch_overrides);
        let command_line = format_command(&install.path, &effective_args);
        info!(
            target: "jupiter",
            command = %command_line,
            "准备启动 Jupiter 二进制进程"
        );

        let process = spawn_process(&self.config, &install, &effective_args).await?;
        let pid = process.child.id();

        {
            let mut state = self.state.lock().await;
            state.process = Some(process);
            state.install = Some(install.clone());
            state.status = BinaryStatus::Running;
            state.restart_attempts = 0;
        }

        match pid {
            Some(pid) => info!(
                target: "jupiter",
                version = %install.version,
                path = %install.path.display(),
                updated_at = ?install.updated_at,
                pid,
                command = %command_line,
                "Jupiter 二进制已启动"
            ),
            None => info!(
                target: "jupiter",
                version = %install.version,
                path = %install.path.display(),
                updated_at = ?install.updated_at,
                command = %command_line,
                "Jupiter 二进制已启动（PID 未获取到）"
            ),
        };

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
                            "开始执行优雅关闭"
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

    async fn prepare_local_market_cache(&self) -> Result<(), JupiterError> {
        if !self.config.core.use_local_market_cache {
            return Ok(());
        }

        let Some(local_path) = market_cache_local_path(
            &self.config.core.market_cache,
            &self.config.binary.install_dir,
        ) else {
            return Err(JupiterError::Schema(
                "启用 use_local_market_cache 时，market_cache 必须为本地文件路径".into(),
            ));
        };

        let existed = fs::metadata(&local_path).await.is_ok();
        if existed {
            info!(
                target: "jupiter",
                path = %local_path.display(),
                "检测到已存在的市场缓存文件，将刷新"
            );
        }

        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let download_url = self.config.core.market_cache_download_url.trim();
        if download_url.is_empty() {
            return Err(JupiterError::Schema(
                "启用 use_local_market_cache 时，需提供 market_cache_download_url".into(),
            ));
        }
        let url = download_url.to_string();
        info!(
            target: "jupiter",
            download_url = %url,
            path = %local_path.display(),
            "正在下载市场缓存文件"
        );

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|source| JupiterError::DownloadFailed {
                    url: url.clone(),
                    source,
                })?;
        let response =
            response
                .error_for_status()
                .map_err(|source| JupiterError::DownloadFailed {
                    url: url.clone(),
                    source,
                })?;
        let bytes = response
            .bytes()
            .await
            .map_err(|source| JupiterError::DownloadFailed {
                url: url.clone(),
                source,
            })?;

        let include_filter: HashSet<_> = if self.config.core.exclude_other_dex_program_ids {
            self.launch_overrides
                .include_dex_program_ids
                .iter()
                .map(|s| s.as_str())
                .collect()
        } else {
            HashSet::new()
        };

        let filtered_bytes = if include_filter.is_empty() {
            bytes.to_vec()
        } else {
            let markets: Vec<Value> = serde_json::from_slice(&bytes)
                .map_err(|err| JupiterError::Schema(format!("解析市场缓存失败: {err}")))?;
            let total = markets.len();
            let filtered: Vec<Value> = markets
                .into_iter()
                .filter(|entry| {
                    entry
                        .get("owner")
                        .and_then(|v| v.as_str())
                        .map(|owner| include_filter.contains(owner))
                        .unwrap_or(false)
                })
                .collect();
            let filtered_len = filtered.len();
            if filtered_len == 0 {
                return Err(JupiterError::Schema(
                    "根据 include_dex_program_ids 过滤后市场为空，请检查配置".into(),
                ));
            }
            info!(
                target: "jupiter",
                total_markets = total,
                filtered_markets = filtered_len,
                included_dexes = ?self.launch_overrides.include_dex_program_ids,
                "已按 include_dex_program_ids 过滤市场缓存"
            );
            serde_json::to_vec(&filtered)
                .map_err(|err| JupiterError::Schema(format!("序列化过滤后的市场缓存失败: {err}")))?
        };

        let tmp_path = temporary_market_cache_path(&local_path);
        fs::write(&tmp_path, &filtered_bytes).await?;
        if existed {
            let _ = fs::remove_file(&local_path).await;
        }
        fs::rename(&tmp_path, &local_path).await?;
        info!(
            target: "jupiter",
            path = %local_path.display(),
            size_bytes = filtered_bytes.len(),
            "市场缓存文件下载完成"
        );

        Ok(())
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
        let timeout = Duration::from_secs(config.timeout.unwrap_or(5));
        let expected_status = config.expected_status.unwrap_or(200);
        let start = std::time::Instant::now();
        loop {
            {
                let state = self.state.lock().await;
                if state.process.is_none()
                    && matches!(state.status, BinaryStatus::Failed | BinaryStatus::Stopped)
                {
                    return Err(JupiterError::HealthCheck(
                        "Jupiter 进程在健康检查完成前已退出".into(),
                    ));
                }
            }

            let response = self.client.get(&config.url).timeout(timeout).send().await;

            match response {
                Ok(resp) if resp.status().as_u16() == expected_status => {
                    info!(
                        target: "jupiter",
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "健康检查通过"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    warn!(
                        target: "jupiter",
                        status = resp.status().as_u16(),
                        expected_status,
                        "健康检查状态码不匹配"
                    );
                }
                Err(err) => {
                    warn!(target: "jupiter", error = %err, "健康检查请求失败");
                }
            }

            if start.elapsed() > timeout {
                self.transition(BinaryStatus::Failed).await;
                return Err(JupiterError::HealthCheck(format!(
                    "等待 {} 时超时，耗时 {:?}",
                    config.url, timeout
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
                                "轮询 Jupiter 进程状态失败"
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
                        "Jupiter 进程已停止"
                    );
                } else {
                    warn!(
                        target: "jupiter",
                        exit_code = event.status.code(),
                        success = event.status.success(),
                        version = ?event.handle.version,
                        "Jupiter 进程异常退出"
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
                            "计划在稍后自动重启"
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
                                        "自动重启成功"
                                    );
                                }
                                Err(err) => {
                                    warn!(
                                        target: "jupiter",
                                        error = %err,
                                        "自动重启失败"
                                    );
                                }
                            }
                        } else {
                            info!(
                                target: "jupiter",
                                ?current_status,
                                "检测到状态已变化，跳过自动重启"
                            );
                        }
                    } else if auto_restart > 0 && attempts_limit > 0 {
                        warn!(
                            target: "jupiter",
                            attempts = event.restart_attempts,
                            max_attempts = attempts_limit,
                            "自动重启次数已用尽"
                        );
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

fn market_cache_local_path(path: &str, install_dir: &Path) -> Option<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return None;
    }

    let candidate = Path::new(trimmed);
    if candidate.is_absolute() {
        Some(candidate.to_path_buf())
    } else {
        Some(install_dir.join(candidate))
    }
}

fn temporary_market_cache_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("markets.json");
    let tmp_name = format!("{}.tmp", file_name);
    path.with_file_name(tmp_name)
}

fn format_command(path: &Path, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(path.display().to_string());
    parts.extend(args.iter().cloned());
    parts.join(" ")
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
