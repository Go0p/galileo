use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::fs;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{debug, info, warn};

use crate::config::PureBlindCacheConfig;
use crate::strategy::pure_blind::observer::catalog::PoolCatalog;
use crate::strategy::pure_blind::observer::routes::RouteCatalog;
use crate::strategy::pure_blind::observer::snapshot::{PoolSnapshot, RouteSnapshot};

const DEFAULT_CACHE_DIR: &str = "monitoring/pure_blind_cache";
const POOLS_FILE: &str = "pools.json";
const ROUTES_FILE: &str = "routes.json";

#[derive(Clone)]
pub struct PureBlindCacheManager {
    enabled: bool,
    dir: PathBuf,
    pools_path: PathBuf,
    routes_path: PathBuf,
    snapshot_interval: Duration,
    snapshot_ttl: Option<Duration>,
}

impl PureBlindCacheManager {
    pub fn new(config: &PureBlindCacheConfig) -> Self {
        let dir = config
            .cache_dir
            .as_ref()
            .map(|value| PathBuf::from(value.trim()))
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_DIR));

        let snapshot_interval = Duration::from_secs(config.snapshot_interval_secs.max(1));
        let snapshot_ttl = if config.snapshot_ttl_secs == 0 {
            None
        } else {
            Some(Duration::from_secs(config.snapshot_ttl_secs))
        };

        Self {
            enabled: config.enable_persistence,
            pools_path: dir.join(POOLS_FILE),
            routes_path: dir.join(ROUTES_FILE),
            dir,
            snapshot_interval,
            snapshot_ttl,
        }
    }

    pub async fn restore(
        &self,
        pool_catalog: &PoolCatalog,
        route_catalog: &RouteCatalog,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.ensure_dir().await?;
        let now = Self::now_secs();

        if let Some(snapshot) = self.read_snapshot::<PoolSnapshot>(&self.pools_path).await? {
            if self.is_snapshot_expired(snapshot.generated_at, now) {
                warn!(
                    target: "pure_blind::cache",
                    path = %self.pools_path.display(),
                    "池子快照已过期，忽略"
                );
            } else {
                let count = snapshot.entries.len();
                if count > 0 {
                    pool_catalog.ingest_snapshot(snapshot);
                    info!(
                        target: "pure_blind::cache",
                        path = %self.pools_path.display(),
                        count,
                        "已恢复池子画像快照"
                    );
                } else {
                    debug!(
                        target: "pure_blind::cache",
                        path = %self.pools_path.display(),
                        "池子快照为空，跳过恢复"
                    );
                }
            }
        }

        if let Some(snapshot) = self
            .read_snapshot::<RouteSnapshot>(&self.routes_path)
            .await?
        {
            if self.is_snapshot_expired(snapshot.generated_at, now) {
                warn!(
                    target: "pure_blind::cache",
                    path = %self.routes_path.display(),
                    "路线快照已过期，忽略"
                );
            } else {
                let count = snapshot.entries.len();
                if count > 0 {
                    route_catalog.ingest_snapshot(snapshot);
                    info!(
                        target: "pure_blind::cache",
                        path = %self.routes_path.display(),
                        count,
                        "已恢复路线快照"
                    );
                } else {
                    debug!(
                        target: "pure_blind::cache",
                        path = %self.routes_path.display(),
                        "路线快照为空，跳过恢复"
                    );
                }
            }
        }

        Ok(())
    }

    pub fn spawn(
        &self,
        pool_catalog: Arc<PoolCatalog>,
        route_catalog: Arc<RouteCatalog>,
    ) -> Option<JoinHandle<()>> {
        if !self.enabled {
            return None;
        }

        let manager = self.clone();
        Some(tokio::spawn(async move {
            manager.run_loop(pool_catalog, route_catalog).await;
        }))
    }

    async fn run_loop(self, pool_catalog: Arc<PoolCatalog>, route_catalog: Arc<RouteCatalog>) {
        if let Err(err) = self.ensure_dir().await {
            warn!(
                target: "pure_blind::cache",
                error = %err,
                "创建缓存目录失败，快照写入被禁用"
            );
            return;
        }

        if let Err(err) = self
            .write_snapshots(pool_catalog.as_ref(), route_catalog.as_ref())
            .await
        {
            warn!(
                target: "pure_blind::cache",
                error = %err,
                "初始快照写入失败"
            );
        }

        let mut ticker = interval(self.snapshot_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            if let Err(err) = self
                .write_snapshots(pool_catalog.as_ref(), route_catalog.as_ref())
                .await
            {
                warn!(
                    target: "pure_blind::cache",
                    error = %err,
                    "写入快照失败"
                );
            }
        }
    }

    async fn write_snapshots(
        &self,
        pool_catalog: &PoolCatalog,
        route_catalog: &RouteCatalog,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let now = Self::now_secs();
        let pool_snapshot = pool_catalog.export_snapshot(now);
        let route_snapshot = route_catalog.export_snapshot(now);

        self.write_snapshot(&self.pools_path, &pool_snapshot)
            .await
            .with_context(|| {
                format!("写入池子快照失败: {}", self.pools_path.as_path().display())
            })?;
        self.write_snapshot(&self.routes_path, &route_snapshot)
            .await
            .with_context(|| {
                format!("写入路线快照失败: {}", self.routes_path.as_path().display())
            })?;

        debug!(
            target: "pure_blind::cache",
            pools = pool_snapshot.entries.len(),
            routes = route_snapshot.entries.len(),
            "快照已写入磁盘"
        );
        Ok(())
    }

    async fn ensure_dir(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        fs::create_dir_all(&self.dir)
            .await
            .with_context(|| format!("创建缓存目录失败: {}", self.dir.display()))
    }

    async fn read_snapshot<T>(&self, path: &Path) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match fs::read(path).await {
            Ok(bytes) => {
                let snapshot = serde_json::from_slice(&bytes)
                    .with_context(|| format!("解析快照文件失败: {}", path.display()))?;
                Ok(Some(snapshot))
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err).with_context(|| format!("读取快照文件失败: {}", path.display())),
        }
    }

    async fn write_snapshot<T>(&self, path: &Path, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec_pretty(value)
            .with_context(|| format!("序列化快照数据失败: {}", path.display()))?;
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, data)
            .await
            .with_context(|| format!("写入临时快照文件失败: {}", tmp_path.display()))?;
        fs::rename(&tmp_path, path)
            .await
            .with_context(|| format!("替换快照文件失败: {}", path.display()))?;
        Ok(())
    }

    fn is_snapshot_expired(&self, generated_at: u64, now: u64) -> bool {
        match self.snapshot_ttl {
            Some(ttl) if now > generated_at => now - generated_at > ttl.as_secs(),
            Some(_) => false,
            None => false,
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}
