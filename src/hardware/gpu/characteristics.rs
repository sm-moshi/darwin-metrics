use crate::error::Result;
use objc2::{msg_send, rc::autoreleasepool, runtime::AnyObject};

use super::gpu_impl::Gpu;

/// Holds information about GPU characteristics
#[derive(Debug, Clone, Default)]
pub struct GpuCharacteristics {
    /// Is this an integrated GPU (vs discrete)
    pub is_integrated: bool,
    /// Is this an Apple Silicon GPU
    pub is_apple_silicon: bool,
    /// Does this GPU have hardware raytracing support
    pub has_raytracing: bool,
    /// Core/execution unit count (if available)
    pub core_count: Option<u32>,
    /// Clock speed in MHz (if available)
    pub clock_speed_mhz: Option<u32>,
}

impl Gpu {
    /// Gets the GPU model name
    pub fn name(&self) -> Result<String> {
        // Get the GPU name from Metal with improved detection
        autoreleasepool(|_| {
            // First try Metal API for the most accurate name
            if let Some(device) = self.get_metal_device() {
                unsafe {
                    let device_obj: *mut AnyObject = device.cast();
                    let name_obj: *mut AnyObject = msg_send![device_obj, name];
                    if !name_obj.is_null() {
                        let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
                        if !utf8_string.is_null() {
                            let c_str = std::ffi::CStr::from_ptr(utf8_string as *const i8);
                            let name = c_str.to_string_lossy().into_owned();

                            // If the name is useful (not generic), return it
                            if !name.is_empty() && name != "Apple GPU" && name != "Apple Graphics" {
                                return Ok(name);
                            }
                        }
                    }
                }
            }

            // Enhanced Apple Silicon detection with chip family identification
            if cfg!(target_arch = "aarch64") {
                // Try to identify the specific Apple Silicon chip
                if let Some(chip_info) = self.detect_apple_silicon_chip() {
                    return Ok(chip_info);
                }

                // Fallback to generic description if specific chip not detected
                return Ok("Apple Silicon Integrated GPU".to_string());
            }

            // If we're on Intel, try to get more specific info from IORegistry
            if let Some(intel_gpu_info) = self.detect_intel_gpu() {
                return Ok(intel_gpu_info);
            }

            Ok("Unknown GPU".to_string())
        })
    }

    /// Detects the specific Apple Silicon chip model
    pub fn detect_apple_silicon_chip(&self) -> Option<String> {
        // Use a more maintainable approach with a mapping structure
        let model = self.get_hardware_model()?;

        // Maps model identifiers to chip families
        // Format: (model_prefix, major_version, minor_version_range) -> chip_name
        let chip_mappings = [
            // M3 series chips
            ("Mac15,", "M3 Max GPU"),
            ("Mac16,", "M3 Pro GPU"),
            ("Mac17,", "M3 GPU"),
            // M2 series chips
            ("Mac14,5", "M2 Max GPU"),
            ("Mac14,6", "M2 Max GPU"),
            ("Mac14,3", "M2 Pro GPU"),
            ("Mac14,4", "M2 Pro GPU"),
            ("Mac14,7", "M2 Ultra GPU"),
            ("Mac14,8", "M2 Ultra GPU"),
            ("Mac14,", "M2 GPU"),
            ("Mac13,", "M2 GPU"),
            // M1 series chips
            ("Mac13,1", "M1 Max GPU"),
            ("MacBookPro18,2", "M1 Max GPU"),
            ("Mac13,2", "M1 Ultra GPU"),
            ("Mac12,", "M1 Pro GPU"),
            ("Mac11,", "M1 GPU"),
            ("MacBookPro17,", "M1 GPU"),
            ("MacBookAir10,1", "M1 GPU"),
            ("Macmini9,1", "M1 GPU"),
        ];

        // Find the first matching chip mapping
        for (prefix, chip_name) in chip_mappings {
            if model.starts_with(prefix) {
                return Some(format!("Apple {}", chip_name));
            }
        }

        // Fallback for any other Apple Silicon
        if cfg!(target_arch = "aarch64") {
            return Some("Apple Silicon GPU".to_string());
        }

        None
    }

    /// Gets the hardware model identifier using sysctl
    fn get_hardware_model(&self) -> Option<String> {
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
            let model = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8)
                .to_string_lossy()
                .into_owned();

            Some(model)
        }
    }

    /// Detects Intel GPU model if available
    pub fn detect_intel_gpu(&self) -> Option<String> {
        // Try to detect Intel GPU from CPU model and other system info
        let cpu_model = self.get_cpu_model()?;

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
    pub fn get_cpu_model(&self) -> Option<String> {
        unsafe {
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
            let model = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8)
                .to_string_lossy()
                .into_owned();

            Some(model)
        }
    }

    /// Get GPU characteristics with improved hardware detection
    pub fn get_characteristics(&self) -> GpuCharacteristics {
        let mut characteristics = GpuCharacteristics::default();

        // Check if we have a Metal device
        if let Some(device) = self.get_metal_device() {
            unsafe {
                let device_obj: *mut AnyObject = device.cast();

                // Check if it's an integrated GPU
                let is_low_power: bool = msg_send![device_obj, isLowPower];
                characteristics.is_integrated = is_low_power;

                // Check for Apple Silicon GPU
                if cfg!(target_arch = "aarch64") {
                    characteristics.is_apple_silicon = true;
                }

                // Check for raytracing support (Metal 3 feature)
                let supports_raytracing: bool = msg_send![device_obj, supportsRaytracing];
                characteristics.has_raytracing = supports_raytracing;
            }

            // Detect raytracing support (M2 Pro/Max/Ultra and M3 series)
            if let Some(chip_info) = self.detect_apple_silicon_chip() {
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

                // Estimate clock speed based on chip type
                characteristics.clock_speed_mhz = if chip_info.contains("M3") {
                    Some(1398)
                } else if chip_info.contains("M2") {
                    Some(1398)
                } else if chip_info.contains("M1") {
                    Some(1278)
                } else {
                    None
                };
            }
        } else {
            // Intel Mac detection
            if let Some(gpu_name) = self.detect_intel_gpu() {
                // Integrated Intel GPU
                if gpu_name.contains("Intel") {
                    characteristics.is_integrated = true;

                    // Estimate core count based on GPU type
                    characteristics.core_count = if gpu_name.contains("Iris Xe") {
                        Some(96)
                    } else if gpu_name.contains("Iris Plus") {
                        Some(64)
                    } else if gpu_name.contains("UHD 630") {
                        Some(24)
                    } else if gpu_name.contains("HD 630") || gpu_name.contains("HD 530") {
                        Some(24)
                    } else if gpu_name.contains("Iris Pro") {
                        Some(48)
                    } else if gpu_name.contains("Iris") {
                        Some(40)
                    } else {
                        None
                    };

                    // Estimate clock speed based on GPU type
                    characteristics.clock_speed_mhz = if gpu_name.contains("Iris Xe") {
                        Some(1300)
                    } else if gpu_name.contains("Iris Plus") {
                        Some(1150)
                    } else if gpu_name.contains("UHD 630") {
                        Some(1200)
                    } else if gpu_name.contains("HD 630") {
                        Some(1100)
                    } else if gpu_name.contains("HD 530") {
                        Some(1050)
                    } else if gpu_name.contains("Iris Pro") {
                        Some(1200)
                    } else if gpu_name.contains("Iris") {
                        Some(1100)
                    } else {
                        None
                    };
                }
            } else {
                // Check for discrete AMD GPU via Metal API
                if let Some(device) = self.get_metal_device() {
                    unsafe {
                        let device_obj: *mut AnyObject = device.cast();
                        let name_obj: *mut AnyObject = msg_send![device_obj, name];
                        if !name_obj.is_null() {
                            let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
                            if !utf8_string.is_null() {
                                let name = std::ffi::CStr::from_ptr(utf8_string as *const i8)
                                    .to_string_lossy();

                                // Discrete AMD GPU detection
                                if name.contains("AMD") || name.contains("Radeon") {
                                    characteristics.is_integrated = false;

                                    // Raytracing support for newer AMD GPUs
                                    characteristics.has_raytracing = name.contains("RX 6")
                                        || name.contains("RX 7")
                                        || name.contains("Radeon Pro");
                                }
                            }
                        }
                    }
                }
            }
        }

        characteristics
    }
}
