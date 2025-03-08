use crate::Error;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

pub struct ResourcePool<T> {
    resources: Arc<tokio::sync::Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T> ResourcePool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            resources: Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    pub async fn acquire(&self) -> Option<T> {
        let mut resources = self.resources.lock().await;
        resources.pop()
    }

    pub async fn try_acquire(&self) -> Result<Option<T>, Error> {
        let resources = self.resources.try_lock();
        match resources {
            Ok(mut res) => Ok(res.pop()),
            Err(_) => Err(Error::system("Failed to acquire resource: mutex busy")),
        }
    }

    pub async fn release(&self, resource: T) -> Result<(), Error> {
        let mut resources = self.resources.lock().await;
        if resources.len() >= self.max_size {
            return Err(Error::system("Resource pool is full"));
        }
        resources.push(resource);
        Ok(())
    }
}

pub struct Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read();
        entries.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.value.clone())
            }
        })
    }

    pub fn set(&self, key: K, value: V) {
        let mut entries = self.entries.write();
        entries.insert(key, CacheEntry::new(value, self.ttl));
    }

    pub fn remove(&self, key: &K) {
        self.entries.write().remove(key);
    }

    pub fn clear_expired(&self) {
        self.entries.write().retain(|_, entry| !entry.is_expired());
    }
}

#[derive(Clone)]
pub struct ResourceManager {
    metric_cache: Arc<Cache<String, Vec<u8>>>,
    usage_tx: broadcast::Sender<ResourceUsage>,
    usage_state: Arc<RwLock<ResourceUsageState>>,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub resource_type: String,
    pub usage_percent: f64,
    pub timestamp: Instant,
}

#[derive(Default)]
struct ResourceUsageState {
    active_resources: HashMap<String, usize>,
    peak_usage: HashMap<String, f64>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let (usage_tx, _) = broadcast::channel(100);

        Self {
            metric_cache: Arc::new(Cache::new(Duration::from_secs(60))),
            usage_tx,
            usage_state: Arc::new(RwLock::new(Default::default())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ResourceUsage> {
        self.usage_tx.subscribe()
    }

    pub async fn track_resource_usage(&self, resource_type: &str, usage: f64) {
        let usage = ResourceUsage {
            resource_type: resource_type.to_string(),
            usage_percent: usage,
            timestamp: Instant::now(),
        };

        {
            let mut state = self.usage_state.write();
            *state.active_resources.entry(resource_type.to_string()).or_insert(0) += 1;
            *state.peak_usage.entry(resource_type.to_string()).or_insert(0.0) =
                f64::max(state.peak_usage.get(resource_type).unwrap_or(&0.0), usage.usage_percent);
        }

        let _ = self.usage_tx.send(usage);
    }

    pub async fn track_resource_usage_with_timeout(
        &self,
        resource_type: &str,
        usage: f64,
        timeout: Duration,
    ) -> Result<(), Error> {
        let result = async {
            self.track_resource_usage(resource_type, usage).await
        };
        tokio::time::timeout(timeout, result)
            .await
            .map_err(|_| Error::system("Resource tracking timeout"))?
    }

    pub fn get_cached_metric(&self, key: &str) -> Option<Vec<u8>> {
        self.metric_cache.get(&key.to_string())
    }

    pub fn cache_metric(&self, key: &str, value: Vec<u8>) {
        self.metric_cache.set(key.to_string(), value);
    }

    pub fn get_usage_stats(&self) -> HashMap<String, f64> {
        self.usage_state.read().peak_usage.clone()
    }

    pub fn cleanup_cache(&self) {
        self.metric_cache.clear_expired();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    async fn test_resource_pool() {
        let pool = ResourcePool::new(2);

        assert!(pool.acquire().await.is_none());

        pool.release(42).await.expect("Failed to release resource");
        assert_eq!(pool.acquire().await, Some(42));
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let manager = ResourceManager::new();
        manager.cache_metric("test", vec![1, 2, 3]);

        let cached = manager.get_cached_metric("test");
        assert_eq!(cached, Some(vec![1, 2, 3]));
    }
}