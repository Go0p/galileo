use std::ffi::OsStr;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
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
pub(crate) const VERSION_FILE_NAME: &str = ".jupiter-version";

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
    fetch_release_by_url(client, &url, "fetch_latest_release").await
}

pub async fn fetch_recent_releases(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    limit: usize,
) -> Result<Vec<ReleaseInfo>, JupiterError> {
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases",
        owner = owner,
        repo = repo
    );

    let metadata = LatencyMetadata::new(
        [("stage".to_string(), "fetch_releases".to_string())]
            .into_iter()
            .collect(),
    );
    let _guard = guard_with_metadata("github.fetch_releases", metadata);

    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(JupiterError::Schema(format!(
            "GitHub 返回非成功状态 {}",
            response.status()
        )));
    }

    let body = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;
    let releases = json
        .as_array()
        .ok_or_else(|| JupiterError::Schema("Release 响应不是数组".to_string()))?;

    let mut infos = Vec::new();
    for release in releases.iter().take(limit) {
        if let Some(info) = parse_release(release)? {
            infos.push(info);
        }
    }

    Ok(infos)
}

pub async fn fetch_release_by_tag(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    tag: &str,
) -> Result<ReleaseInfo, JupiterError> {
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}",
        owner = owner,
        repo = repo,
        tag = tag
    );
    fetch_release_by_url(client, &url, "fetch_release_by_tag").await
}

async fn fetch_release_by_url(
    client: &reqwest::Client,
    url: &str,
    stage: &str,
) -> Result<ReleaseInfo, JupiterError> {
    let metadata = LatencyMetadata::new(
        [("stage".to_string(), stage.to_string())]
            .into_iter()
            .collect(),
    );
    let _guard = guard_with_metadata("github.fetch_release", metadata);

    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(JupiterError::Schema(format!(
            "GitHub 返回非成功状态 {}",
            response.status()
        )));
    }

    let body = response.text().await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;
    parse_release(&json)?
        .ok_or_else(|| JupiterError::Schema("Release 响应缺少必需的 tag_name 字段".to_string()))
}

fn parse_release(value: &serde_json::Value) -> Result<Option<ReleaseInfo>, JupiterError> {
    let tag_name = match value.get("tag_name").and_then(|v| v.as_str()) {
        Some(tag) => tag.to_string(),
        None => return Ok(None),
    };

    let mut assets = Vec::new();
    if let Some(items) = value.get("assets").and_then(|a| a.as_array()) {
        for item in items {
            if let (Some(id), Some(name), Some(download_url)) = (
                item.get("id").and_then(|v| v.as_u64()),
                item.get("name").and_then(|v| v.as_str()),
                item.get("browser_download_url").and_then(|v| v.as_str()),
            ) {
                assets.push(ReleaseAsset {
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
                });
            }
        }
    }

    Ok(Some(ReleaseInfo { tag_name, assets }))
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
    let total_size = if asset.size > 0 {
        asset.size
    } else {
        response.content_length().unwrap_or_default()
    };
    let has_known_size = total_size > 0;
    let mut progress_bar = create_download_progress_bar(&asset.name, total_size);
    if let Some(pb) = progress_bar.as_ref() {
        if !has_known_size {
            pb.set_message(format!("下载 {} {}", asset.name, HumanBytes(0)));
        }
    } else {
        info!(
            target: "jupiter",
            asset = %asset.name,
            total_bytes = total_size,
            "开始下载 Release 资源"
        );
    }
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_logged_bytes: u64 = 0;
    let mut last_logged_at = Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        if let Some(pb) = progress_bar.as_ref() {
            pb.inc(chunk.len() as u64);
            if !has_known_size {
                pb.set_message(format!("下载 {} {}", asset.name, HumanBytes(downloaded)));
            }
        } else {
            let should_log = downloaded == total_size
                || downloaded.saturating_sub(last_logged_bytes) >= 2 * 1024 * 1024
                || last_logged_at.elapsed() >= Duration::from_secs(5);
            if should_log {
                if has_known_size {
                    let percent = if total_size > 0 {
                        (downloaded as f64 / total_size as f64 * 100.0).min(100.0)
                    } else {
                        0.0
                    };
                    info!(
                        target: "jupiter",
                        asset = %asset.name,
                        downloaded_bytes = downloaded,
                        total_bytes = total_size,
                        progress_pct = format_args!("{percent:.1}"),
                        "正在下载 Release 资源"
                    );
                } else {
                    info!(
                        target: "jupiter",
                        asset = %asset.name,
                        downloaded_bytes = downloaded,
                        "正在下载 Release 资源"
                    );
                }
                last_logged_bytes = downloaded;
                last_logged_at = Instant::now();
            }
        }
    }
    file.flush().await?;
    if progress_bar.is_none() {
        info!(
            target: "jupiter",
            asset = %asset.name,
            downloaded_bytes = downloaded,
            "下载完成，准备安装"
        );
    }
    if let Some(pb) = progress_bar.take() {
        if has_known_size {
            pb.finish_with_message(format!(
                "{} 下载完成 ({}/{})，开始解压…",
                asset.name,
                HumanBytes(downloaded),
                HumanBytes(total_size)
            ));
        } else {
            pb.finish_with_message(format!(
                "{} 下载完成 ({})，开始解压…",
                asset.name,
                HumanBytes(downloaded)
            ));
        }
    }
    download_guard.finish();
    drop(download_guard);

    let binary_path = install_asset(&temp_path, config).await?;
    let binary_path = canonicalize_path(binary_path).await;
    let updated_at = SystemTime::now();
    let versioned_path = create_versioned_copy(&binary_path, version, config).await?;
    write_version_file(&config.binary.install_dir, version).await?;
    info!(
        target: "jupiter",
        version,
        path = %binary_path.display(),
        versioned_path = %versioned_path.display(),
        size_bytes = asset.size,
        content_type = ?asset.content_type,
        updated_at = ?updated_at,
        "Jupiter 二进制安装完成"
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
                "设置 {} 权限失败: {err}",
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
            JupiterError::ExtractionFailed(format!("解压 tar 包后未找到二进制 {}", binary_name))
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
            JupiterError::ExtractionFailed(format!("解压 zip 后未找到二进制 {}", binary_name))
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
                "二进制 {} 在 Windows 上缺少 .exe 扩展名，请确认兼容性",
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
                warn!("为 {} 设置可执行权限失败: {err:?}", path.display());
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

fn target_candidates(_config: &JupiterConfig) -> Vec<String> {
    let mut candidates = Vec::new();
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

async fn create_versioned_copy(
    binary_path: &Path,
    version: &str,
    config: &JupiterConfig,
) -> Result<PathBuf, JupiterError> {
    let versioned_name = format!("{}-{}", config.binary.binary_name, version);
    let releases_dir = config.binary.install_dir.join("releases");
    fs::create_dir_all(&releases_dir)
        .await
        .map_err(JupiterError::Io)?;
    let versioned_path = releases_dir.join(&versioned_name);

    // 将历史遗留的版本化二进制迁移至 releases 目录，确保 bin/ 根目录保持整洁
    relocate_legacy_versioned_binaries(config, &releases_dir, &versioned_name).await?;

    if fs::metadata(&versioned_path).await.is_ok() {
        fs::remove_file(&versioned_path)
            .await
            .map_err(JupiterError::Io)?;
    }

    // 清理历史遗留的根目录副本，防止在 bin/ 下留下重复版本文件
    let legacy_path = config.binary.install_dir.join(&versioned_name);
    if legacy_path != versioned_path && fs::metadata(&legacy_path).await.is_ok() {
        if let Err(err) = fs::remove_file(&legacy_path).await {
            warn!(
                target: "jupiter",
                path = %legacy_path.display(),
                error = %err,
                "移除旧版本化二进制失败"
            );
        }
    }

    fs::copy(binary_path, &versioned_path)
        .await
        .map_err(JupiterError::Io)?;
    set_executable(&versioned_path)?;
    Ok(versioned_path)
}

async fn relocate_legacy_versioned_binaries(
    config: &JupiterConfig,
    releases_dir: &Path,
    current_versioned_name: &str,
) -> Result<(), JupiterError> {
    let mut entries = match fs::read_dir(&config.binary.install_dir).await {
        Ok(entries) => entries,
        Err(err) => return Err(JupiterError::Io(err)),
    };
    let prefix = format!("{}-", config.binary.binary_name);

    while let Some(entry) = entries.next_entry().await.map_err(JupiterError::Io)? {
        let path = entry.path();
        if path == releases_dir {
            continue;
        }
        let file_type = entry.file_type().await.map_err(JupiterError::Io)?;
        if !file_type.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with(&prefix) {
            continue;
        }

        if file_name == current_versioned_name {
            // 交由后续 copy 移除 legacy_path 即可
            continue;
        }

        let target = releases_dir.join(file_name);
        if target == path {
            continue;
        }

        if let Err(err) = fs::rename(&path, &target).await {
            warn!(
                target: "jupiter",
                from = %path.display(),
                to = %target.display(),
                error = %err,
                "迁移旧版 Jupiter 二进制至 releases 目录失败，尝试回退为复制"
            );
            fs::copy(&path, &target).await.map_err(JupiterError::Io)?;
            fs::remove_file(&path).await.map_err(JupiterError::Io)?;
        }
    }

    Ok(())
}

pub(crate) async fn write_version_file(dir: &Path, version: &str) -> Result<(), JupiterError> {
    fs::write(dir.join(VERSION_FILE_NAME), version)
        .await
        .map_err(JupiterError::Io)
}

pub(crate) async fn read_version_file(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join(VERSION_FILE_NAME))
        .await
        .map(|content| content.trim().to_string())
        .ok()
}

async fn canonicalize_path(path: PathBuf) -> PathBuf {
    match fs::canonicalize(&path).await {
        Ok(abs) => abs,
        Err(_) => path,
    }
}

fn create_download_progress_bar(name: &str, total_size: u64) -> Option<ProgressBar> {
    if !std::io::stderr().is_terminal() {
        return None;
    }

    if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        let style = ProgressStyle::with_template(
            "{spinner:.green} {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar());
        pb.set_style(style);
        pb.set_message(format!("下载 {name}"));
        Some(pb)
    } else {
        let pb = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());
        pb.set_style(style);
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message(format!("下载 {name}"));
        Some(pb)
    }
}
