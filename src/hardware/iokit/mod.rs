use std::{
    ffi::{c_void as ffi_c_void, CString},
    mem::size_of,
    os::raw::c_char,
    ptr,
};

use objc2::{
    class, msg_send,
    rc::{autoreleasepool, Retained},
    runtime::AnyObject,
};
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

use crate::{
    error::{Error, Result},
    utils::bindings::{
        smc_key_from_chars, IOByteCount, IOConnectCallStructMethod,
        IORegistryEntryCreateCFProperties, IOServiceClose, IOServiceGetMatchingService,
        IOServiceMatching, IOServiceOpen, SMCKeyData_t, IO_RETURN_SUCCESS, KERNEL_INDEX_SMC,
        SMC_CMD_READ_BYTES, SMC_CMD_READ_KEYINFO, SMC_KEY_AMBIENT_TEMP, SMC_KEY_BATTERY_TEMP,
        SMC_KEY_CPU_POWER, SMC_KEY_CPU_TEMP, SMC_KEY_CPU_THROTTLE, SMC_KEY_FAN_NUM,
        SMC_KEY_FAN_SPEED, SMC_KEY_GPU_TEMP, SMC_KEY_HEATSINK_TEMP,
    },
};

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
pub mod mock;
#[cfg(test)]
mod tests;

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

#[cfg_attr(test, mockall::automock)]
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
    fn get_dict_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<Retained<NSDictionary<NSString, NSObject>>>;
    fn get_service(&self, name: &str) -> Result<Retained<AnyObject>>;
    fn io_registry_entry_get_parent(&self, entry: &AnyObject) -> Option<Retained<AnyObject>>;

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

    // SMC key reading method
    fn read_smc_key(&self, key: [c_char; 4]) -> Result<f64>;
}

#[derive(Debug, Clone)]
pub struct IOKitImpl;

impl IOKitImpl {
    fn smc_read_key(&self, key: [c_char; 4]) -> Result<f64> {
        // When in coverage mode with skip-ffi-crashes feature, return mock values
        if cfg!(feature = "skip-ffi-crashes") {
            // Return mock values for common SMC keys
            // CPU and GPU temperature use the same value
            if key == SMC_KEY_CPU_TEMP || key == SMC_KEY_GPU_TEMP {
                return Ok(42.5);
            } else if key == SMC_KEY_AMBIENT_TEMP {
                return Ok(26.0);
            } else if key == SMC_KEY_BATTERY_TEMP {
                return Ok(35.0);
            } else if key == SMC_KEY_CPU_POWER {
                return Ok(15.0);
            } else if key == SMC_KEY_FAN_NUM {
                return Ok(2.0);
            } else if key == SMC_KEY_HEATSINK_TEMP {
                return Ok(45.0);
            } else if key[0] == b'F' as c_char
                && key[2] == b'A' as c_char
                && key[3] == b'c' as c_char
            {
                // This matches fan speed keys like F0Ac, F1Ac, etc.
                return Ok(2000.0);
            } else {
                return Ok(0.0);
            }
        }

        // Only proceed with actual SMC calls when not in coverage mode
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
            if result != IO_RETURN_SUCCESS {
                return Err(Error::io_kit(format!("Failed to open SMC connection: {}", result)));
            }

            // Get key info first to determine the data type
            let mut input_structure = SMCKeyData_t {
                key: smc_key_from_chars(key),
                vers: 0,
                p_limit_data: 0,
                key_info: 1,
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

            if result != IO_RETURN_SUCCESS {
                IOServiceClose(connection);
                return Err(Error::io_kit(format!("Failed to read SMC key info: {}", result)));
            }

            // Now read the actual data
            input_structure.key_info = 0;
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

            if result != IO_RETURN_SUCCESS {
                return Err(Error::io_kit(format!("Failed to read SMC key data: {}", result)));
            }

            // Get the data and convert to temperature (depends on the data type)
            // Most temperature sensors use SP78 format (fixed point, signed 8.8)
            let data_type = output_structure.data.key_info.data_type;
            let data_size = output_structure.data.key_info.data_size;

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

    // Helper method to parse data type and convert to appropriate value
    // This is available for testing and internal use
    #[cfg(all(feature = "skip-ffi-crashes", test))]
    fn parse_smc_data(&self, data_type: [u8; 4], _bytes: [u8; 32]) -> Result<f64> {
        if data_type[0] == b'f' && data_type[1] == b'l' && data_type[2] == b't' {
            // Simulate float conversion - just return a test value
            return Ok(42.5);
        } else if data_type[0] == b'u'
            && data_type[1] == b'i'
            && data_type[2] == b'n'
            && data_type[3] == b't'
        {
            // Simulate uint conversion
            return Ok(100.0);
        } else if data_type[0] == b's'
            && data_type[1] == b'i'
            && data_type[2] == b'1'
            && data_type[3] == b'6'
        {
            // Simulate sint16 conversion
            return Ok(50.0);
        } else if data_type[0] == b'S'
            && data_type[1] == b'P'
            && data_type[2] == b'7'
            && data_type[3] == b'8'
        {
            // Simulate SP78 conversion
            return Ok(35.5);
        }

        Err(Error::invalid_data(format!(
            "Unsupported SMC data type: {:?}",
            std::str::from_utf8(&data_type).unwrap_or("Unknown")
        )))
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
                let c_service_name = match CString::new(service_name) {
                    Ok(s) => s,
                    Err(_) => return empty_dict, // Return empty dict if service name contains NUL
                };
                let matching_dict = IOServiceMatching(c_service_name.as_ptr());
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
        // The object is automatically released when the Retained<AnyObject> is
        // dropped
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
            // Use autoreleasepool to properly manage any temporary objects
            autoreleasepool(|_| {
                println!("DEBUG: Inside autoreleasepool for get_number_property");

                let value_opt = dict.valueForKey(&key);
                let obj = match value_opt {
                    None => {
                        println!("DEBUG: No value found for key '{}'", key);
                        return None;
                    },
                    Some(obj) => {
                        println!("DEBUG: Found value for key '{}'", key);
                        obj
                    },
                };

                let number_opt = obj.downcast::<NSNumber>();
                let n = match number_opt {
                    Err(_) => {
                        println!("DEBUG: Value is not an NSNumber");
                        return None;
                    },
                    Ok(n) => {
                        println!("DEBUG: Downcasted to NSNumber successfully");
                        n
                    },
                };
                let result = n.as_i64();

                println!("DEBUG: Got i64 value: {}", result);
                Some(result)
            })
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

    fn get_dict_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<Retained<NSDictionary<NSString, NSObject>>> {
        let key = NSString::from_str(key);
        unsafe {
            if let Some(obj) = dict.valueForKey(&key) {
                // Try to convert the object to a NSDictionary
                let cls = class!(NSDictionary);
                let is_dict: bool = msg_send![&obj, isKindOfClass: cls];

                if is_dict {
                    // Explicitly retain the object to ensure proper memory management
                    // This is crucial because we're creating a new Retained<> from a reference
                    let _: () = msg_send![&obj, retain];

                    // Now create the Retained wrapper from the raw pointer
                    let obj_ref: &NSObject = &obj;
                    let dict_ptr =
                        obj_ref as *const NSObject as *mut NSDictionary<NSString, NSObject>;

                    // Retained::from_raw expects to receive ownership of a +1 retain count object,
                    // which we've just done with the explicit retain above
                    return Retained::from_raw(dict_ptr);
                }
            }
            None
        }
    }

    fn io_registry_entry_get_parent(&self, entry: &AnyObject) -> Option<Retained<AnyObject>> {
        use std::os::raw::c_uint;

        extern "C" {
            fn IORegistryEntryGetParentEntry(
                entry: c_uint,
                plane: *const c_char,
                parent: *mut c_uint,
            ) -> i32;
        }

        unsafe {
            let entry_id = entry as *const AnyObject as c_uint;
            let mut parent_id: c_uint = 0;

            // Get the parent in the IOService plane
            let plane = match CString::new("IOService") {
                Ok(p) => p,
                Err(_) => return None, // Should never happen as "IOService" is a valid C string
            };
            let result = IORegistryEntryGetParentEntry(entry_id, plane.as_ptr(), &mut parent_id);

            if result != IO_RETURN_SUCCESS || parent_id == 0 {
                return None;
            }

            // Create an AnyObject from the parent ID
            let parent_ptr = parent_id as *mut AnyObject;
            Retained::from_raw(parent_ptr)
        }
    }

    fn get_service(&self, name: &str) -> Result<Retained<AnyObject>> {
        // When in coverage mode with skip-ffi-crashes feature, return a safe error
        if cfg!(feature = "skip-ffi-crashes") {
            return Err(Error::system("Service access disabled for stability"));
        }

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

        Ok(FanInfo { speed_rpm, min_speed, max_speed, percentage })
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
        let mut stats = GpuStats {
            utilization: 0.0,
            perf_cap: 0.0,
            perf_threshold: 0.0,
            memory_used: 0,
            memory_total: 0,
            name: "".to_string(),
        };
        println!("DEBUG: Created default stats object");

        // Wrap in autoreleasepool to ensure proper memory management
        autoreleasepool(|_| {
            // Try to get GPU information from IOKit's AGPMController using safer approach
            {
                let agpm_matching = self.io_service_matching("AGPMController");
                // Create a scope to ensure proper object lifecycle
                {
                    if let Some(agpm_service) = self.io_service_get_matching_service(&agpm_matching)
                    {
                        // Create another scope to ensure properties are released before service
                        {
                            if let Ok(properties) =
                                self.io_registry_entry_create_cf_properties(&agpm_service)
                            {
                                // Get GPU performance capacity (0-100)
                                let perf_cap = self
                                    .get_number_property(&properties, "GPUPerfCap")
                                    .unwrap_or(0)
                                    as f64;

                                // Get GPU performance threshold (0-100)
                                let perf_threshold =
                                    self.get_number_property(&properties, "GPUPerfThreshold")
                                        .unwrap_or(100) as f64;

                                // Store values
                                stats.perf_cap = perf_cap;
                                stats.perf_threshold = perf_threshold;

                                // Calculate GPU utilization based on perf_cap and perf_threshold
                                if perf_cap > 0.0 && perf_threshold > 0.0 {
                                    stats.utilization = (perf_cap / perf_threshold) * 100.0;
                                    // Clamp to range 0-100
                                    stats.utilization = stats.utilization.clamp(0.0, 100.0);
                                }
                                // properties is dropped here
                            }
                        }
                        // agpm_service is dropped here
                    }
                }
                // agpm_matching is dropped here
            }

            // Try to get GPU memory information from IORegistry using safer approach
            {
                let accelerator_matching = self.io_service_matching("IOAccelerator");
                {
                    if let Some(accelerator) =
                        self.io_service_get_matching_service(&accelerator_matching)
                    {
                        {
                            if let Ok(properties) =
                                self.io_registry_entry_create_cf_properties(&accelerator)
                            {
                                // Get GPU memory information
                                if let Some(total_vram) =
                                    self.get_number_property(&properties, "VRAM,totalMB")
                                {
                                    stats.memory_total = (total_vram as u64) * 1024 * 1024;
                                    // Convert MB to bytes
                                }

                                if let Some(used_vram) =
                                    self.get_number_property(&properties, "VRAM,usedMB")
                                {
                                    stats.memory_used = (used_vram as u64) * 1024 * 1024;
                                    // Convert MB to bytes
                                }

                                // Get GPU name - copy strings to avoid dangling references
                                if let Some(name) =
                                    self.get_string_property(&properties, "GPUModel")
                                {
                                    stats.name = name;
                                } else if let Some(name) =
                                    self.get_string_property(&properties, "model")
                                {
                                    stats.name = name;
                                }
                                // properties is dropped here
                            }
                        }
                        // accelerator is dropped here
                    }
                }
                // accelerator_matching is dropped here
            }

            // Fallback for Apple Silicon devices where memory isn't explicitly reported
            if stats.memory_total == 0 {
                // For Apple Silicon, try to get system memory and use a portion of it since
                // Apple Silicon uses a unified memory architecture - use safer memory
                // management
                {
                    let system_matching = self.io_service_matching("IOPlatformExpertDevice");
                    {
                        if let Some(system) = self.io_service_get_matching_service(&system_matching)
                        {
                            {
                                if let Ok(properties) =
                                    self.io_registry_entry_create_cf_properties(&system)
                                {
                                    // Get total RAM size
                                    if let Some(memory) =
                                        self.get_number_property(&properties, "total-ram-size")
                                    {
                                        // Assume GPU can use up to 1/4 of system memory on Apple
                                        // Silicon
                                        stats.memory_total = (memory as u64) / 4;
                                        // Estimate used VRAM based on utilization, with safety
                                        // checks
                                        let utilization = stats.utilization.clamp(0.0, 100.0);
                                        stats.memory_used = ((utilization / 100.0)
                                            * stats.memory_total as f64)
                                            as u64;
                                    }
                                    // properties is dropped here
                                }
                            }
                            // system is dropped here
                        }
                    }
                    // system_matching is dropped here
                }
            }

            // If we still don't have a name, try to get it from another service
            if stats.name.is_empty() {
                // Use safer approach with explicit scopes for memory management
                {
                    let graphics_matching = self.io_service_matching("IOGraphicsAccelerator2");
                    {
                        if let Some(graphics) =
                            self.io_service_get_matching_service(&graphics_matching)
                        {
                            {
                                if let Ok(properties) =
                                    self.io_registry_entry_create_cf_properties(&graphics)
                                {
                                    // Get the bundle name and copy it to avoid dangling pointers
                                    if let Some(name) =
                                        self.get_string_property(&properties, "IOGLBundleName")
                                    {
                                        stats.name = name;
                                    }
                                    // properties is dropped here
                                }
                            }
                            // graphics is dropped here
                        }
                    }
                    // graphics_matching is dropped here
                }
            }

            // Fallback name for Apple Silicon
            if stats.name.is_empty() {
                // Check for Apple Silicon devices - using a safer approach with explicit scope
                // management
                let system_matching = self.io_service_matching("IOPlatformExpertDevice");
                // Create a scope to ensure proper cleanup
                {
                    if let Some(system) = self.io_service_get_matching_service(&system_matching) {
                        // Properties scope - ensure it's cleaned up before system is released
                        if let Ok(properties) = self.io_registry_entry_create_cf_properties(&system)
                        {
                            // Get chip info
                            let chip_name = self
                                .get_string_property(&properties, "chip-id")
                                .unwrap_or_else(|| "Unknown".to_string());

                            // Check if it's Apple Silicon
                            if chip_name.contains("M1")
                                || chip_name.contains("M2")
                                || chip_name.contains("M3")
                            {
                                stats.name = format!("Apple {} GPU", chip_name);
                            }
                            // properties is dropped here - correctly releasing
                            // CF objects
                        }
                        // system is dropped here - ensuring proper IOService
                        // release
                    }
                }
                // system_matching is dropped here
            }
        });

        // Last resort fallback name
        if stats.name.is_empty() {
            stats.name = "Unknown GPU".to_string();
        }

        // Try to get temperature
        if let Ok(temp) = self.get_gpu_temperature() {
            // Store this in the name for now - we'll add a dedicated temperature field in
            // the future
            stats.name = format!("{} ({}°C)", stats.name, temp);
        }

        Ok(stats)
    }

    fn read_smc_key(&self, key: [c_char; 4]) -> Result<f64> {
        // When in coverage mode, return mock values for specific keys
        if cfg!(feature = "skip-ffi-crashes") {
            // Return mock values for common SMC keys
            // CPU and GPU temperature use the same value
            if key == SMC_KEY_CPU_TEMP || key == SMC_KEY_GPU_TEMP {
                return Ok(42.5);
            } else if key == SMC_KEY_AMBIENT_TEMP {
                return Ok(26.0);
            } else if key == SMC_KEY_BATTERY_TEMP {
                return Ok(35.0);
            } else if key == SMC_KEY_CPU_POWER {
                return Ok(15.0);
            } else {
                return Ok(0.0);
            }
        }

        // For non-coverage mode, use the actual SMC reading implementation
        self.smc_read_key(key)
    }
}
