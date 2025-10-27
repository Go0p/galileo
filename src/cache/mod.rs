use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;

/// 缓存后端抽象：统一 `get` / `put` / `remove` 接口，支持插拔式实现。
#[async_trait]
pub trait CacheBackend: Send + Sync + 'static {
    type Key: Eq + Hash + Clone + Send + Sync + 'static;
    type Value: Send + Sync + 'static;

    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>>;

    async fn insert(&self, key: Self::Key, value: Arc<Self::Value>, ttl: Option<Duration>);

    async fn remove(&self, key: &Self::Key);
}

/// 高层缓存封装，提供 `load_or_fetch` 等常用操作，并内建 per-key 并发锁。
pub struct Cache<B>
where
    B: CacheBackend,
{
    backend: B,
    locks: DashMap<B::Key, Arc<tokio::sync::Mutex<()>>>,
}

impl<B> Clone for Cache<B>
where
    B: CacheBackend + Clone,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            locks: DashMap::new(),
        }
    }
}

impl<B> Default for Cache<B>
where
    B: CacheBackend + Default,
{
    fn default() -> Self {
        Self::new(B::default())
    }
}

impl<B> Cache<B>
where
    B: CacheBackend,
{
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            locks: DashMap::new(),
        }
    }

    pub async fn get(&self, key: &B::Key) -> Option<Arc<B::Value>> {
        self.backend.get(key).await
    }

    pub async fn insert(&self, key: B::Key, value: B::Value, ttl: Option<Duration>) {
        self.backend.insert(key, Arc::new(value), ttl).await;
    }

    pub async fn insert_arc(&self, key: B::Key, value: Arc<B::Value>, ttl: Option<Duration>) {
        self.backend.insert(key, value, ttl).await;
    }

    pub async fn load_or_fetch<F, Fut>(
        &self,
        key: B::Key,
        fetcher: F,
    ) -> anyhow::Result<Arc<B::Value>>
    where
        F: FnOnce(&B::Key) -> Fut + Send,
        Fut: std::future::Future<Output = anyhow::Result<FetchOutcome<B::Value>>> + Send,
    {
        if let Some(hit) = self.backend.get(&key).await {
            return Ok(hit);
        }

        let lock = self
            .locks
            .entry(key.clone())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        if let Some(hit) = self.backend.get(&key).await {
            return Ok(hit);
        }

        let FetchOutcome { value, ttl } = fetcher(&key).await?;
        let arc = Arc::new(value);
        self.backend.insert(key.clone(), arc.clone(), ttl).await;
        Ok(arc)
    }

    pub async fn remove(&self, key: &B::Key) {
        self.backend.remove(key).await;
    }
}

/// fetch 回源的返回值：包含实体及 TTL。
pub struct FetchOutcome<V> {
    pub value: V,
    pub ttl: Option<Duration>,
}

impl<V> FetchOutcome<V> {
    pub fn new(value: V, ttl: Option<Duration>) -> Self {
        Self { value, ttl }
    }
}

/// 默认内存后端，基于 DashMap + Arc 实现，支持 TTL。
#[derive(Clone)]
pub struct InMemoryBackend<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    entries: DashMap<K, Entry<V>>,
}

impl<K, V> Default for InMemoryBackend<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }
}

#[derive(Clone)]
struct Entry<V> {
    value: Arc<V>,
    expires_at: Option<Instant>,
}

impl<V> Entry<V> {
    fn new(value: Arc<V>, ttl: Option<Duration>) -> Self {
        Self {
            value,
            expires_at: ttl.map(|dur| Instant::now() + dur),
        }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(deadline) => Instant::now() >= deadline,
            None => false,
        }
    }
}

#[async_trait]
impl<K, V> CacheBackend for InMemoryBackend<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Key = K;
    type Value = V;

    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.entries.remove(key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    async fn insert(&self, key: Self::Key, value: Arc<Self::Value>, ttl: Option<Duration>) {
        self.entries.insert(key, Entry::new(value, ttl));
    }

    async fn remove(&self, key: &Self::Key) {
        self.entries.remove(key);
    }
}
