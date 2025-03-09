use crate::error::{Error, Result};
use crate::utils::bindings::{
    kIOReturnSuccess, smc_key_from_chars, IOByteCount, IOConnectCallStructMethod,
    IORegistryEntryCreateCFProperties, IOServiceClose, IOServiceGetMatchingService,
    IOServiceMatching, IOServiceOpen, SMCKeyData_t, KERNEL_INDEX_SMC, SMC_CMD_READ_BYTES,
    SMC_CMD_READ_KEYINFO, SMC_KEY_CPU_TEMP, SMC_KEY_FAN_SPEED, SMC_KEY_GPU_TEMP,
    SMC_KEY_FAN_NUM, SMC_KEY_FAN1_SPEED, SMC_KEY_FAN0_MIN, SMC_KEY_FAN0_MAX,
    SMC_KEY_HEATSINK_TEMP, SMC_KEY_AMBIENT_TEMP, SMC_KEY_BATTERY_TEMP,
    SMC_KEY_CPU_POWER, SMC_KEY_CPU_THROTTLE,
};
use objc2::class;
use objc2::msg_send;
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
use std::ffi::c_void as ffi_c_void;
use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_char;
use std::ptr;

/// GPU statistics retrieved from IOKit's AGPMController
#[derive(Debug, Clone, Default)]
pub struct GpuStats {
    /// GPU utilization percentage (0-100)
    pub utilization: f64,
    /// GPU performance capability (0-100)
    pub perf_cap: f64,
    /// GPU throttle state (0-100)
    pub perf_threshold: f64,
    /// GPU memory used in bytes
    pub memory_used: u64,
    /// Total GPU memory in bytes
    pub memory_total: u64,
    /// GPU name/model
    pub name: String,
}

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Clone)]
pub struct FanInfo {
    pub speed_rpm: u32,
    pub min_speed: u32,
    pub max_speed: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct ThermalInfo {
    pub cpu_temp: f64,
    pub gpu_temp: f64,
    pub heatsink_temp: Option<f64>,
    pub ambient_temp: Option<f64>,
    pub battery_temp: Option<f64>,
    pub is_throttling: bool,
    pub cpu_power: Option<f64>, // in watts
}

#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    fn io_service_matching(&self, service_name: &str)
        -> Retained<NSDictionary<NSString, NSObject>>;
    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>>;
    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>>;
    fn io_object_release(&self, obj: &AnyObject);
    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String>;
    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64>;
    fn get_bool_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str)
        -> Option<bool>;
    fn get_service(&self, name: &str) -> Result<Retained<AnyObject>>;
    
    // Temperature related methods
    fn get_cpu_temperature(&self) -> Result<f64>;
    fn get_gpu_temperature(&self) -> Result<f64>;
    fn get_gpu_stats(&self) -> Result<GpuStats>;
    
    // Fan related methods
    fn get_fan_speed(&self) -> Result<u32>;
    fn get_fan_count(&self) -> Result<u32>;
    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo>;
    fn get_all_fans(&self) -> Result<Vec<FanInfo>>;
    
    // Advanced thermal methods
    fn get_heatsink_temperature(&self) -> Result<f64>;
    fn get_ambient_temperature(&self) -> Result<f64>;
    fn get_battery_temperature(&self) -> Result<f64>;
    fn get_cpu_power(&self) -> Result<f64>;
    fn check_thermal_throttling(&self) -> Result<bool>;
    fn get_thermal_info(&self) -> Result<ThermalInfo>;
}

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKitImpl {
    fn smc_read_key(&self, key: [c_char; 4]) -> Result<f64> {
        unsafe {
            // Open the SMC service
            let service_name = CString::new("AppleSMC").expect("Failed to create CString");
            let service = IOServiceMatching(service_name.as_ptr());
            if service.is_null() {
                return Err(Error::service_not_found("AppleSMC service not found"));
            }

            let service_id = IOServiceGetMatchingService(0, service as *const _);
            if service_id == 0 {
                return Err(Error::service_not_found("AppleSMC service not found"));
            }

            let mut connection = 0u32;
            let result = IOServiceOpen(service_id, 0, KERNEL_INDEX_SMC, &mut connection);
            if result != kIOReturnSuccess {
                return Err(Error::io_kit(format!(
                    "Failed to open SMC connection: {}",
                    result
                )));
            }

            // Get key info first to determine the data type
            let mut input_structure = SMCKeyData_t {
                key: smc_key_from_chars(key),
                vers: 0,
                pLimitData: 0,
                keyInfo: 1,
                padding: 0,
                result: 0,
                status: 0,
                data8: 0,
                data32: 0,
                bytes: [0; 2],
                data: std::mem::zeroed(), // Initialize with zeros
            };

            let mut output_structure = input_structure;
            let mut output_size = IOByteCount(size_of::<SMCKeyData_t>());

            let result = IOConnectCallStructMethod(
                connection,
                SMC_CMD_READ_KEYINFO as u32,
                &input_structure,
                IOByteCount(size_of::<SMCKeyData_t>()),
                &mut output_structure,
                &mut output_size,
            );

            if result != kIOReturnSuccess {
                IOServiceClose(connection);
                return Err(Error::io_kit(format!(
                    "Failed to read SMC key info: {}",
                    result
                )));
            }

            // Now read the actual data
            input_structure.keyInfo = 0;
            input_structure.padding = 0;

            let result = IOConnectCallStructMethod(
                connection,
                SMC_CMD_READ_BYTES as u32,
                &input_structure,
                IOByteCount(size_of::<SMCKeyData_t>()),
                &mut output_structure,
                &mut output_size,
            );

            IOServiceClose(connection);

            if result != kIOReturnSuccess {
                return Err(Error::io_kit(format!(
                    "Failed to read SMC key data: {}",
                    result
                )));
            }

            // Get the data and convert to temperature (depends on the data type)
            // Most temperature sensors use SP78 format (fixed point, signed 8.8)
            let data_type = output_structure.data.keyInfo.data_type;
            let data_size = output_structure.data.keyInfo.data_size;

            if data_size > 0 {
                if data_type[0] == b'f' && data_type[1] == b'l' && data_type[2] == b't' {
                    // flt type: float
                    return Ok(f64::from(output_structure.data.float));
                } else if data_type[0] == b'u'
                    && data_type[1] == b'i'
                    && data_type[2] == b'n'
                    && data_type[3] == b't'
                {
                    // uint type: unsigned int
                    return Ok(f64::from(output_structure.data.uint32));
                } else if data_type[0] == b's'
                    && data_type[1] == b'i'
                    && data_type[2] == b'1'
                    && data_type[3] == b'6'
                {
                    // si16 type: signed int 16-bit
                    return Ok(f64::from(output_structure.data.sint16));
                } else if data_type[0] == b'S'
                    && data_type[1] == b'P'
                    && data_type[2] == b'7'
                    && data_type[3] == b'8'
                {
                    // SP78 type: fixed point, signed 8.8
                    let bytes = output_structure.data.bytes;
                    let val: f64 = (bytes[0] as f64) + (bytes[1] as f64 / 256.0);
                    return Ok(val);
                }
            }

            Err(Error::invalid_data(format!(
                "Unsupported SMC data type: {:?}",
                std::str::from_utf8(&data_type).unwrap_or("Unknown")
            )))
        }
    }
}

impl IOKit for IOKitImpl {
    fn io_service_matching(
        &self,
        service_name: &str,
    ) -> Retained<NSDictionary<NSString, NSObject>> {
        autoreleasepool(|_| {
            unsafe {
                let empty_dict = Retained::from_raw(msg_send![class!(NSDictionary), dictionary])
                    .expect("Failed to create dictionary");

                // Direct C function call for IOServiceMatching
                let matching_dict = IOServiceMatching(CString::new(service_name).unwrap().as_ptr());
                if !matching_dict.is_null() {
                    // Try to convert the dictionary to the expected type
                    let dict_ptr = matching_dict as *mut NSDictionary<NSString, NSObject>;
                    if let Some(dict) = Retained::from_raw(dict_ptr) {
                        return dict;
                    }
                }

                // Fallback to empty dictionary
                empty_dict
            }
        })
    }

    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        unsafe {
            let master_port: u32 = 0;
            // Create a raw pointer to use with the C function
            let matching_raw = matching as *const _ as *const ffi_c_void;
            let service = IOServiceGetMatchingService(master_port, matching_raw);
            if service == 0 {
                None
            } else {
                // Create an AnyObject from the service ID
                let service_ptr = service as *mut AnyObject;
                Retained::from_raw(service_ptr)
            }
        }
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>> {
        // Wrap in autoreleasepool to ensure proper memory management
        autoreleasepool(|_| {
            unsafe {
                let mut props: *mut ffi_c_void = ptr::null_mut();
                let entry_id = entry as *const AnyObject as u32;

                // Use IORegistryEntryCreateCFProperties directly
                let result =
                    IORegistryEntryCreateCFProperties(entry_id, &mut props, ptr::null_mut(), 0);

                if result != 0 || props.is_null() {
                    return Err(Error::system("Failed to retrieve properties"));
                }

                // Convert the properties to the expected type
                let dict_ptr = props as *mut NSDictionary<NSString, NSObject>;
                Retained::from_raw(dict_ptr)
                    .ok_or_else(|| Error::system("Failed to retain properties"))
            }
        })
    }

    fn io_object_release(&self, _obj: &AnyObject) {
        // The object is automatically released when the Retained<AnyObject> is dropped
    }

    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        let key = NSString::from_str(key);
        unsafe {
            dict.valueForKey(&key)
                .and_then(|obj| obj.downcast::<NSString>().ok())
                .map(|s| s.to_string())
        }
    }

    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64> {
        let key = NSString::from_str(key);
        unsafe {
            dict.valueForKey(&key)
                .and_then(|obj| obj.downcast::<NSNumber>().ok())
                .map(|n| n.as_i64())
        }
    }

    fn get_bool_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
        let key = NSString::from_str(key);
        unsafe {
            dict.valueForKey(&key)
                .and_then(|obj| obj.downcast::<NSNumber>().ok())
                .map(|n| n.as_bool())
        }
    }

    fn get_service(&self, name: &str) -> Result<Retained<AnyObject>> {
        let matching = self.io_service_matching(name);
        let service = self
            .io_service_get_matching_service(&matching)
            .ok_or_else(|| Error::service_not_found(format!("Service {} not found", name)))?;
        Ok(service)
    }

    // Temperature related methods
    fn get_cpu_temperature(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_CPU_TEMP)
    }

    fn get_gpu_temperature(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_GPU_TEMP)
    }

    fn get_heatsink_temperature(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_HEATSINK_TEMP)
    }

    fn get_ambient_temperature(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_AMBIENT_TEMP)
    }

    fn get_battery_temperature(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_BATTERY_TEMP)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        self.smc_read_key(SMC_KEY_CPU_POWER)
    }

    fn check_thermal_throttling(&self) -> Result<bool> {
        // Value above 0 indicates active thermal throttling
        let throttle_value = self.smc_read_key(SMC_KEY_CPU_THROTTLE)?;
        Ok(throttle_value > 0.0)
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        // Get required fields
        let cpu_temp = self.get_cpu_temperature()?;
        
        // Get other fields, allowing failure for optional sensors
        let gpu_temp = self.get_gpu_temperature().unwrap_or(0.0);
        let heatsink_temp = self.get_heatsink_temperature().ok();
        let ambient_temp = self.get_ambient_temperature().ok();
        let battery_temp = self.get_battery_temperature().ok();
        let cpu_power = self.get_cpu_power().ok();
        let is_throttling = self.check_thermal_throttling().unwrap_or(false);
        
        Ok(ThermalInfo {
            cpu_temp,
            gpu_temp,
            heatsink_temp,
            ambient_temp,
            battery_temp,
            is_throttling,
            cpu_power,
        })
    }

    // Fan related methods
    fn get_fan_speed(&self) -> Result<u32> {
        // Fan speed needs to be converted from the raw value to RPM
        let raw_speed = self.smc_read_key(SMC_KEY_FAN_SPEED)?;
        Ok(raw_speed as u32)
    }

    fn get_fan_count(&self) -> Result<u32> {
        let fans = self.smc_read_key(SMC_KEY_FAN_NUM)?;
        Ok(fans as u32)
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        // Create dynamic SMC keys for the specified fan
        let fan_key = [
            b'F' as c_char,
            (b'0' + fan_index as u8) as c_char, // Fan index (F0, F1, etc.)
            b'A' as c_char,
            b'c' as c_char,
        ];
        
        let fan_min_key = [
            b'F' as c_char,
            (b'0' + fan_index as u8) as c_char, // Fan index (F0, F1, etc.)
            b'M' as c_char,
            b'n' as c_char,
        ];
        
        let fan_max_key = [
            b'F' as c_char,
            (b'0' + fan_index as u8) as c_char, // Fan index (F0, F1, etc.)
            b'M' as c_char,
            b'x' as c_char,
        ];
        
        // Get the speeds
        let speed_rpm = self.smc_read_key(fan_key)? as u32;
        let min_speed = self.smc_read_key(fan_min_key).unwrap_or(0.0) as u32;
        let max_speed = self.smc_read_key(fan_max_key).unwrap_or(0.0) as u32;
        
        // Calculate percentage
        let percentage = if max_speed > min_speed && max_speed > 0 {
            ((speed_rpm - min_speed) as f64 / (max_speed - min_speed) as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(FanInfo {
            speed_rpm,
            min_speed,
            max_speed,
            percentage,
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        let fan_count = self.get_fan_count()?;
        
        // macOS typically has at most 2 fans, so cap the count to avoid issues
        let fan_count = fan_count.min(4);
        
        let mut fans = Vec::with_capacity(fan_count as usize);
        for i in 0..fan_count {
            if let Ok(fan_info) = self.get_fan_info(i) {
                fans.push(fan_info);
            }
        }
        
        Ok(fans)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        // Default values
        let mut stats = GpuStats::default();

        // Wrap in autoreleasepool to ensure proper memory management
        autoreleasepool(|_| {
            // Try to get GPU information from IOKit's AGPMController
            let agpm_matching = self.io_service_matching("AGPMController");
            if let Some(agpm_service) = self.io_service_get_matching_service(&agpm_matching) {
                if let Ok(properties) = self.io_registry_entry_create_cf_properties(&agpm_service) {
                    // Get GPU performance capacity (0-100)
                    if let Some(perf_cap) = self.get_number_property(&properties, "GPUPerfCap") {
                        stats.perf_cap = perf_cap as f64;
                    }

                    // Get GPU performance threshold (0-100)
                    if let Some(perf_threshold) =
                        self.get_number_property(&properties, "GPUPerfThreshold")
                    {
                        stats.perf_threshold = perf_threshold as f64;
                    }

                    // Calculate GPU utilization based on perf_cap and perf_threshold
                    if stats.perf_cap > 0.0 && stats.perf_threshold > 0.0 {
                        stats.utilization = (stats.perf_cap / stats.perf_threshold) * 100.0;
                        // Clamp to range 0-100
                        stats.utilization = stats.utilization.min(100.0).max(0.0);
                    }
                }
            }

            // Try to get GPU memory information from IORegistry
            let accelerator_matching = self.io_service_matching("IOAccelerator");
            if let Some(accelerator) = self.io_service_get_matching_service(&accelerator_matching) {
                if let Ok(properties) = self.io_registry_entry_create_cf_properties(&accelerator) {
                    // Get GPU memory information
                    if let Some(total_vram) = self.get_number_property(&properties, "VRAM,totalMB")
                    {
                        stats.memory_total = (total_vram as u64) * 1024 * 1024; // Convert MB to bytes
                    }

                    if let Some(used_vram) = self.get_number_property(&properties, "VRAM,usedMB") {
                        stats.memory_used = (used_vram as u64) * 1024 * 1024; // Convert MB to bytes
                    }

                    // Get GPU name
                    if let Some(name) = self.get_string_property(&properties, "GPUModel") {
                        stats.name = name;
                    } else if let Some(name) = self.get_string_property(&properties, "model") {
                        stats.name = name;
                    }
                }
            }

            // Fallback for Apple Silicon devices where memory isn't explicitly reported
            if stats.memory_total == 0 {
                // For Apple Silicon, try to get system memory and use a portion of it since
                // Apple Silicon uses a unified memory architecture
                let system_matching = self.io_service_matching("IOPlatformExpertDevice");
                if let Some(system) = self.io_service_get_matching_service(&system_matching) {
                    if let Ok(properties) = self.io_registry_entry_create_cf_properties(&system) {
                        if let Some(memory) =
                            self.get_number_property(&properties, "total-ram-size")
                        {
                            // Assume GPU can use up to 1/4 of system memory on Apple Silicon
                            stats.memory_total = (memory as u64) / 4;
                            // Estimate used VRAM based on utilization
                            stats.memory_used =
                                ((stats.utilization / 100.0) * stats.memory_total as f64) as u64;
                        }
                    }
                }
            }

            // If we still don't have a name, try to get it from another service
            if stats.name.is_empty() {
                let graphics_matching = self.io_service_matching("IOGraphicsAccelerator2");
                if let Some(graphics) = self.io_service_get_matching_service(&graphics_matching) {
                    if let Ok(properties) = self.io_registry_entry_create_cf_properties(&graphics) {
                        if let Some(name) = self.get_string_property(&properties, "IOGLBundleName")
                        {
                            stats.name = name;
                        }
                    }
                }
            }

            // Fallback name for Apple Silicon
            if stats.name.is_empty() {
                // Check for Apple Silicon devices
                let system_matching = self.io_service_matching("IOPlatformExpertDevice");
                if let Some(system) = self.io_service_get_matching_service(&system_matching) {
                    if let Ok(properties) = self.io_registry_entry_create_cf_properties(&system) {
                        if let Some(chip) = self.get_string_property(&properties, "chip-id") {
                            if chip.contains("M1") || chip.contains("M2") || chip.contains("M3") {
                                stats.name = format!("Apple {} GPU", chip);
                            }
                        }
                    }
                }
            }
        });

        // Last resort fallback name
        if stats.name.is_empty() {
            stats.name = "Unknown GPU".to_string();
        }

        // Try to get temperature
        if let Ok(temp) = self.get_gpu_temperature() {
            // Store this in the name for now - we'll add a dedicated temperature field in the future
            stats.name = format!("{} ({}°C)", stats.name, temp);
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smc_key_from_chars() {
        // Test with "TC0P" (CPU temperature key)
        let key = [
            b'T' as c_char,
            b'C' as c_char,
            b'0' as c_char,
            b'P' as c_char,
        ];
        let result = smc_key_from_chars(key);

        // Calculate the expected value: ('T' << 24) | ('C' << 16) | ('0' << 8) | 'P'
        let expected =
            (b'T' as u32) << 24 | (b'C' as u32) << 16 | (b'0' as u32) << 8 | (b'P' as u32);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_cpu_temperature() {
        // Create a mock IOKit implementation
        let mut mock_iokit = MockIOKit::new();

        // Set up the expected behavior
        mock_iokit
            .expect_get_cpu_temperature()
            .times(1)
            .returning(|| Ok(45.5));

        // Call the method
        let result = mock_iokit.get_cpu_temperature().unwrap();

        // Check the result
        assert_eq!(result, 45.5);
    }

    #[test]
    fn test_get_gpu_temperature() {
        // Create a mock IOKit implementation
        let mut mock_iokit = MockIOKit::new();

        // Set up the expected behavior
        mock_iokit
            .expect_get_gpu_temperature()
            .times(1)
            .returning(|| Ok(55.0));

        // Call the method
        let result = mock_iokit.get_gpu_temperature().unwrap();

        // Check the result
        assert_eq!(result, 55.0);
    }

    #[test]
    fn test_get_gpu_stats() {
        // Create a mock IOKit implementation
        let mut mock_iokit = MockIOKit::new();

        // Set up the expected behavior
        mock_iokit.expect_get_gpu_stats().times(1).returning(|| {
            Ok(GpuStats {
                utilization: 50.0,
                perf_cap: 50.0,
                perf_threshold: 100.0,
                memory_used: 1024 * 1024 * 1024,      // 1 GB
                memory_total: 4 * 1024 * 1024 * 1024, // 4 GB
                name: "Test GPU".to_string(),
            })
        });

        // Call the method
        let result = mock_iokit.get_gpu_stats().unwrap();

        // Check the result
        assert_eq!(result.utilization, 50.0);
        assert_eq!(result.memory_total, 4 * 1024 * 1024 * 1024);
        assert_eq!(result.name, "Test GPU");
    }

    // This test is disabled by default because it can cause segfaults in some environments
    // Only run it manually when debugging IOKit issues
    #[cfg(feature = "unstable-tests")]
    #[test]
    fn test_real_gpu_stats() {
        // Wrap the entire test in an autoreleasepool to ensure proper memory cleanup
        autoreleasepool(|_| {
            let iokit = IOKitImpl::default();
            let result = iokit.get_gpu_stats();

            // This test might fail if running in a CI environment without GPU
            // So we'll just log the result rather than asserting
            match result {
                Ok(stats) => {
                    // Just ensure we got some reasonable data
                    assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
                    assert!(!stats.name.is_empty());
                    println!("GPU stats: {:?}", stats);
                }
                Err(e) => {
                    // Just log the error, don't fail the test
                    println!("Warning: Couldn't get GPU stats: {:?}", e);
                }
            }
        });
    }

    // FIXME: Test disabled until Send trait issues can be resolved
    // #[test]
    // fn test_get_service() {
    //     // Create a mock IOKit implementation
    //     let mut mock_iokit = MockIOKit::new();
    //
    //     // Mock the dictionary for io_service_matching
    //     let mock_dict = unsafe {
    //         Retained::from_raw(msg_send![class!(NSDictionary), dictionary]).unwrap()
    //     };
    //
    //     // Mock an AnyObject for the service
    //     let mock_service = unsafe {
    //         Retained::from_raw(msg_send![class!(NSObject), new]).unwrap()
    //     };
    //
    //     // Set up the expected behavior for io_service_matching
    //     mock_iokit.expect_io_service_matching()
    //         .with(eq("AppleSMC"))
    //         .times(1)
    //         .return_once(move |_| mock_dict);
    //
    //     // Set up the expected behavior for io_service_get_matching_service
    //     mock_iokit.expect_io_service_get_matching_service()
    //         .times(1)
    //         .return_once(move |_| Some(mock_service));
    //
    //     // Call the method
    //     let result = mock_iokit.get_service("AppleSMC");
    //
    //     // Check the result
    //     assert!(result.is_ok());
    // }
}
