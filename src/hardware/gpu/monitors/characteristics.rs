use crate::{error::Result, hardware::gpu::types::GpuCharacteristics};
use metal::Device as MTLDevice;
use objc2::msg_send;
use objc2::{rc::autoreleasepool, runtime::AnyObject};
use tokio::task;

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
