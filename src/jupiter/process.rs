use std::process::Stdio;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};

use super::error::JupiterError;
use super::types::{BinaryInstall, ProcessHandle};
use crate::config::JupiterConfig;

pub async fn spawn_process(
    config: &JupiterConfig,
    install: &BinaryInstall,
) -> Result<ProcessHandle, JupiterError> {
    let effective_args = config.effective_args();

    let mut command = Command::new(&install.path);
    command
        .current_dir(&config.binary.install_dir)
        .args(&effective_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if !config.environment.contains_key("RUST_LOG") {
        command.env("RUST_LOG", "info");
    }

    for (key, value) in &config.environment {
        command.env(key, value);
    }

    let mut child = command.spawn()?;

    let stdout_task = child
        .stdout
        .take()
        .map(|stdout| spawn_output_task(stdout, "stdout"));

    let stderr_task = child
        .stderr
        .take()
        .map(|stderr| spawn_output_task(stderr, "stderr"));

    Ok(ProcessHandle {
        child,
        started_at: Instant::now(),
        version: Some(install.version.clone()),
        stdout_task,
        stderr_task,
    })
}

pub async fn shutdown_process(mut handle: ProcessHandle) -> Result<(), JupiterError> {
    let stdout_task = handle.stdout_task.take();
    let stderr_task = handle.stderr_task.take();

    match handle.child.try_wait() {
        Ok(Some(status)) => {
            info!(
                target: "jupiter",
                code = status.code(),
                success = status.success(),
                "process already exited"
            );
            return Ok(());
        }
        Ok(None) => {
            if let Err(err) = handle.child.start_kill() {
                warn!(target: "jupiter", error = %err, "failed to send kill signal");
            }
        }
        Err(err) => return Err(err.into()),
    }

    match handle.child.wait().await {
        Ok(status) => {
            info!(
                target: "jupiter",
                code = status.code(),
                success = status.success(),
                "process exited"
            );
            if let Some(task) = stdout_task {
                task.abort();
            }
            if let Some(task) = stderr_task {
                task.abort();
            }
            Ok(())
        }
        Err(err) => Err(JupiterError::Io(err)),
    }
}

fn spawn_output_task<R>(reader: R, stream: &'static str) -> tokio::task::JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    info!(target: "jupiter::process", stream, message = %line);
                }
                Ok(None) => break,
                Err(err) => {
                    error!(target: "jupiter::process", stream, error = %err, "failed to read line");
                    break;
                }
            }
        }
    })
}
