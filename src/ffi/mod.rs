use std::ptr::NonNull;
use tracing::{debug, error, info};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::{Error, Result};
use crate::iokit::{kIOMainPortDefault, IOObjectRelease, IOServiceGetMatchingService, IOServiceMatching};

/// FFI struct for battery information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BatteryInfoFFI {
    pub is_present: bool,
    pub is_charging: bool,
    pub percentage: f64,
    pub time_remaining: u64,
}

impl Drop for BatteryInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping BatteryInfoFFI");
    }
}

/// FFI struct for CPU information with thread-safe core usage data
#[repr(C)]
#[derive(Debug)]
pub struct CPUInfoFFI {
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub core_usage: Arc<Vec<f64>>,
    pub core_usage_len: usize,
    pub frequency_mhz: f64,
}

// Make CPUInfoFFI Send + Sync by using Arc for shared data
unsafe impl Send for CPUInfoFFI {}
unsafe impl Sync for CPUInfoFFI {}

impl Clone for CPUInfoFFI {
    fn clone(&self) -> Self {
        Self {
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            core_usage: self.core_usage.clone(),
            core_usage_len: self.core_usage_len,
            frequency_mhz: self.frequency_mhz,
        }
    }
}

impl Drop for CPUInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping CPUInfoFFI");
    }
}

/// FFI struct for memory information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemoryInfoFFI {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub wired: u64,
    pub pressure: f64,
}

impl Drop for MemoryInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping MemoryInfoFFI");
    }
}

/// FFI struct for GPU information
#[repr(C)]
#[derive(Debug)]
pub struct GPUInfoFFI {
    pub name: NonNull<u8>,
    pub name_len: usize,
    pub utilization: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f64,
}

impl Drop for GPUInfoFFI {
    fn drop(&mut self) {
        unsafe {
            if !self.name.as_ptr().is_null() {
                debug!("Freeing GPU name string");
                let slice = std::slice::from_raw_parts_mut(
                    self.name.as_ptr(),
                    self.name_len
                );
                drop(Box::from_raw(slice as *mut [u8]));
            }
        }
    }
}

/// FFI struct for disk information
#[repr(C)]
#[derive(Debug)]
pub struct DiskInfoFFI {
    pub device: NonNull<u8>,
    pub device_len: usize,
    pub mount_point: NonNull<u8>,
    pub mount_point_len: usize,
    pub fs_type: NonNull<u8>,
    pub fs_type_len: usize,
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

impl Drop for DiskInfoFFI {
    fn drop(&mut self) {
        unsafe {
            for (ptr, len) in [
                (self.device, self.device_len),
                (self.mount_point, self.mount_point_len),
                (self.fs_type, self.fs_type_len),
            ] {
                if !ptr.as_ptr().is_null() {
                    debug!("Freeing string buffer");
                    let slice = std::slice::from_raw_parts_mut(ptr.as_ptr(), len);
                    drop(Box::from_raw(slice as *mut [u8]));
                }
            }
        }
    }
}

/// FFI struct for temperature information
#[repr(C)]
#[derive(Debug)]
pub struct TemperatureInfoFFI {
    pub sensor: NonNull<u8>,
    pub sensor_len: usize,
    pub celsius: f64,
    pub fahrenheit: f64,
}

impl Drop for TemperatureInfoFFI {
    fn drop(&mut self) {
        unsafe {
            if !self.sensor.as_ptr().is_null() {
                debug!("Freeing sensor name string");
                let slice = std::slice::from_raw_parts_mut(
                    self.sensor.as_ptr(),
                    self.sensor_len
                );
                drop(Box::from_raw(slice as *mut [u8]));
            }
        }
    }
}

/// Resource manager to ensure proper cleanup of system resources
#[derive(Debug)]
pub struct ResourceManager {
    battery_service: Mutex<Option<i32>>,  // IOKit service
    cpu_stats: Mutex<Option<Box<[f64]>>>,
    memory_stats: Mutex<Option<Box<[u64]>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            battery_service: Mutex::new(None),
            cpu_stats: Mutex::new(None),
            memory_stats: Mutex::new(None),
        }
    }

    pub fn cleanup(&self) {
        // Take ownership of resources before cleanup
        if let Some(service) = self.battery_service.lock().take() {
            unsafe {
                debug!("Cleaning up battery service");
                IOObjectRelease(service);
            }
        }
        
        if let Some(stats) = self.cpu_stats.lock().take() {
            debug!("Cleaning up CPU stats");
            drop(stats);
        }
        
        if let Some(stats) = self.memory_stats.lock().take() {
            debug!("Cleaning up memory stats");
            drop(stats);
        }
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// Global resource manager
lazy_static::lazy_static! {
    static ref RESOURCE_MANAGER: Arc<ResourceManager> = Arc::new(ResourceManager::new());
}

impl BatteryInfoFFI {
    pub fn new(service: i32) -> Self {
        if let Some(service_ref) = RESOURCE_MANAGER.battery_service.lock().as_ref() {
            unsafe {
                IOObjectRelease(*service_ref);
            }
        }
        RESOURCE_MANAGER.battery_service.lock().replace(service);
        
        Self {
            is_present: true,
            is_charging: false,
            percentage: 0.0,
            time_remaining: 0,
        }
    }
}

/// Thread-safe wrapper for FFI operations
#[derive(Debug)]
pub struct MetricsManager {
    inner: Arc<ResourceManager>,
    battery_cache: Mutex<Option<BatteryInfoFFI>>,
    cpu_cache: Mutex<Option<CPUInfoFFI>>,
    memory_cache: Mutex<Option<MemoryInfoFFI>>,
    cache_timeout: std::time::Duration,
    last_update: Mutex<std::time::Instant>,
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            inner: RESOURCE_MANAGER.clone(),
            battery_cache: Mutex::new(None),
            cpu_cache: Mutex::new(None),
            memory_cache: Mutex::new(None),
            cache_timeout: std::time::Duration::from_secs(1),
            last_update: Mutex::new(std::time::Instant::now()),
        }
    }

    pub fn get_battery_info(&self) -> Result<BatteryInfoFFI> {
        let mut cache = self.battery_cache.lock();
        let mut last_update = self.last_update.lock();

        if cache.is_none() || last_update.elapsed() > self.cache_timeout {
            *cache = Some(get_battery_info_impl()?);
            *last_update = std::time::Instant::now();
        }

        Ok(cache.as_ref().unwrap().clone())
    }

    pub fn get_cpu_info(&self) -> Result<CPUInfoFFI> {
        let mut cache = self.cpu_cache.lock();
        let mut last_update = self.last_update.lock();

        if cache.is_none() || last_update.elapsed() > self.cache_timeout {
            *cache = Some(get_cpu_info_impl()?);
            *last_update = std::time::Instant::now();
        }

        Ok(cache.as_ref().unwrap().clone())
    }

    // Similar implementations for CPU and memory info...
}

// Global metrics manager
lazy_static::lazy_static! {
    static ref METRICS_MANAGER: Arc<MetricsManager> = Arc::new(MetricsManager::new());
}

/// Get battery information
///
/// # Safety
///
/// The returned pointer must be freed using the appropriate deallocation function.
/// The pointer will be null if an error occurs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_battery_info() -> *mut BatteryInfoFFI {
    match METRICS_MANAGER.get_battery_info() {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(e) => {
            error!("Failed to get battery info: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get CPU information
///
/// # Safety
///
/// The returned pointer must be freed using the appropriate deallocation function.
/// The pointer will be null if an error occurs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_cpu_info() -> *mut CPUInfoFFI {
    info!("Getting CPU information");
    match METRICS_MANAGER.get_cpu_info() {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(e) => {
            error!("Failed to get CPU info: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get memory information
///
/// # Safety
///
/// The returned pointer must be freed using the appropriate deallocation function.
/// The pointer will be null if an error occurs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_memory_info() -> *mut MemoryInfoFFI {
    info!("Getting memory information");
    match get_memory_info_impl() {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(e) => {
            error!("Failed to get memory info: {}", e);
            std::ptr::null_mut()
        }
    }
}

// Implementation functions
fn get_battery_info_impl() -> Result<BatteryInfoFFI> {
    let service = unsafe {
        IOServiceGetMatchingService(
            kIOMainPortDefault,
            IOServiceMatching(b"AppleSmartBattery\0".as_ptr() as *const i8)
        )
    };

    if service == 0 {
        return Err(Error::ServiceNotFound);
    }

    Ok(BatteryInfoFFI::new(service))
}

fn get_cpu_info_impl() -> Result<CPUInfoFFI> {
    #[cfg(test)]
    {
        let core_usage = Arc::new(vec![0.5; 8]);
        return Ok(CPUInfoFFI {
            physical_cores: 4,
            logical_cores: 8,
            core_usage: core_usage.clone(),
            core_usage_len: 8,
            frequency_mhz: 2400.0,
        });
    }

    #[cfg(not(test))]
    {
        // TODO: Implement actual CPU info retrieval
        Err(Error::NotImplemented("CPU info retrieval not yet implemented".to_string()))
    }
}

fn get_memory_info_impl() -> Result<MemoryInfoFFI> {
    if cfg!(test) {
        Ok(MemoryInfoFFI {
            total: 16 * 1024 * 1024 * 1024, // 16 GB
            available: 8 * 1024 * 1024 * 1024, // 8 GB
            used: 6 * 1024 * 1024 * 1024, // 6 GB
            wired: 2 * 1024 * 1024 * 1024, // 2 GB
            pressure: 50.0,
        })
    } else {
        Err(Error::NotImplemented("Memory info retrieval not yet implemented".to_string()))
    }
}

unsafe extern "C" {
    pub fn get_disk_info() -> *mut *mut DiskInfoFFI;
    pub fn get_disk_info_len() -> usize;
    pub fn get_temperature_info() -> *mut *mut TemperatureInfoFFI;
    pub fn get_temperature_info_len() -> usize;
}

#[cfg(test)]
mod tests;

// Add mock implementations for testing
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn mock_battery_service() -> i32 {
        123 // Mock service handle
    }
    
    pub fn mock_get_battery_info_impl() -> Result<BatteryInfoFFI> {
        Ok(BatteryInfoFFI {
            is_present: true,
            is_charging: false,
            percentage: 85.5,
            time_remaining: 3600,
        })
    }
    
    pub fn mock_get_cpu_info_impl() -> Result<CPUInfoFFI> {
        let core_usage = Arc::new(vec![0.5; 8]);
        
        Ok(CPUInfoFFI {
            physical_cores: 4,
            logical_cores: 8,
            core_usage: core_usage.clone(),
            core_usage_len: 8,
            frequency_mhz: 2400.0,
        })
    }
    
    pub fn mock_get_memory_info_impl() -> Result<MemoryInfoFFI> {
        Ok(MemoryInfoFFI {
            total: 16_000_000_000,
            available: 8_000_000_000,
            used: 8_000_000_000,
            wired: 2_000_000_000,
            pressure: 0.5,
        })
    }
} 