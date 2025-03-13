use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use parking_lot::RwLock;
use crate::{
    Error,
    disk::DiskInfo,
    hardware::{memory::Memory, temperature::Temperature},
    network::NetworkInfo,
    power::PowerInfo,
    process::Process,
    Result,
};

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
    resources: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T> ResourcePool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            resources: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    pub fn acquire(&self) -> Option<T> {
        let mut resources = self.resources.lock().unwrap();
        resources.pop()
    }

    pub fn try_acquire(&self) -> Result<Option<T>, Error> {
        let resources = self.resources.try_lock();
        match resources {
            Ok(mut res) => Ok(res.pop()),
            Err(_) => Err(Error::system("Failed to acquire resource: mutex busy")),
        }
    }

    pub fn release(&self, resource: T) -> Result<(), Error> {
        let mut resources = self.resources.lock().unwrap();
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
    usage_tx: Sender<ResourceUsage>,
    usage_rx: Arc<Mutex<Receiver<ResourceUsage>>>,
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
        let (usage_tx, usage_rx) = mpsc::channel();

        Self {
            metric_cache: Arc::new(Cache::new(Duration::from_secs(60))),
            usage_tx,
            usage_rx: Arc::new(Mutex::new(usage_rx)),
            usage_state: Arc::new(RwLock::new(Default::default())),
        }
    }

    pub fn subscribe(&self) -> Receiver<ResourceUsage> {
        let (tx, rx) = mpsc::channel();
        let usage_rx = self.usage_rx.clone();
        
        thread::spawn(move || {
            let rx = usage_rx.lock().unwrap();
            while let Ok(usage) = rx.recv() {
                if tx.send(usage).is_err() {
                    break;
                }
            }
        });
        
        rx
    }

    pub fn track_resource_usage(&self, resource_type: &str, usage: f64) -> Result<(), Error> {
        let usage = ResourceUsage {
            resource_type: resource_type.to_string(),
            usage_percent: usage,
            timestamp: Instant::now(),
        };

        {
            let mut state = self.usage_state.write();
            *state
                .active_resources
                .entry(resource_type.to_string())
                .or_insert(0) += 1;
            *state
                .peak_usage
                .entry(resource_type.to_string())
                .or_insert(0.0) = f64::max(
                state.peak_usage.get(resource_type).unwrap_or(&0.0),
                usage.usage_percent,
            );
        }

        self.usage_tx.send(usage).map_err(|_| Error::system("Failed to send resource usage"))
    }

    pub fn track_resource_usage_with_timeout(
        &self,
        resource_type: &str,
        usage: f64,
        timeout: Duration,
    ) -> Result<(), Error> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            match self.track_resource_usage(resource_type, usage) {
                Ok(_) => return Ok(()),
                Err(_) if start.elapsed() < timeout => thread::sleep(Duration::from_millis(10)),
                Err(e) => return Err(e),
            }
        }
        Err(Error::system("Resource tracking timeout"))
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

/// Resource monitor for tracking system metrics
///
/// This struct provides functionality to monitor various system resources and metrics.
pub struct ResourceMonitor {
    /// Interval between updates in milliseconds
    update_interval: Duration,
    /// Channel for sending stop signals
    stop_tx: Sender<()>,
    /// Thread handle for the monitoring task
    monitor_thread: Option<thread::JoinHandle<()>>,
    /// Channel for receiving resource updates
    update_rx: Arc<Mutex<Receiver<ResourceUpdate>>>,
}

/// Resource update containing system metrics
///
/// This struct contains various system metrics collected during monitoring.
#[derive(Debug, Clone)]
pub struct ResourceUpdate {
    /// Memory usage information
    pub memory: Memory,
    /// Temperature readings
    pub temperature: Temperature,
    /// Network interface statistics
    pub network: NetworkInfo,
    /// Disk usage information
    pub disk: DiskInfo,
    /// Power consumption information
    pub power: PowerInfo,
    /// List of running processes
    pub processes: Vec<Process>,
    /// Timestamp of the update
    pub timestamp: Instant,
}

impl ResourceMonitor {
    /// Creates a new ResourceMonitor with the specified update interval
    pub fn new(update_interval: Duration) -> Result<Self> {
        let (stop_tx, stop_rx) = mpsc::channel();
        let (update_tx, update_rx) = mpsc::channel();
        let update_rx = Arc::new(Mutex::new(update_rx));

        let monitor_thread = {
            let update_interval = update_interval;
            let update_tx = update_tx;
            let stop_rx = stop_rx;

            thread::spawn(move || {
                let mut last_update = Instant::now();

                loop {
                    // Check for stop signal
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }

                    // Sleep until next update
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_update);
                    if elapsed < update_interval {
                        thread::sleep(update_interval - elapsed);
                        continue;
                    }

                    // Collect resource metrics
                    let memory = Memory::new();
                    let temperature = Temperature::new();
                    let network = NetworkInfo::new();
                    let disk = DiskInfo::new();
                    let power = PowerInfo::new();
                    let processes = Process::get_all().unwrap_or_default();

                    // Create and send update
                    let update = ResourceUpdate {
                        memory,
                        temperature,
                        network,
                        disk,
                        power,
                        processes,
                        timestamp: Instant::now(),
                    };

                    if update_tx.send(update).is_err() {
                        // Receiver was dropped, stop monitoring
                        break;
                    }

                    last_update = now;
                }
            })
        };

        Ok(Self {
            update_interval,
            stop_tx,
            monitor_thread: Some(monitor_thread),
            update_rx,
        })
    }

    /// Gets the next resource update
    pub fn next_update(&self) -> Result<ResourceUpdate> {
        self.update_rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|e| crate::Error::resource_error(format!("Failed to receive update: {}", e)))
    }

    /// Stops the resource monitor
    pub fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.monitor_thread.take() {
            // Send stop signal
            self.stop_tx.send(()).map_err(|e| {
                crate::Error::resource_error(format!("Failed to send stop signal: {}", e))
            })?;

            // Wait for monitor thread to finish
            handle.join().map_err(|_| {
                crate::Error::resource_error("Failed to join monitor thread".to_string())
            })?;
        }

        Ok(())
    }
}

impl Drop for ResourceMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new(Duration::from_millis(100)).unwrap();
        
        // Get first update
        let update = monitor.next_update().unwrap();
        
        // Verify update contents
        assert!(update.memory.total > 0);
        assert!(!update.processes.is_empty());
        
        // Verify timestamp
        assert!(update.timestamp <= Instant::now());
    }

    #[test]
    fn test_resource_monitor_stop() {
        let mut monitor = ResourceMonitor::new(Duration::from_millis(100)).unwrap();
        
        // Get an update
        let _ = monitor.next_update().unwrap();
        
        // Stop the monitor
        monitor.stop().unwrap();
        
        // Verify monitor is stopped
        assert!(monitor.monitor_thread.is_none());
    }
}
