use std::ffi::CString;
use std::ptr;
use tokio::task;
use std::ptr::null_mut as null_ptr;
use std::os::raw::c_void;
use std::mem;

use metal::Device as MTLDevice;
use objc2::rc::autoreleasepool;
use objc2::{msg_send, runtime::AnyObject};
use async_trait::async_trait;

use crate::{
    core::{metrics::Metric, types::{Percentage, ByteSize}},
    error::{Error, Result},
    gpu::types::{GpuUtilization, GpuCharacteristics, GpuMemory},
    traits::{HardwareMonitor, UtilizationMonitor},
    utils::bindings::{IOServiceGetMatchingService, IOServiceMatching, K_IOMASTER_PORT_DEFAULT}
};

//------------------------------------------------------------------------------
// GpuCharacteristicsMonitor
//------------------------------------------------------------------------------

/// Monitor for GPU characteristics
pub struct GpuCharacteristicsMonitor {
    metal_device: Option<MTLDevice>,
}

impl GpuCharacteristicsMonitor {
    /// Create a new GPU characteristics monitor
    pub fn new(metal_device: Option<MTLDevice>) -> Self {
        Self { metal_device }
    }

    /// Get the Metal device
    pub fn get_metal_device(&self) -> Option<&MTLDevice> {
        self.metal_device.as_ref()
    }

    /// Get GPU characteristics
    pub async fn get_characteristics(&self) -> Result<GpuCharacteristics> {
        let mut characteristics = GpuCharacteristics::default();

        // Check if we have a Metal device
        if let Some(device) = self.get_metal_device() {
            // Clone the device to avoid borrowing self
            let device_clone = device.clone();

            if let Ok(metal_info) = task::spawn_blocking(move || {
                autoreleasepool(|_| unsafe {
                    // Use proper conversion instead of direct casting
                    let device_ptr = &device_clone as *const _ as *const AnyObject;
                    let device_obj = &*device_ptr;

                    // Check if it's an integrated GPU
                    let is_low_power: bool = msg_send![device_obj, isLowPower];

                    // Check for raytracing support (Metal 3 feature)
                    let supports_raytracing: bool = msg_send![device_obj, supportsRaytracing];

                    (is_low_power, supports_raytracing)
                })
            })
            .await
            {
                characteristics.is_integrated = metal_info.0;
                characteristics.has_raytracing = metal_info.1;
            }
        }

        // Check for Apple Silicon GPU
        if cfg!(target_arch = "aarch64") {
            characteristics.is_apple_silicon = true;
            // Apple Silicon GPUs should always be detected as integrated
            characteristics.is_integrated = true;
        }

        // Detect raytracing support (M2 Pro/Max/Ultra and M3 series)
        if let Some(chip_info) = self.detect_apple_silicon_chip().await {
            characteristics.has_raytracing = chip_info.contains("M2 Pro")
                || chip_info.contains("M2 Max")
                || chip_info.contains("M2 Ultra")
                || chip_info.contains("M3");

            // Estimate core count based on chip type
            characteristics.core_count = if chip_info.contains("M3 Max") {
                Some(40)
            } else if chip_info.contains("M3 Pro") {
                Some(18)
            } else if chip_info.contains("M3") {
                Some(10)
            } else if chip_info.contains("M2 Ultra") {
                Some(76)
            } else if chip_info.contains("M2 Max") {
                Some(38)
            } else if chip_info.contains("M2 Pro") {
                Some(19)
            } else if chip_info.contains("M2") {
                Some(10)
            } else if chip_info.contains("M1 Ultra") {
                Some(64)
            } else if chip_info.contains("M1 Max") {
                Some(32)
            } else if chip_info.contains("M1 Pro") {
                Some(16)
            } else if chip_info.contains("M1") {
                Some(8)
            } else {
                None
            };
        }

        Ok(characteristics)
    }

    fn is_apple_silicon(&self) -> bool {
        if let Some(device) = &self.metal_device {
            let device_name = device.name();
            device_name.contains("Apple")
        } else {
            false
        }
    }

    fn is_integrated(&self) -> bool {
        if let Some(device) = &self.metal_device {
            let device_name = device.name();
            device_name.contains("Apple") || device_name.contains("Intel")
        } else {
            false
        }
    }

    fn get_chip_info(&self) -> Result<String> {
        if let Some(device) = &self.metal_device {
            let device_name = device.name().to_string();
            Ok(device_name)
        } else {
            Ok("Unknown GPU".to_string())
        }
    }

    /// Detects the specific Apple Silicon chip model
    pub async fn detect_apple_silicon_chip(&self) -> Option<String> {
        // Clone any data needed for the task to avoid borrowing self
        task::spawn_blocking(move || {
            // Use sysctl to get hardware model
            let model = Self::get_hardware_model()?;
            Some(model)
        })
        .await
        .ok()?
    }

    /// Get hardware model using sysctl
    fn get_hardware_model() -> Option<String> {
        unsafe {
            // Use sysctl to get machine hardware model
            let mut buffer = [0u8; 256];
            let mut size = buffer.len();
            let model_name_cstring = std::ffi::CString::new("hw.model").ok()?;

            let result = libc::sysctlbyname(
                model_name_cstring.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );

            if result != 0 || size == 0 {
                return None;
            }

            // Convert model identifier to string
            let model = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8).to_string_lossy().into_owned();

            Some(model)
        }
    }

    /// Detects Intel GPU model if available
    pub async fn detect_intel_gpu(&self) -> Option<String> {
        // Try to detect Intel GPU from CPU model and other system info
        let cpu_model = self.get_cpu_model().await?;

        // Check for integrated Intel GPU based on CPU model
        if cpu_model.contains("Intel") {
            // Try to determine generation based on CPU model
            if cpu_model.contains("i9") || cpu_model.contains("i7") || cpu_model.contains("i5") {
                // Check for specific generations
                if cpu_model.contains("12th") || cpu_model.contains("11th") {
                    return Some("Intel Iris Xe Graphics".to_string());
                } else if cpu_model.contains("10th") {
                    return Some("Intel Iris Plus Graphics".to_string());
                } else if cpu_model.contains("9th") || cpu_model.contains("8th") {
                    return Some("Intel UHD Graphics 630".to_string());
                } else if cpu_model.contains("7th") {
                    return Some("Intel HD Graphics 630".to_string());
                } else if cpu_model.contains("6th") {
                    return Some("Intel HD Graphics 530".to_string());
                } else if cpu_model.contains("5th") {
                    return Some("Intel Iris Pro Graphics".to_string());
                } else if cpu_model.contains("4th") {
                    return Some("Intel Iris Graphics".to_string());
                }
            }

            // Generic fallback for Intel
            return Some("Intel Integrated Graphics".to_string());
        }

        None
    }

    /// Gets CPU model info to help with GPU identification
    pub async fn get_cpu_model(&self) -> Option<String> {
        task::spawn_blocking(|| unsafe {
            // Use sysctl to get CPU model info
            let mut buffer = [0u8; 256];
            let mut size = buffer.len();
            let model_name_cstring = std::ffi::CString::new("machdep.cpu.brand_string").ok()?;

            let result = libc::sysctlbyname(
                model_name_cstring.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );

            if result != 0 || size == 0 {
                return None;
            }

            // Convert to string
            let model = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8).to_string_lossy().into_owned();

            Some(model)
        })
        .await
        .ok()?
    }
}

//------------------------------------------------------------------------------
// GpuMemoryMonitor
//------------------------------------------------------------------------------

/// Monitor for GPU memory metrics
pub struct GpuMemoryMonitor {
    metal_device: Option<MTLDevice>,
}

impl GpuMemoryMonitor {
    /// Creates a new GpuMemoryMonitor with the provided Metal device
    pub fn new(metal_device: Option<MTLDevice>) -> Self {
        Self { metal_device }
    }

    /// Gets the current GPU memory information
    pub async fn get_memory_info(&self) -> Result<GpuMemory> {
        let total = self.get_total_memory().await?;
        let pressure = self.get_memory_pressure().await?;

        // For Apple Silicon GPUs, we can get more accurate memory info
        let (used, free) = if let Some(device) = &self.metal_device {
            let used = device.current_allocated_size();
            let free = total.saturating_sub(used);
            (used, free)
        } else {
            // For other GPUs, estimate based on pressure
            let used = (total as f64 * pressure) as u64;
            let free = total.saturating_sub(used);
            (used, free)
        };

        Ok(GpuMemory::new(total, used, free))
    }

    async fn get_total_memory(&self) -> Result<u64> {
        task::spawn_blocking(move || {
            let mut size: u64 = 0;
            let mut size_len = std::mem::size_of::<u64>();

            let name = CString::new("hw.memsize").unwrap();
            let ret = unsafe {
                libc::sysctlbyname(
                    name.as_ptr(),
                    &mut size as *mut u64 as *mut libc::c_void,
                    &mut size_len,
                    ptr::null_mut(),
                    0,
                )
            };

            if ret == 0 {
                Ok(size)
            } else {
                Err(Error::system("Failed to get system memory size"))
            }
        })
        .await?
    }

    async fn get_memory_pressure(&self) -> Result<f64> {
        task::spawn_blocking(move || {
            let mut pressure: f64 = 0.0;
            let mut pressure_len = std::mem::size_of::<f64>();

            let name = CString::new("vm.memory_pressure").unwrap();
            let ret = unsafe {
                libc::sysctlbyname(
                    name.as_ptr(),
                    &mut pressure as *mut f64 as *mut libc::c_void,
                    &mut pressure_len,
                    ptr::null_mut(),
                    0,
                )
            };

            if ret == 0 {
                Ok(pressure / 100.0) // Convert to 0.0-1.0 range
            } else {
                Err(Error::system("Failed to get memory pressure"))
            }
        })
        .await?
    }

    async fn get_metric(&self) -> Result<Metric<ByteSize>> {
        let memory_info = self.get_memory_info().await?;
        Ok(Metric::new(ByteSize::from_bytes(memory_info.used)))
    }
}

//------------------------------------------------------------------------------
// GpuTemperatureMonitor
//------------------------------------------------------------------------------

/// Monitor for GPU temperature
pub struct GpuTemperatureMonitor {
    metal_device: Option<MTLDevice>,
}

impl GpuTemperatureMonitor {
    /// Create a new GPU temperature monitor
    pub fn new(metal_device: Option<MTLDevice>) -> Self {
        Self { metal_device }
    }

    /// Get the Metal device
    pub fn get_metal_device(&self) -> Option<&MTLDevice> {
        self.metal_device.as_ref()
    }

    /// Get GPU temperature in Celsius
    pub async fn get_temperature(&self) -> Result<f32> {
        // Clone the metal_device to avoid borrowing self in the closure
        let metal_device_clone = self.metal_device.clone();

        // Use tokio's spawn_blocking for IO-heavy operations
        tokio::task::spawn_blocking(move || {
            // We can't call self.get_temperature_from_smc() here because self is not available
            // in the closure. Instead, we'll implement the SMC temperature reading directly.

            // Fall back to Metal API if available
            if let Some(_device) = &metal_device_clone {
                // Use autoreleasepool for Objective-C memory management
                autoreleasepool(|_| {
                    // TODO: Implement this
                    Ok(50.0) // Default to 50°C when actual reading is not available
                })
            } else {
                Err(Error::NotAvailable {
                    resource: "GPU temperature".to_string(),
                    reason: "No Metal device available".to_string(),
                })
            }
        })
        .await?
    }

    /// Attempt to get temperature from System Management Controller
    fn get_temperature_from_smc(&self) -> Result<f32> {
        // In a real implementation, we would use SMC keys to get the GPU temperature
        // For example, on Intel Macs: "TG0P" for GPU proximity
        // TODO: Implement this
        // This is a placeholder implementation
        Err(Error::NotAvailable {
            resource: "SMC temperature reading".to_string(),
            reason: "Not implemented".to_string(),
        })
    }
}

fn get_service_for_name(service_name: &CString) -> u32 {
    unsafe { IOServiceGetMatchingService(K_IOMASTER_PORT_DEFAULT, IOServiceMatching(service_name.as_ptr())) }
}

//------------------------------------------------------------------------------
// GpuUtilizationMonitor
//------------------------------------------------------------------------------

/// Monitor for GPU utilization
pub struct GpuUtilizationMonitor {
    metal_device: Option<MTLDevice>,
    gpu_id: usize,
}

impl GpuUtilizationMonitor {
    /// Create a new GPU utilization monitor
    pub fn new(metal_device: Option<MTLDevice>, gpu_id: usize) -> Self {
        Self { metal_device, gpu_id }
    }

    /// Get current GPU utilization
    pub async fn get_utilization(&self) -> Result<GpuUtilization> {
        // Attempt to get utilization
        let utilization_value = if let Some(_device) = &self.metal_device {
            // For now, we'll use a simpler method based on process activity
            // This is a placeholder - in the real implementation, this would
            // call the get_gpu_process_activity method
            let random_value = 50.0;
            random_value
        } else {
            // If we don't have a Metal device, use a default value 
            50.0
        };

        Ok(GpuUtilization::new(utilization_value))
    }
}

#[async_trait]
impl HardwareMonitor for GpuUtilizationMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("GPU Utilization".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("gpu{}", self.gpu_id))
    }

    async fn get_metric(&self) -> Result<Metric<Percentage>> {
        let gpu_utilization = self.get_utilization().await?;
        Ok(Metric::new(Percentage::from_f64(gpu_utilization.value)))
    }
}

#[async_trait]
impl UtilizationMonitor for GpuUtilizationMonitor {
    async fn utilization(&self) -> Result<f64> {
        let gpu_utilization = self.get_utilization().await?;
        Ok(gpu_utilization.value)
    }
}