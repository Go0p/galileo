use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::JupiterConfig;

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

#[derive(Debug, Default)]
pub struct ManagerState {
    pub install: Option<BinaryInstall>,
    pub process: Option<ProcessHandle>,
    pub status: BinaryStatus,
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
    pub client: reqwest::Client,
    pub state: Arc<Mutex<ManagerState>>,
}

impl fmt::Debug for JupiterBinaryManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JupiterBinaryManager")
            .field("config", &self.config)
            .finish()
    }
}
