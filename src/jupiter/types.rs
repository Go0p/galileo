use std::collections::VecDeque;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::warn;

use crate::config::{JupiterConfig, LaunchOverrides};

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    pub id: u64,
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BinaryInstall {
    pub version: String,
    pub path: PathBuf,
    pub updated_at: SystemTime,
}

pub struct ProcessHandle {
    pub child: Child,
    pub started_at: Instant,
    pub version: Option<String>,
    pub stdout_task: Option<JoinHandle<()>>,
    pub stderr_task: Option<JoinHandle<()>>,
    pub captured_output: Option<Arc<CapturedOutput>>,
}

impl fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessHandle")
            .field("started_at", &self.started_at)
            .field("version", &self.version)
            .field(
                "stdout_task_alive",
                &self.stdout_task.as_ref().map(|h| !h.is_finished()),
            )
            .field(
                "stderr_task_alive",
                &self.stderr_task.as_ref().map(|h| !h.is_finished()),
            )
            .finish()
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                if let Err(err) = self.child.start_kill() {
                    warn!(
                        target: "jupiter",
                        error = %err,
                        "在 ProcessHandle drop 时发送终止信号失败"
                    );
                }
            }
            Err(err) => {
                warn!(
                    target: "jupiter",
                    error = %err,
                    "ProcessHandle drop 时查询子进程状态失败"
                );
            }
        }
    }
}

const CAPTURED_LINES: usize = 50;

#[derive(Default)]
pub struct CapturedOutput {
    stdout: Mutex<VecDeque<String>>,
    stderr: Mutex<VecDeque<String>>,
}

pub struct OutputSnapshot {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl CapturedOutput {
    pub async fn push_stdout(&self, line: String) {
        let mut guard = self.stdout.lock().await;
        if guard.len() >= CAPTURED_LINES {
            guard.pop_front();
        }
        guard.push_back(line);
    }

    pub async fn push_stderr(&self, line: String) {
        let mut guard = self.stderr.lock().await;
        if guard.len() >= CAPTURED_LINES {
            guard.pop_front();
        }
        guard.push_back(line);
    }

    pub async fn snapshot(&self) -> OutputSnapshot {
        let stdout = {
            let guard = self.stdout.lock().await;
            guard.iter().cloned().collect()
        };
        let stderr = {
            let guard = self.stderr.lock().await;
            guard.iter().cloned().collect()
        };
        OutputSnapshot { stdout, stderr }
    }
}

#[derive(Debug, Default)]
pub struct ManagerState {
    pub install: Option<BinaryInstall>,
    pub process: Option<ProcessHandle>,
    pub status: BinaryStatus,
    pub monitor_spawned: bool,
    pub restart_attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryStatus {
    Stopped,
    Updating,
    Starting,
    Running,
    Stopping,
    Failed,
}

impl Default for BinaryStatus {
    fn default() -> Self {
        BinaryStatus::Stopped
    }
}

#[derive(Clone)]
pub struct JupiterBinaryManager {
    pub config: JupiterConfig,
    pub launch_overrides: LaunchOverrides,
    pub disable_local_binary: bool,
    pub show_jupiter_logs: bool,
    pub client: reqwest::Client,
    pub state: Arc<Mutex<ManagerState>>,
}

impl fmt::Debug for JupiterBinaryManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JupiterBinaryManager")
            .field("config", &self.config)
            .field("launch_overrides", &self.launch_overrides)
            .field("disable_local_binary", &self.disable_local_binary)
            .field("show_jupiter_logs", &self.show_jupiter_logs)
            .finish()
    }
}
