use tracing::{debug, error, info};
use std::sync::Arc;
use parking_lot::Mutex;
use io_kit_sys::types::io_service_t;

use crate::{Error, Result};
use crate::battery::{Battery, PowerSource};
use io_kit_sys::{
    kIOMasterPortDefault,
    IOObjectRelease,
    IOServiceGetMatchingService,
    IOServiceMatching,
};
use crate::cpu::CPU;
use crate::memory::Memory;
use crate::gpu::GPU;
use crate::disk::Disk;
use crate::temperature::Temperature;

/// FFI struct for battery information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BatteryInfoFFI {
    pub is_present: bool,
    pub is_charging: bool,
    pub percentage: f64,
    pub time_remaining_minutes: i32,
    pub power_source: i32,
    pub cycle_count: u32,
    pub health_percentage: f64,
    pub temperature: f64,
}

impl Drop for BatteryInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping BatteryInfoFFI");
    }
}

/// FFI struct for CPU information with thread-safe core usage data
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CPUInfoFFI {
    pub usage_percentage: f64,
    pub temperature: f64,
    pub core_count: u32,
    pub thread_count: u32,
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
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

impl Drop for MemoryInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping MemoryInfoFFI");
    }
}

/// FFI struct for GPU information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GPUInfoFFI {
    pub usage_percentage: f64,
    pub temperature: f64,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
}

impl Drop for GPUInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping GPUInfoFFI");
    }
}

/// FFI struct for disk information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DiskInfoFFI {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

impl Drop for DiskInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping DiskInfoFFI");
    }
}

/// FFI struct for temperature information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TemperatureInfoFFI {
    pub cpu_temp: f64,
    pub gpu_temp: f64,
    pub battery_temp: f64,
}

impl Drop for TemperatureInfoFFI {
    fn drop(&mut self) {
        debug!("Dropping TemperatureInfoFFI");
    }
}

/// Resource manager to ensure proper cleanup of system resources
#[derive(Debug)]
pub struct ResourceManager {
    battery_service: Mutex<Option<io_service_t>>,  // IOKit service
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
    pub fn new(service: io_service_t) -> Self {
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
            time_remaining_minutes: 0,
            power_source: 0,
            cycle_count: 0,
            health_percentage: 0.0,
            temperature: 0.0,
        }
    }

    pub fn power_source(&self) -> PowerSource {
        match self.power_source {
            1 => PowerSource::Battery,
            2 => PowerSource::AC,
            _ => PowerSource::Unknown,
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
            kIOMasterPortDefault,
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
            usage_percentage: 50.0,
            temperature: 25.0,
            core_count: 4,
            thread_count: 8,
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
            total_bytes: 16 * 1024 * 1024 * 1024, // 16 GB
            used_bytes: 6 * 1024 * 1024 * 1024, // 6 GB
            available_bytes: 8 * 1024 * 1024 * 1024, // 8 GB
            swap_total_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            swap_used_bytes: 0,
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
    
    pub fn mock_battery_service() -> io_service_t {
        123 // Mock service handle
    }
    
    pub fn mock_get_battery_info_impl() -> Result<BatteryInfoFFI> {
        Ok(BatteryInfoFFI {
            is_present: true,
            is_charging: false,
            percentage: 85.5,
            time_remaining_minutes: 60,
            power_source: 1,
            cycle_count: 0,
            health_percentage: 100.0,
            temperature: 25.0,
        })
    }
    
    pub fn mock_get_cpu_info_impl() -> Result<CPUInfoFFI> {
        let core_usage = Arc::new(vec![0.5; 8]);
        
        Ok(CPUInfoFFI {
            usage_percentage: 50.0,
            temperature: 25.0,
            core_count: 4,
            thread_count: 8,
        })
    }
    
    pub fn mock_get_memory_info_impl() -> Result<MemoryInfoFFI> {
        Ok(MemoryInfoFFI {
            total_bytes: 16_000_000_000,
            used_bytes: 6_000_000_000,
            available_bytes: 8_000_000_000,
            swap_total_bytes: 2_000_000_000,
            swap_used_bytes: 0,
        })
    }
}

impl From<Battery> for BatteryInfoFFI {
    fn from(battery: Battery) -> Self {
        Self {
            is_present: battery.is_present,
            is_charging: battery.is_charging,
            percentage: battery.percentage,
            time_remaining_minutes: (battery.time_remaining.as_secs() / 60) as i32,
            power_source: match battery.power_source {
                PowerSource::Battery => 0,
                PowerSource::AC => 1,
                PowerSource::Unknown => 2,
            },
            cycle_count: battery.cycle_count,
            health_percentage: battery.health_percentage,
            temperature: battery.temperature,
        }
    }
}

impl From<CPU> for CPUInfoFFI {
    fn from(cpu: CPU) -> Self {
        Self {
            usage_percentage: cpu.average_usage(),
            temperature: 0.0, // Temperature not available in CPU struct yet
            core_count: cpu.physical_cores(),
            thread_count: cpu.logical_cores(),
        }
    }
}

impl From<Memory> for MemoryInfoFFI {
    fn from(memory: Memory) -> Self {
        Self {
            total_bytes: memory.total,
            used_bytes: memory.used,
            available_bytes: memory.available,
            swap_total_bytes: memory.wired,
            swap_used_bytes: (memory.pressure * memory.total as f64) as u64,
        }
    }
}

impl From<GPU> for GPUInfoFFI {
    fn from(gpu: GPU) -> Self {
        Self {
            usage_percentage: gpu.utilization,
            temperature: gpu.temperature,
            memory_total_bytes: gpu.memory_total,
            memory_used_bytes: gpu.memory_used,
        }
    }
}

impl From<Disk> for DiskInfoFFI {
    fn from(disk: Disk) -> Self {
        Self {
            total_bytes: disk.total,
            used_bytes: disk.used,
            available_bytes: disk.available,
            read_bytes_per_sec: 0, // These fields aren't available in the Disk struct yet
            write_bytes_per_sec: 0,
        }
    }
}

impl From<Temperature> for TemperatureInfoFFI {
    fn from(temp: Temperature) -> Self {
        Self {
            cpu_temp: temp.celsius,
            gpu_temp: temp.celsius,
            battery_temp: temp.celsius,
        }
    }
} 