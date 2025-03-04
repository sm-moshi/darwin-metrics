use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use parking_lot::RwLock as PLRwLock;
use crate::Error;

/// Cache entry with value and expiration
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

/// Thread-safe resource pool with async support
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

    pub async fn release(&self, resource: T) -> Result<(), Error> {
        let mut resources = self.resources.lock().await;
        if resources.len() >= self.max_size {
            return Err(Error::SystemError("Resource pool is full".to_string()));
        }
        resources.push(resource);
        Ok(())
    }
}

/// Thread-safe cache with TTL support
pub struct Cache<K, V> 
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    entries: Arc<PLRwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(PLRwLock::new(HashMap::new())),
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
        let mut entries = self.entries.write();
        entries.remove(key);
    }

    pub fn clear_expired(&self) {
        let mut entries = self.entries.write();
        entries.retain(|_, entry| !entry.is_expired());
    }
}

/// Global resource manager for system metrics
#[derive(Clone)]
pub struct ResourceManager {
    /// Shared cache for metric values
    metric_cache: Arc<Cache<String, Vec<u8>>>,
    /// Channel for resource usage notifications
    usage_tx: broadcast::Sender<ResourceUsage>,
    /// Shared state for tracking resource usage
    usage_state: Arc<RwLock<ResourceUsageState>>,
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub resource_type: String,
    pub usage_percent: f64,
    pub timestamp: Instant,
}

/// Internal state for tracking resource usage
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
            usage_state: Arc::new(RwLock::new(ResourceUsageState::default())),
        }
    }

    /// Get a subscription to resource usage events
    pub fn subscribe(&self) -> broadcast::Receiver<ResourceUsage> {
        self.usage_tx.subscribe()
    }

    /// Track resource acquisition
    pub async fn track_resource_usage(&self, resource_type: &str, usage: f64) {
        let usage = ResourceUsage {
            resource_type: resource_type.to_string(),
            usage_percent: usage,
            timestamp: Instant::now(),
        };

        // Update usage state
        {
            let mut state = self.usage_state.write().unwrap();
            let count = state.active_resources.entry(resource_type.to_string()).or_insert(0);
            *count += 1;
            
            let peak = state.peak_usage.entry(resource_type.to_string()).or_insert(0.0);
            *peak = f64::max(*peak, usage.usage_percent);
        }

        // Broadcast usage update
        let _ = self.usage_tx.send(usage);
    }

    /// Track resource acquisition with timeout
    pub async fn track_resource_usage_with_timeout(
        &self,
        resource_type: &str,
        usage: f64,
        timeout: Duration,
    ) -> Result<(), Error> {
        tokio::select! {
            _ = self.track_resource_usage(resource_type, usage) => Ok(()),
            _ = tokio::time::sleep(timeout) => {
                Err(Error::SystemError("Resource tracking timeout".to_string()))
            }
        }
    }

    /// Get cached metric value
    pub fn get_cached_metric(&self, key: &str) -> Option<Vec<u8>> {
        self.metric_cache.get(&key.to_string())
    }

    /// Cache metric value
    pub fn cache_metric(&self, key: &str, value: Vec<u8>) {
        self.metric_cache.set(key.to_string(), value);
    }

    /// Get current resource usage statistics
    pub fn get_usage_stats(&self) -> HashMap<String, f64> {
        let state = self.usage_state.read().unwrap();
        state.peak_usage.clone()
    }

    /// Clear expired cache entries
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
    use std::thread;

    #[tokio::test]
    async fn test_async_resource_pool() {
        let pool = ResourcePool::new(2);
        
        pool.release(1).await.unwrap();
        pool.release(2).await.unwrap();
        
        assert!(pool.release(3).await.is_err());
        
        assert_eq!(pool.acquire().await, Some(2));
        assert_eq!(pool.acquire().await, Some(1));
        assert_eq!(pool.acquire().await, None);
    }

    #[test]
    fn test_cache() {
        let cache = Cache::new(Duration::from_millis(100));
        
        // Set and get
        cache.set("key1", 42);
        assert_eq!(cache.get(&"key1"), Some(42));
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let manager = ResourceManager::new();
        let mut rx = manager.subscribe();
        
        // Track usage
        manager.track_resource_usage("cpu", 50.0).await;
        
        // Receive usage update
        let usage = rx.recv().await.unwrap();
        assert_eq!(usage.resource_type, "cpu");
        assert_eq!(usage.usage_percent, 50.0);
        
        // Check stats
        let stats = manager.get_usage_stats();
        assert_eq!(stats.get("cpu"), Some(&50.0));
    }
}