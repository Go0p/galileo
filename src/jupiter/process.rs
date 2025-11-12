use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{info, warn};

use super::error::JupiterError;
use super::types::{BinaryInstall, CapturedOutput, ProcessHandle};
use crate::config::JupiterConfig;

pub async fn spawn_process(
    config: &JupiterConfig,
    install: &BinaryInstall,
    args: &[String],
    show_output: bool,
) -> Result<ProcessHandle, JupiterError> {
    let mut command = Command::new(&install.path);
    command.current_dir(&config.binary.install_dir).args(args);

    if show_output {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    }

    if !config.environment.contains_key("RUST_LOG") {
        command.env("RUST_LOG", "info");
    }

    if !config.environment.contains_key("NO_COLOR") && !show_output {
        command.env("NO_COLOR", "1");
    }

    for (key, value) in &config.environment {
        command.env(key, value);
    }

    let mut child = command.spawn()?;
    let mut captured_output = None;
    let mut stdout_task = None;
    let mut stderr_task = None;

    if !show_output {
        if let Some(stdout) = child.stdout.take() {
            let output = captured_output
                .get_or_insert_with(|| Arc::new(CapturedOutput::default()))
                .clone();
            stdout_task = Some(tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    output.push_stdout(line).await;
                }
            }));
        }
        if let Some(stderr) = child.stderr.take() {
            let output = captured_output
                .get_or_insert_with(|| Arc::new(CapturedOutput::default()))
                .clone();
            stderr_task = Some(tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    output.push_stderr(line).await;
                }
            }));
        }
    }

    Ok(ProcessHandle {
        child,
        started_at: Instant::now(),
        version: Some(install.version.clone()),
        stdout_task,
        stderr_task,
        captured_output,
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
                "Jupiter 进程已提前退出"
            );
            return Ok(());
        }
        Ok(None) => {
            if let Err(err) = handle.child.start_kill() {
                warn!(target: "jupiter", error = %err, "发送终止信号失败");
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
                "Jupiter 进程已退出"
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
