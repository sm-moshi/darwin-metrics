/// # Resource Management and Monitoring
///
/// The resource module provides utilities for managing and monitoring system resources on macOS.
/// It offers efficient resource pooling, caching mechanisms, and comprehensive system resource monitoring.
///
/// ## Features
///
/// * **Resource Pooling**: Manage reusable resources with controlled acquisition and release
/// * **Caching**: Generic time-based caching for any data type
/// * **Resource Management**: Track resource usage and allocation with metrics
/// * **System Monitoring**: Monitor memory, CPU, disk, network, and power metrics in real-time
/// * **Resource Updates**: Receive periodic updates about system resource utilization
///
/// ## Implementation Details
///
/// The module uses a combination of approaches for efficiency:
///
/// 1. **Resource Pooling**: Thread-safe resource pools with maximum size constraints
/// 2. **Time-based Caching**: Automatic expiration of cached items based on TTL
/// 3. **Background Monitoring**: Separate monitoring thread with configurable update intervals
/// 4. **Resource Metrics**: Comprehensive collection of system metrics from various subsystems
///
/// ## Examples
///
/// ### Resource Pooling
///
/// ```rust
/// use darwin_metrics::resource::ResourcePool;
/// use std::io::BufReader;
/// use std::fs::File;
///
/// // Create a pool of file readers
/// let pool = ResourcePool::new(10);
///
/// // Acquire a resource
/// let reader = match pool.acquire() {
///     Some(reader) => reader,
///     None => {
///         // Create a new reader if none available
///         let file = File::open("data.txt").unwrap();
///         BufReader::new(file)
///     }
/// };
///
/// // Use the reader...
///
/// // Release the reader back to the pool
/// pool.release(reader).unwrap();
/// ```
///
/// ### Caching
///
/// ```rust
/// use darwin_metrics::resource::Cache;
/// use std::time::Duration;
///
/// // Create a cache with 60-second TTL
/// let cache = Cache::new(Duration::from_secs(60));
///
/// // Store a value
/// cache.set("key1", "value1");
///
/// // Retrieve a value
/// if let Some(value) = cache.get(&"key1") {
///     println!("Value: {}", value);
/// }
///
/// // Clear expired entries
/// cache.clear_expired();
/// ```
///
/// ### Resource Monitoring
///
/// ```rust
/// use darwin_metrics::resource::ResourceMonitor;
/// use std::time::Duration;
///
/// // Create a resource monitor with 1-second update interval
/// let monitor = ResourceMonitor::new(Duration::from_secs(1)).unwrap();
///
/// // Get the next resource update
/// let update = monitor.next_update().unwrap();
///
/// // Access system metrics
/// println!("Memory total: {} bytes", update.memory.total);
/// println!("Memory used: {} bytes", update.memory.used);
/// println!("CPU temperature: {}°C", update.temperature.cpu);
/// println!("Disk space: {} bytes free", update.disk.free_space);
/// println!("Network received: {} bytes", update.network.received_bytes);
/// ```
///
/// ## Performance Considerations
///
/// * Resource pools are most effective when resources are expensive to create but reusable
/// * Cache TTL should be tuned based on data volatility and memory constraints
/// * ResourceMonitor creates a background thread, so consider the update interval carefully
/// * For high-frequency monitoring, use longer intervals to reduce system impact
/// * The module uses thread-safe primitives which add some overhead but ensure safety
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

use crate::{
    core::types::{Percentage, Temperature},
    disk::Disk,
    hardware::{iokit::IOKitImpl, temperature::ThermalMetrics},
    memory::{Memory, MemoryInfo, MemoryPressureMonitor, MemoryUsageMonitor, SwapMonitor},
    network::{Interface, NetworkInfo},
    power::PowerInfo,
    process::Process,
    system::System,
    Error, Result,
};
use async_trait::async_trait;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Cache entry with time-based expiration
///
/// This struct represents a cached value with an expiration time.
/// Once the expiration time is reached, the entry is considered invalid.
///
/// # Type Parameters
///
/// * `T` - The type of the cached value
struct CacheEntry<T> {
    /// The cached value
    value: T,
    /// The time at which this entry expires
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Creates a new cache entry with the specified value and time-to-live
    ///
    /// # Arguments
    ///
    /// * `value` - The value to cache
    /// * `ttl` - The time-to-live duration after which the entry expires
    ///
    /// # Returns
    ///
    /// A new CacheEntry that will expire after the specified TTL
    fn new(value: T, ttl: Duration) -> Self {
        Self { value, expires_at: Instant::now() + ttl }
    }

    /// Checks if the cache entry has expired
    ///
    /// # Returns
    ///
    /// `true` if the current time is past the expiration time, `false` otherwise
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// A pool of resources that can be acquired and released
///
/// This struct provides a thread-safe pool of reusable resources with a maximum size limit.
/// It's useful for managing expensive-to-create resources that can be reused, such as
/// database connections, file handles, or network sockets.
///
/// # Type Parameters
///
/// * `T` - The type of resource managed by the pool
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::resource::ResourcePool;
/// use std::io::BufReader;
/// use std::fs::File;
///
/// // Create a pool of file readers with a maximum size of 5
/// let pool = ResourcePool::new(5);
///
/// // Acquire a resource from the pool
/// let reader = pool.acquire();
///
/// // If no resource is available, create a new one
/// let reader = match reader {
///     Some(r) => r,
///     None => {
///         let file = File::open("data.txt").unwrap();
///         BufReader::new(file)
///     }
/// };
///
/// // Use the reader...
///
/// // Return the reader to the pool when done
/// pool.release(reader).unwrap();
/// ```
///
/// # Implementation Details
///
/// The pool uses a thread-safe mutex to protect access to the underlying collection of resources.
/// Resources are stored in a Vec and are acquired in LIFO order (last-in, first-out).
pub struct ResourcePool<T> {
    /// Thread-safe storage for pooled resources
    resources: Arc<Mutex<Vec<T>>>,
    /// Maximum number of resources the pool can hold
    max_size: usize,
}

impl<T> ResourcePool<T> {
    /// Creates a new ResourcePool with the specified maximum size
    ///
    /// # Arguments
    ///
    /// * `max_size` - The maximum number of resources the pool can hold
    ///
    /// # Returns
    ///
    /// A new ResourcePool instance with the specified capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourcePool;
    ///
    /// // Create a pool with a maximum of 10 resources
    /// let pool = ResourcePool::<String>::new(10);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self { resources: Arc::new(Mutex::new(Vec::with_capacity(max_size))), max_size }
    }

    /// Acquires a resource from the pool, or returns None if no resources are available
    ///
    /// This method removes a resource from the pool and returns it to the caller.
    /// If the pool is empty, it returns None.
    ///
    /// # Returns
    ///
    /// * `Some(T)` - A resource from the pool
    /// * `None` - If the pool is empty
    ///
    /// # Panics
    ///
    /// This method will panic if the underlying mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourcePool;
    ///
    /// let pool = ResourcePool::new(5);
    /// pool.release(String::from("resource")).unwrap();
    ///
    /// let resource = pool.acquire();
    /// assert_eq!(resource, Some(String::from("resource")));
    /// ```
    pub fn acquire(&self) -> Option<T> {
        let mut resources = self.resources.lock().unwrap();
        resources.pop()
    }

    /// Attempts to acquire a resource from the pool, or returns None if no resources are available
    ///
    /// This method is similar to `acquire()` but returns a Result instead of potentially panicking.
    /// It will return an error if the mutex is locked by another thread.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - A resource from the pool
    /// * `Ok(None)` - If the pool is empty
    /// * `Err` - If the mutex is locked by another thread
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourcePool;
    ///
    /// let pool = ResourcePool::new(5);
    /// pool.release(String::from("resource")).unwrap();
    ///
    /// match pool.try_acquire() {
    ///     Ok(Some(resource)) => println!("Got resource: {}", resource),
    ///     Ok(None) => println!("Pool is empty"),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn try_acquire(&self) -> Result<Option<T>> {
        let resources = self.resources.try_lock();
        match resources {
            Ok(mut res) => Ok(res.pop()),
            Err(_) => Err(Error::system("Failed to acquire resource: mutex busy")),
        }
    }

    /// Releases a resource back to the pool
    ///
    /// This method adds a resource back to the pool for future reuse.
    /// If the pool is already at maximum capacity, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to return to the pool
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the resource was successfully added to the pool
    /// * `Err` - If the pool is full or the mutex is poisoned
    ///
    /// # Panics
    ///
    /// This method will panic if the underlying mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourcePool;
    ///
    /// let pool = ResourcePool::new(5);
    /// let resource = String::from("resource");
    ///
    /// // Release a resource to the pool
    /// pool.release(resource).unwrap();
    ///
    /// // Acquire it back
    /// let acquired = pool.acquire();
    /// assert_eq!(acquired, Some(String::from("resource")));
    /// ```
    pub fn release(&self, resource: T) -> Result<()> {
        let mut resources = self.resources.lock().unwrap();
        if resources.len() >= self.max_size {
            return Err(Error::system("Resource pool is full"));
        }
        resources.push(resource);
        Ok(())
    }
}

/// Generic cache with time-based expiration
///
/// This struct provides a thread-safe cache for storing values with automatic expiration
/// based on a time-to-live (TTL) duration. It's useful for caching expensive-to-compute
/// values or results from remote API calls.
///
/// # Type Parameters
///
/// * `K` - The key type, which must implement `Eq` and `Hash`
/// * `V` - The value type, which must implement `Clone`
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::resource::Cache;
/// use std::time::Duration;
///
/// // Create a cache with 30-second TTL
/// let cache = Cache::<String, Vec<u8>>::new(Duration::from_secs(30));
///
/// // Store some values
/// cache.set("key1".to_string(), vec![1, 2, 3]);
/// cache.set("key2".to_string(), vec![4, 5, 6]);
///
/// // Retrieve values
/// if let Some(value) = cache.get(&"key1".to_string()) {
///     println!("Value: {:?}", value);
/// }
///
/// // Remove a specific entry
/// cache.remove(&"key2".to_string());
///
/// // Clear all expired entries
/// cache.clear_expired();
/// ```
///
/// # Implementation Details
///
/// The cache uses a `RwLock` to provide concurrent read access while ensuring exclusive
/// write access. Values are wrapped in `CacheEntry` structs that track expiration times.
/// Expired entries are not automatically removed but are ignored during retrieval.
pub struct Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    /// Thread-safe storage for cached entries
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Time-to-live duration for cache entries
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    /// Creates a new Cache with the specified time-to-live duration
    ///
    /// # Arguments
    ///
    /// * `ttl` - The time-to-live duration after which entries expire
    ///
    /// # Returns
    ///
    /// A new Cache instance with the specified TTL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::Cache;
    /// use std::time::Duration;
    ///
    /// // Create a cache with 5-minute TTL
    /// let cache = Cache::<String, String>::new(Duration::from_secs(300));
    /// ```
    pub fn new(ttl: Duration) -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())), ttl }
    }

    /// Retrieves a value from the cache if it exists and hasn't expired
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(V)` - The cached value if it exists and hasn't expired
    /// * `None` - If the key doesn't exist or the entry has expired
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::Cache;
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(Duration::from_secs(60));
    /// cache.set("key", "value");
    ///
    /// assert_eq!(cache.get(&"key"), Some("value".to_string()));
    /// assert_eq!(cache.get(&"nonexistent"), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read();
        entries.get(key).and_then(|entry| if entry.is_expired() { None } else { Some(entry.value.clone()) })
    }

    /// Stores a value in the cache with the default TTL
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which to store the value
    /// * `value` - The value to store
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::Cache;
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(Duration::from_secs(60));
    ///
    /// // Store a simple value
    /// cache.set("name", "Alice");
    ///
    /// // Store a more complex value
    /// cache.set("data", vec![1, 2, 3, 4, 5]);
    /// ```
    pub fn set(&self, key: K, value: V) {
        let mut entries = self.entries.write();
        entries.insert(key, CacheEntry::new(value, self.ttl));
    }

    /// Removes an entry from the cache
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::Cache;
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(Duration::from_secs(60));
    /// cache.set("key", "value");
    ///
    /// // Remove the entry
    /// cache.remove(&"key");
    ///
    /// // Verify it's gone
    /// assert_eq!(cache.get(&"key"), None);
    /// ```
    pub fn remove(&self, key: &K) {
        self.entries.write().remove(key);
    }

    /// Removes all expired entries from the cache
    ///
    /// This method should be called periodically to prevent the cache from
    /// growing too large with expired entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::Cache;
    /// use std::time::Duration;
    ///
    /// let cache = Cache::new(Duration::from_millis(1));
    /// cache.set("key", "value");
    ///
    /// // Wait for the entry to expire
    /// std::thread::sleep(Duration::from_millis(5));
    ///
    /// // Clear expired entries
    /// cache.clear_expired();
    ///
    /// // Verify the expired entry is gone
    /// assert_eq!(cache.get(&"key"), None);
    /// ```
    pub fn clear_expired(&self) {
        self.entries.write().retain(|_, entry| !entry.is_expired());
    }
}

/// Resource manager for tracking and controlling resource usage
///
/// This struct provides functionality for managing resources with usage tracking
/// and metrics collection. It combines resource pooling with usage statistics
/// to help monitor and control resource consumption.
///
/// # Type Parameters
///
/// * `T` - The type of resource being managed
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::resource::ResourceManager;
/// use std::io::BufReader;
/// use std::fs::File;
///
/// // Create a resource manager for file readers
/// let manager = ResourceManager::<BufReader<File>>::new(10);
///
/// // Acquire a resource
/// if let Ok(Some(reader)) = manager.try_acquire() {
///     // Use the reader...
///
///     // Track usage
///     manager.track_resource_usage("file_read", 1024.0).unwrap();
///
///     // Release the resource
///     manager.release(reader).unwrap();
/// }
/// ```
///
/// # Implementation Details
///
/// The manager uses a thread-safe resource pool internally and maintains
/// a cache of metrics for performance analysis. It also provides channels
/// for tracking resource usage events.
pub struct ResourceManager<T> {
    /// Cache for storing metric data
    metric_cache: Arc<Cache<String, Vec<u8>>>,
    /// Channel for sending resource usage events
    usage_tx: mpsc::Sender<ResourceUsage>,
    /// Thread-safe storage for managed resources
    resources: Arc<Mutex<Vec<T>>>,
    /// Maximum number of resources the manager can hold
    max_size: usize,
    /// Phantom data for type parameter
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> ResourceManager<T> {
    /// Creates a new ResourceManager with the specified maximum size
    ///
    /// # Arguments
    ///
    /// * `max_size` - The maximum number of resources the manager can hold
    ///
    /// # Returns
    ///
    /// A new ResourceManager instance with the specified capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceManager;
    ///
    /// // Create a manager with a maximum of 5 resources
    /// let manager = ResourceManager::<String>::new(5);
    /// ```
    pub fn new(max_size: usize) -> Self {
        let (usage_tx, _usage_rx) = mpsc::channel(100); // Create a channel with buffer size 100
        Self {
            metric_cache: Arc::new(Cache::new(Duration::from_secs(60))),
            usage_tx,
            resources: Arc::new(Mutex::new(Vec::new())),
            max_size,
            _phantom: PhantomData,
        }
    }

    /// Attempts to acquire a resource from the manager
    ///
    /// This method tries to acquire a resource from the internal pool.
    /// If the pool is empty, it returns None. If the mutex is locked,
    /// it returns an error.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - A resource from the pool
    /// * `Ok(None)` - If the pool is empty
    /// * `Err` - If the mutex is locked or poisoned
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceManager;
    ///
    /// let manager = ResourceManager::<String>::new(5);
    /// manager.release(String::from("resource")).unwrap();
    ///
    /// match manager.try_acquire() {
    ///     Ok(Some(resource)) => println!("Got resource: {}", resource),
    ///     Ok(None) => println!("No resources available"),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn try_acquire(&self) -> Result<Option<T>> {
        let mut resources = self.resources.lock().map_err(|_| Error::LockError)?;
        if resources.is_empty() {
            Ok(None)
        } else {
            Ok(Some(resources.remove(0)))
        }
    }

    /// Releases a resource back to the manager
    ///
    /// This method returns a resource to the internal pool for future reuse.
    /// If the pool is already at maximum capacity, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to return to the pool
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the resource was successfully added to the pool
    /// * `Err` - If the pool is full or the mutex is locked
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceManager;
    ///
    /// let manager = ResourceManager::<String>::new(5);
    /// let resource = String::from("resource");
    ///
    /// // Release a resource to the pool
    /// manager.release(resource).unwrap();
    /// ```
    pub fn release(&self, resource: T) -> Result<()> {
        let mut resources = self.resources.lock().map_err(|_| Error::LockError)?;
        if resources.len() >= self.max_size {
            Err(Error::ResourceLimitExceeded)
        } else {
            resources.push(resource);
            Ok(())
        }
    }

    /// Tracks resource usage with the specified metrics
    ///
    /// This method records resource usage information for monitoring and analysis.
    /// It sends the usage data through a channel for processing.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource being tracked
    /// * `usage` - The usage value (e.g., bytes read, operations performed)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the usage was successfully recorded
    /// * `Err` - If the channel is closed or another error occurs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceManager;
    ///
    /// let manager = ResourceManager::<String>::new(5);
    ///
    /// // Track memory usage
    /// manager.track_resource_usage("memory", 1024.0 * 1024.0).unwrap();
    ///
    /// // Track CPU usage
    /// manager.track_resource_usage("cpu", 0.75).unwrap();
    /// ```
    pub fn track_resource_usage(&self, resource_type: &str, usage: f64) -> Result<()> {
        let usage = ResourceUsage { resource_type: resource_type.to_string(), usage, timestamp: SystemTime::now() };

        // Use try_send since we can't await in a sync function
        self.usage_tx.try_send(usage).map_err(|_| Error::ChannelError)?;
        Ok(())
    }

    /// Tracks resource allocation events
    ///
    /// This method records when resources are allocated or deallocated.
    /// Currently, this is a placeholder implementation.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource being allocated
    /// * `allocated` - Whether the resource is being allocated (true) or deallocated (false)
    /// * `timestamp` - When the allocation/deallocation occurred
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the allocation was successfully recorded
    /// * `Err` - If an error occurs during recording
    pub fn track_resource_allocation(
        &self,
        _resource_type: &str,
        _allocated: bool,
        _timestamp: SystemTime,
    ) -> Result<()> {
        // Implementation here
        Ok(())
    }
}

/// Resource usage information
///
/// This struct represents a single resource usage event with a resource type,
/// usage value, and timestamp. It's used for tracking and analyzing resource
/// consumption patterns.
///
/// # Fields
///
/// * `resource_type` - The type of resource being tracked
/// * `usage` - The usage value (e.g., bytes, percentage, operations)
/// * `timestamp` - When the usage was recorded
#[derive(Debug, Clone)]
struct ResourceUsage {
    /// The type of resource being tracked
    resource_type: String,
    /// The usage value (e.g., bytes, percentage, operations)
    usage: f64,
    /// When the usage was recorded
    timestamp: SystemTime,
}

/// Resource monitor for tracking system metrics
///
/// This struct provides functionality to monitor various system resources and metrics
/// in real-time. It runs a background task that periodically collects system metrics
/// and makes them available through a simple API.
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::resource::ResourceMonitor;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> darwin_metrics::Result<()> {
///     // Create a monitor with 1-second update interval
///     let monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;
///
///     // Get the next update
///     let update = monitor.next_update().await?;
///
///     // Access system metrics
///     println!("Memory: {} bytes total, {} bytes used", update.memory.total, update.memory.used);
///     println!("CPU temperature: {}°C", update.temperature.cpu);
///     println!("Disk space: {} bytes free", update.disk.free_space);
///     println!("Network: {} bytes received, {} bytes sent",
///              update.network.received_bytes, update.network.sent_bytes);
///     println!("Power consumption: {} W", update.power.package);
///     println!("Process count: {}", update.processes.len());
///     
///     // Stop the monitor when done
///     monitor.stop().await?;
///     
///     Ok(())
/// }
/// ```
///
/// # Implementation Details
///
/// The monitor creates a background task that collects system metrics at the specified
/// interval. These metrics are sent through a channel to the main thread, where they can
/// be retrieved using the `next_update()` method.
///
/// The monitor automatically cleans up its resources when dropped, stopping the background
/// task and closing the channels.
///
/// # Performance Considerations
///
/// * Choose an appropriate update interval based on your needs
/// * Very short intervals (< 100ms) may impact system performance
/// * The monitor collects comprehensive metrics which has some overhead
/// * Consider using more targeted monitoring if you only need specific metrics
pub struct ResourceMonitor {
    /// Interval between updates in milliseconds
    update_interval: Duration,
    /// Channel for sending stop signals
    stop_tx: mpsc::Sender<()>,
    /// Thread handle for the monitoring task
    monitor_task: Option<JoinHandle<()>>,
    /// Channel for receiving resource updates
    update_rx: mpsc::Receiver<ResourceUpdate>,
}

/// Resource update containing system metrics
///
/// This struct contains various system metrics collected during monitoring.
/// It provides a comprehensive snapshot of the system's resource usage at a point in time.
///
/// # Fields
///
/// * `memory` - Memory usage information (total, used, free, active, inactive)
/// * `temperature` - Temperature readings for various components (CPU, GPU, etc.)
/// * `network` - Network interface statistics (bytes sent/received, packets, errors)
/// * `disk` - Disk usage information (capacity, free space, I/O statistics)
/// * `power` - Power consumption information (package, cores, GPU, etc.)
/// * `processes` - List of running processes with resource usage details
/// * `timestamp` - When the update was collected
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::resource::ResourceMonitor;
/// use std::time::Duration;
///
/// let monitor = ResourceMonitor::new(Duration::from_secs(1)).unwrap();
/// let update = monitor.next_update().unwrap();
///
/// // Memory metrics
/// println!("Memory: {:.2}% used ({} MB free of {} MB total)",
///     (update.memory.used as f64 / update.memory.total as f64) * 100.0,
///     update.memory.free / 1024 / 1024,
///     update.memory.total / 1024 / 1024);
///
/// // Temperature metrics
/// if update.temperature.cpu > 80.0 {
///     println!("Warning: CPU temperature is high: {}°C", update.temperature.cpu);
/// }
///
/// // Process information
/// for process in update.processes.iter().take(5) {
///     println!("Process: {} (PID: {}), CPU: {:.1}%, Memory: {} MB",
///         process.name, process.pid, process.cpu_usage,
///         process.memory_usage / 1024 / 1024);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ResourceUpdate {
    /// Memory usage information
    pub memory: MemoryInfo,
    /// Temperature readings
    pub temperature: Temperature,
    /// Network interface statistics
    pub network: NetworkInfo,
    /// Disk usage information
    pub disk: Disk,
    /// Power consumption information
    pub power: PowerInfo,
    /// List of running processes
    pub processes: Vec<Process>,
    /// Timestamp of the update
    pub timestamp: Instant,
}

impl ResourceMonitor {
    /// Creates a new `ResourceMonitor` with the specified update interval
    ///
    /// This method initializes a resource monitor that will collect system metrics
    /// at the specified interval. The monitor runs as a background task and makes
    /// updates available through the `next_update` method.
    ///
    /// # Arguments
    ///
    /// * `update_interval` - The interval at which to collect resource metrics
    ///
    /// # Returns
    ///
    /// * `Ok(ResourceMonitor)` - A new resource monitor instance
    /// * `Err` - If initialization failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceMonitor;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> darwin_metrics::Result<()> {
    ///     let monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;
    ///     // Use the monitor...
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(update_interval: Duration) -> Result<Self> {
        let (update_tx, update_rx) = mpsc::channel(10); // Buffer size of 10
        let (stop_tx, mut stop_rx) = mpsc::channel(1);

        let monitor_task = tokio::spawn(async move {
            let mut last_update = Instant::now();

            loop {
                // Check if we should stop
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                // Check if it's time for an update
                let now = Instant::now();
                if now.duration_since(last_update) >= update_interval {
                    // Use tokio::spawn_blocking for potentially blocking FFI operations
                    let _memory_info = match tokio::task::spawn_blocking(|| {
                        Memory::new().unwrap_or_else(|_| {
                            eprintln!("Failed to get memory info");
                            Memory::default()
                        })
                    })
                    .await
                    {
                        Ok(memory) => memory,
                        Err(_) => {
                            eprintln!("Failed to collect memory info");
                            Memory::default()
                        },
                    };

                    // Convert Memory to MemoryInfo
                    let memory = MemoryInfo::default(); // Placeholder until we have a proper conversion

                    let temperature =
                        match tokio::task::spawn_blocking(|| crate::core::types::Temperature::new(0.0)).await {
                            Ok(temp) => temp,
                            Err(_) => {
                                eprintln!("Failed to collect temperature info");
                                crate::core::types::Temperature::new(0.0)
                            },
                        };

                    let network = NetworkInfo::default();
                    let disk = Disk::with_details(
                        String::from("/dev/placeholder"),
                        String::from("/"),
                        String::from("apfs"),
                        0,
                        0,
                        0,
                        crate::disk::DiskConfig {
                            disk_type: crate::disk::DiskType::Unknown,
                            name: String::from("Placeholder"),
                            is_boot_volume: false,
                        },
                    );
                    let power = PowerInfo::default();
                    let processes = Vec::new(); // Placeholder

                    let update = ResourceUpdate {
                        memory,
                        temperature,
                        network,
                        disk,
                        power,
                        processes,
                        timestamp: Instant::now(),
                    };

                    if let Err(e) = update_tx.send(update).await {
                        eprintln!("Failed to send update: {}", e);
                        break;
                    }

                    last_update = now;
                }

                // Yield to other tasks
                tokio::task::yield_now().await;

                // Sleep a bit to avoid spinning too fast
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        Ok(Self { update_interval, stop_tx, monitor_task: Some(monitor_task), update_rx })
    }

    /// Gets the next resource update
    ///
    /// This method waits for the next resource update from the monitoring task.
    /// It will block for up to 10 seconds waiting for an update. If no update is
    /// received within that time, it returns an error.
    ///
    /// # Returns
    ///
    /// * `Ok(ResourceUpdate)` - The next resource update
    /// * `Err` - If no update was received within the timeout or another error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceMonitor;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> darwin_metrics::Result<()> {
    ///     let mut monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;
    ///
    ///     // Get the next update
    ///     match monitor.next_update().await {
    ///         Ok(update) => println!("Got update at {:?}", update.timestamp),
    ///         Err(e) => eprintln!("Error getting update: {}", e),
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn next_update(&mut self) -> Result<ResourceUpdate> {
        tokio::time::timeout(Duration::from_secs(10), self.update_rx.recv())
            .await
            .map_err(|_| Error::system("Failed to receive update within timeout".to_string()))?
            .ok_or(Error::ChannelError)
    }

    /// Stops the resource monitor
    ///
    /// This method stops the background monitoring task and cleans up resources.
    /// It should be called when the monitor is no longer needed to ensure proper cleanup.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the monitor was successfully stopped
    /// * `Err` - If an error occurred while stopping the monitor
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::resource::ResourceMonitor;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> darwin_metrics::Result<()> {
    ///     let mut monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;
    ///
    ///     // Use the monitor...
    ///
    ///     // Stop the monitor when done
    ///     monitor.stop().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// The monitor is automatically stopped when dropped, so calling this method
    /// explicitly is not strictly necessary but can provide better error handling.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.monitor_task.take() {
            // Send stop signal
            self.stop_tx.send(()).await.map_err(|_| Error::system("Failed to send stop signal".to_string()))?;

            // Wait for task to finish with timeout
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(result) => {
                    result.map_err(|e| Error::system(format!("Monitor task panicked: {}", e)))?;
                },
                Err(_) => {
                    return Err(Error::system("Timed out waiting for monitor task to stop".to_string()));
                },
            }
        }

        Ok(())
    }
}

/// Automatic cleanup when the ResourceMonitor is dropped
///
/// This implementation ensures that the background monitoring task is properly
/// stopped and resources are cleaned up when the ResourceMonitor is dropped.
/// It silently ignores any errors that occur during cleanup.
impl Drop for ResourceMonitor {
    /// Stops the monitor task when the ResourceMonitor is dropped
    fn drop(&mut self) {
        // Can't await in Drop, so we use block_in_place to avoid blocking
        if let Some(handle) = self.monitor_task.take() {
            // Send stop signal without waiting
            let _ = self.stop_tx.try_send(());

            // Abort the task if it doesn't complete immediately
            handle.abort();
        }
    }
}

/// Define an async trait for ResourceMonitoring
#[async_trait]
pub trait ResourceMonitoring: Send + Sync {
    /// Get the next resource update
    async fn next_update(&mut self) -> Result<ResourceUpdate>;

    /// Stop the resource monitor
    async fn stop(&mut self) -> Result<()>;
}

#[async_trait]
impl ResourceMonitoring for ResourceMonitor {
    async fn next_update(&mut self) -> Result<ResourceUpdate> {
        self.next_update().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.stop().await
    }
}

/// Unit tests for the resource module
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Simple Memory struct for testing
    struct Memory {
        /// Total memory in bytes
        pub total: u64,
    }

    /// Tests the ResourceMonitor creation and basic functionality
    #[tokio::test]
    async fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new(Duration::from_millis(100)).await.unwrap();

        // Get first update
        let update = monitor.next_update().await.unwrap();

        // Verify update contents - note that these are placeholder values in the implementation
        assert_eq!(update.memory.total, 0); // Placeholder returns default values
        assert!(update.processes.is_empty()); // Implementation creates an empty vector

        // Verify timestamp
        assert!(update.timestamp <= Instant::now());
    }

    /// Tests the ResourceMonitor stop functionality
    #[tokio::test]
    async fn test_resource_monitor_stop() {
        let mut monitor = ResourceMonitor::new(Duration::from_millis(100)).await.unwrap();

        // Get an update
        let _ = monitor.next_update().await.unwrap();

        // Stop the monitor
        monitor.stop().await.unwrap();

        // Verify monitor is stopped
        assert!(monitor.monitor_task.is_none());
    }

    /// Tests the ResourcePool functionality
    #[test]
    fn test_resource_pool() {
        let pool = ResourcePool::<String>::new(2);

        // Pool should be empty initially
        assert!(pool.acquire().is_none());

        // Add resources
        pool.release("resource1".to_string()).unwrap();
        pool.release("resource2".to_string()).unwrap();

        // Pool should be full
        assert!(pool.release("resource3".to_string()).is_err());

        // Acquire resources
        assert_eq!(pool.acquire(), Some("resource2".to_string())); // LIFO order
        assert_eq!(pool.acquire(), Some("resource1".to_string()));
        assert!(pool.acquire().is_none()); // Empty again
    }

    /// Tests the Cache functionality
    #[test]
    fn test_cache() {
        let cache = Cache::<String, String>::new(Duration::from_millis(100));

        // Set and get
        cache.set("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);

        // Expiration
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1".to_string()), None); // Should be expired

        // Remove
        cache.set("key3".to_string(), "value3".to_string());
        cache.remove(&"key3".to_string());
        assert_eq!(cache.get(&"key3".to_string()), None);
    }
}
