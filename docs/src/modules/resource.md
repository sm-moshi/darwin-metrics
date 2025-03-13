# Resource Module

The Resource module provides efficient resource management utilities for the darwin-metrics crate,
including resource pooling, caching, and usage tracking capabilities.

## Core Components

### ResourcePool

A generic thread-safe pool for managing reusable resources:

```rust
pub struct ResourcePool<T> {
    resources: Arc<tokio::sync::Mutex<Vec<T>>>,
    max_size: usize,
}
```

#### Key Methods

```rust
impl<T> ResourcePool<T> {
    // Create a new resource pool with specified capacity
    pub fn new(max_size: usize) -> Self;
    
    // Acquire a resource asynchronously
    pub async fn acquire(&self) -> Option<T>;
    
    // Try to acquire a resource without blocking
    pub async fn try_acquire(&self) -> Result<Option<T>, Error>;
    
    // Release a resource back to the pool
    pub async fn release(&self, resource: T) -> Result<(), Error>;
}
```

### Cache

A generic time-based cache implementation:

```rust
pub struct Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}
```

#### Key Methods

```rust
impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    // Create a new cache with specified TTL
    pub fn new(ttl: Duration) -> Self;
    
    // Get a cached value
    pub fn get(&self, key: &K) -> Option<V>;
    
    // Set a cached value
    pub fn set(&self, key: K, value: V);
    
    // Remove a cached value
    pub fn remove(&self, key: &K);
    
    // Clear expired entries
    pub fn clear_expired(&self);
}
```

### ResourceManager

Central manager for resource tracking and metrics:

```rust
#[derive(Clone)]
pub struct ResourceManager {
    metric_cache: Arc<Cache<String, Vec<u8>>>,
    usage_tx: broadcast::Sender<ResourceUsage>,
    usage_state: Arc<RwLock<ResourceUsageState>>,
}
```

#### Key Methods

```rust
impl ResourceManager {
    // Create a new resource manager
    pub fn new() -> Self;
    
    // Subscribe to resource usage updates
    pub fn subscribe(&self) -> broadcast::Receiver<ResourceUsage>;
    
    // Track resource usage
    pub async fn track_resource_usage(&self, resource_type: &str, usage: f64);
    
    // Track resource usage with timeout
    pub async fn track_resource_usage_with_timeout(
        &self,
        resource_type: &str,
        usage: f64,
        timeout: Duration,
    ) -> Result<(), Error>;
    
    // Get cached metric
    pub fn get_cached_metric(&self, key: &str) -> Option<Vec<u8>>;
    
    // Cache metric
    pub fn cache_metric(&self, key: &str, value: Vec<u8>);
    
    // Get usage statistics
    pub fn get_usage_stats(&self) -> HashMap<String, f64>;
    
    // Clean up expired cache entries
    pub fn cleanup_cache(&self);
}
```

## Usage Examples

### Resource Pool

```rust
use darwin_metrics::resource::ResourcePool;

#[tokio::main]
async fn main() {
    // Create a pool with capacity 10
    let pool = ResourcePool::new(10);
    
    // Acquire a resource
    if let Some(resource) = pool.acquire().await {
        // Use the resource
        
        // Release it back to the pool
        pool.release(resource).await.expect("Failed to release");
    }
}
```

### Cache

```rust
use darwin_metrics::resource::Cache;
use std::time::Duration;

fn main() {
    // Create cache with 1 minute TTL
    let cache = Cache::new(Duration::from_secs(60));
    
    // Set value
    cache.set("key", "value");
    
    // Get value
    if let Some(value) = cache.get("key") {
        println!("Value: {}", value);
    }
    
    // Clear expired entries
    cache.clear_expired();
}
```

### Resource Manager

```rust
use darwin_metrics::resource::ResourceManager;

#[tokio::main]
async fn main() {
    let manager = ResourceManager::new();
    
    // Subscribe to usage updates
    let mut rx = manager.subscribe();
    
    // Track CPU usage
    manager.track_resource_usage("cpu", 75.5).await;
    
    // Cache a metric
    manager.cache_metric("cpu_temp", vec![42]);
    
    // Get usage stats
    let stats = manager.get_usage_stats();
    println!("Usage stats: {:?}", stats);
}
```

## Thread Safety

The module is designed for concurrent access:

- Uses `Arc` for shared ownership
- `RwLock` for concurrent read/write access
- `tokio::sync::Mutex` for async-aware locking
- `broadcast` channel for usage notifications

## Performance Considerations

1. **Resource Pool**
   - Efficient resource reuse
   - Configurable pool size
   - Non-blocking acquire option
   - Async-aware synchronization

2. **Cache**
   - Time-based expiration
   - Lazy cleanup of expired entries
   - Efficient read/write locking
   - Clone-on-read for thread safety

3. **Resource Manager**
   - Broadcast channel for efficient notifications
   - Cached metrics for fast access
   - Atomic usage tracking
   - Timeout support for tracking operations

## Error Handling

The module uses the crate's error system:

- Pool overflow protection
- Timeout handling
- Thread-safe error propagation
- Graceful resource cleanup

## Best Practices

1. **Resource Management**
   - Always release acquired resources
   - Use appropriate pool sizes
   - Handle timeouts gracefully
   - Clean up expired cache entries

2. **Concurrency**
   - Use appropriate synchronization primitives
   - Handle contention gracefully
   - Implement proper cleanup
   - Monitor resource usage

3. **Performance**
   - Configure appropriate TTLs
   - Monitor pool utilization
   - Clean up unused resources
   - Use timeouts for long operations
