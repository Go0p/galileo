use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use flate2::read::GzDecoder;
use futures::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tracing::{info, warn};
use walkdir::WalkDir;
use zip::read::ZipArchive;

use super::error::JupiterError;
use super::types::{BinaryInstall, ReleaseAsset, ReleaseInfo};
use crate::config::JupiterConfig;
use crate::metrics::{LatencyMetadata, guard_with_metadata};

pub(crate) const USER_AGENT: &str = "galileo-bot/0.1";

pub async fn fetch_latest_release(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> Result<ReleaseInfo, JupiterError> {
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest",
        owner = owner,
        repo = repo
    );
    let metadata = LatencyMetadata::new(
        [("stage".to_string(), "fetch_latest_release".to_string())]
            .into_iter()
            .collect(),
    );
    let _guard = guard_with_metadata("github.fetch_latest_release", metadata);

    let response = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(JupiterError::Schema(format!(
            "GitHub returned non-success status {}",
            response.status()
        )));
    }

    let body = response.text().await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;
    let tag_name = json
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| JupiterError::Schema("tag_name missing in release payload".to_string()))?;

    let mut assets = Vec::new();
    if let Some(items) = json.get("assets").and_then(|a| a.as_array()) {
        for item in items {
            if let (Some(id), Some(name), Some(download_url)) = (
                item.get("id").and_then(|v| v.as_u64()),
                item.get("name").and_then(|v| v.as_str()),
                item.get("browser_download_url").and_then(|v| v.as_str()),
            ) {
                let asset = ReleaseAsset {
                    id,
                    name: name.to_string(),
                    download_url: download_url.to_string(),
                    size: item
                        .get("size")
                        .and_then(|v| v.as_u64())
                        .unwrap_or_default(),
                    content_type: item
                        .get("content_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };
                assets.push(asset);
            }
        }
    }

    Ok(ReleaseInfo {
        tag_name: tag_name.to_string(),
        assets,
    })
}

pub fn select_asset_for_host(
    release: &ReleaseInfo,
    config: &JupiterConfig,
) -> Result<ReleaseAsset, JupiterError> {
    let candidates = target_candidates(config);
    for candidate in &candidates {
        if let Some(asset) = release
            .assets
            .iter()
            .find(|asset| asset.name.contains(candidate.as_str()))
        {
            return Ok(asset.clone());
        }
    }

    release
        .assets
        .get(0)
        .cloned()
        .ok_or_else(|| JupiterError::AssetNotFound(candidates.join(",")))
}

pub async fn download_and_install(
    client: &reqwest::Client,
    config: &JupiterConfig,
    asset: &ReleaseAsset,
    version: &str,
) -> Result<BinaryInstall, JupiterError> {
    fs::create_dir_all(&config.binary.install_dir).await?;

    let temp_dir = tempfile::Builder::new()
        .prefix("jupiter-download")
        .tempdir_in(&config.binary.install_dir)?;
    let temp_path = temp_dir.path().join(&asset.name);

    let download_metadata = LatencyMetadata::new(
        [
            ("stage".to_string(), "download_asset".to_string()),
            ("asset".to_string(), asset.name.clone()),
        ]
        .into_iter()
        .collect(),
    );
    let download_guard = guard_with_metadata("github.download_asset", download_metadata);

    let response = client
        .get(&asset.download_url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()
        .await
        .map_err(|source| JupiterError::DownloadFailed {
            url: asset.download_url.clone(),
            source,
        })?;

    if !response.status().is_success() {
        return Err(JupiterError::DownloadStatus {
            url: asset.download_url.clone(),
            status: response.status(),
        });
    }

    let mut file = fs::File::create(&temp_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    download_guard.finish();
    drop(download_guard);

    let binary_path = install_asset(&temp_path, config).await?;
    let updated_at = SystemTime::now();
    info!(
        target: "jupiter",
        version,
        path = %binary_path.display(),
        size_bytes = asset.size,
        content_type = ?asset.content_type,
        updated_at = ?updated_at,
        "installed Jupiter binary"
    );

    Ok(BinaryInstall {
        version: version.to_string(),
        path: binary_path,
        updated_at,
    })
}

async fn install_asset(temp_path: &Path, config: &JupiterConfig) -> Result<PathBuf, JupiterError> {
    let install_dir = config.binary.install_dir.clone();
    let binary_name = config.binary.binary_name.clone();

    if is_tarball(temp_path) {
        extract_tarball(temp_path, &install_dir, &binary_name).await
    } else if is_zip(temp_path) {
        extract_zip(temp_path, &install_dir, &binary_name).await
    } else {
        let target_path = install_dir.join(&binary_name);
        fs::copy(temp_path, &target_path).await?;
        set_executable(&target_path).map_err(|err| {
            JupiterError::ExtractionFailed(format!(
                "failed to set permissions on {}: {err}",
                target_path.display()
            ))
        })?;
        Ok(target_path)
    }
}

async fn extract_tarball(
    archive_path: &Path,
    install_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, JupiterError> {
    let archive_path = archive_path.to_path_buf();
    let install_dir = install_dir.to_path_buf();
    let binary_name = binary_name.to_string();

    task::spawn_blocking(move || -> Result<PathBuf, JupiterError> {
        let file = std::fs::File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive
            .unpack(&install_dir)
            .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
        find_binary(&install_dir, &binary_name).ok_or_else(|| {
            JupiterError::ExtractionFailed(format!(
                "binary {} not found after extracting tarball",
                binary_name
            ))
        })
    })
    .await
    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?
}

async fn extract_zip(
    archive_path: &Path,
    install_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, JupiterError> {
    let archive_path = archive_path.to_path_buf();
    let install_dir = install_dir.to_path_buf();
    let binary_name = binary_name.to_string();

    task::spawn_blocking(move || -> Result<PathBuf, JupiterError> {
        let file = std::fs::File::open(&archive_path)?;
        let mut archive =
            ZipArchive::new(file).map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
            let out_path = match file.enclosed_name() {
                Some(path) => install_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&out_path)
                    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
            }

            {
                let mut outfile = std::fs::File::create(&out_path)
                    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
            }
        }

        find_binary(&install_dir, &binary_name).ok_or_else(|| {
            JupiterError::ExtractionFailed(format!(
                "binary {} not found after extracting zip",
                binary_name
            ))
        })
    })
    .await
    .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?
}

fn set_executable(path: &Path) -> Result<(), JupiterError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)
            .map_err(|err| JupiterError::ExtractionFailed(err.to_string()))?;
    }

    #[cfg(windows)]
    {
        // Windows executes based on extension; ensure .exe
        if path.extension() != Some(OsStr::new("exe")) {
            warn!(
                "binary {} lacks .exe extension on Windows; ensure compatibility",
                path.display()
            );
        }
    }

    Ok(())
}

fn find_binary(root: &Path, binary_name: &str) -> Option<PathBuf> {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(binary_name)
        {
            let path = entry.path().to_path_buf();
            if let Err(err) = set_executable(&path) {
                warn!(
                    "failed to set executable bit on {}: {err:?}",
                    path.display()
                );
            }
            return Some(path);
        }
    }
    None
}

fn is_tarball(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("gz" | "tgz" | "tar")
    )
}

fn is_zip(path: &Path) -> bool {
    matches!(path.extension().and_then(OsStr::to_str), Some("zip"))
}

fn target_candidates(config: &JupiterConfig) -> Vec<String> {
    let mut candidates = Vec::new();
    if !config.download_preference.is_empty() {
        candidates.extend(config.download_preference.iter().cloned());
    }

    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => {
            candidates.push("x86_64-unknown-linux-gnu".to_string());
            candidates.push("linux-x86_64".to_string());
        }
        ("linux", "aarch64") => {
            candidates.push("aarch64-unknown-linux-gnu".to_string());
            candidates.push("linux-aarch64".to_string());
        }
        ("macos", "aarch64") => {
            candidates.push("aarch64-apple-darwin".to_string());
            candidates.push("arm64-macos".to_string());
        }
        ("macos", "x86_64") => {
            candidates.push("x86_64-apple-darwin".to_string());
            candidates.push("x86_64-macos".to_string());
        }
        ("windows", "x86_64") => {
            candidates.push("x86_64-pc-windows-msvc".to_string());
            candidates.push("windows-x86_64".to_string());
        }
        _ => {
            candidates.push(format!(
                "{}-{}",
                std::env::consts::ARCH,
                std::env::consts::OS
            ));
        }
    }

    candidates
}
