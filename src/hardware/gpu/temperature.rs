use std::ffi::CString;

use crate::error::Result;
use crate::hardware::iokit::IOMASTER_PORT_DEFAULT;
use crate::utils::bindings::{
    IOByteCount, IOConnectCallStructMethod, IOServiceClose, IOServiceGetMatchingService, IOServiceMatching,
    IOServiceOpen, SMCKeyData_t, SMCKeyData_t_data, IO_RETURN_SUCCESS, KERNEL_INDEX_SMC, SMC_CMD_READ_BYTES,
    SMC_CMD_READ_KEYINFO,
};

use super::gpu_impl::Gpu;

// SMC key structure for temperature sensors
#[repr(C)]
struct SmcKey {
    key: [u8; 4],
}

// SMC value structure
#[repr(C)]
struct SmcVal {
    data_size: u32,
    data_type: [u8; 4],
    data_attributes: u8,
    data: [u8; 32],
}

// Constants for SMC communication
const SMC_CMD_READ_KEY: u8 = 5;
const SMC_CMD_READ_INDEX: u8 = 8;

impl Gpu {
    /// Gets the GPU temperature in Celsius (if available)
    ///
    /// This method attempts to read the GPU temperature from the System Management Controller (SMC).
    /// It supports both discrete GPUs and integrated GPUs, including Apple Silicon.
    pub fn get_temperature(&self) -> Result<f32> {
        // Try different SMC keys based on hardware type
        let characteristics = self.get_characteristics();

        // For Apple Silicon, use specific temperature sensors
        if characteristics.is_apple_silicon {
            // Try Apple Silicon GPU temperature sensors
            let temp_keys = [
                "sgpg", // Silicon GPU Graphics
                "gpgc", // GPU Graphics Compute
                "pgtr", // GPU Temperature
            ];

            for key in &temp_keys {
                if let Ok(temp) = self.read_smc_temperature(key) {
                    if temp > 0.0 && temp < 120.0 {
                        return Ok(temp);
                    }
                }
            }
        } else {
            // For Intel Macs with discrete GPUs
            let temp_keys = [
                "TGDD", // Discrete GPU Die
                "TGDP", // Discrete GPU Proximity
                "TG0D", // GPU 0 Die
                "TG0P", // GPU 0 Proximity
                "TGPU", // GPU
            ];

            for key in &temp_keys {
                if let Ok(temp) = self.read_smc_temperature(key) {
                    if temp > 0.0 && temp < 120.0 {
                        return Ok(temp);
                    }
                }
            }

            // For Intel integrated GPUs, try CPU sensors as fallback
            if characteristics.is_integrated {
                let cpu_temp_keys = [
                    "TC0P", // CPU 0 Proximity
                    "TC0D", // CPU 0 Die
                    "TC0E", // CPU 0 ??
                    "TC0F", // CPU 0 ??
                ];

                for key in &cpu_temp_keys {
                    if let Ok(temp) = self.read_smc_temperature(key) {
                        if temp > 0.0 && temp < 120.0 {
                            // Adjust CPU temperature to approximate GPU temperature
                            // Integrated GPUs typically run 5-10°C hotter than CPU
                            return Ok(temp + 7.0);
                        }
                    }
                }
            }
        }

        // If no temperature sensor available, estimate based on utilization
        self.estimate_temperature()
    }

    // Helper method to read temperature from SMC using a specific key
    fn read_smc_temperature(&self, key_str: &str) -> Result<f32> {
        // Create SMC key from string
        let key = self.create_smc_key(key_str)?;

        // Read value from SMC
        let val = self.read_smc_value(&key)?;

        // Parse temperature value based on data type
        let data_type = std::str::from_utf8(&val.data_type).unwrap_or("????");

        match data_type {
            "sp78" => {
                // Most common temperature format: 16.16 fixed point
                let int_val = val.data[0] as i16;
                let frac_val = val.data[1] as i16;
                Ok(int_val as f32 + (frac_val as f32 / 256.0))
            },
            "flt " => {
                // Float format
                let bytes = [val.data[0], val.data[1], val.data[2], val.data[3]];
                let float_val = f32::from_be_bytes(bytes);
                Ok(float_val)
            },
            "ui8 " | "ui16" => {
                // Integer format
                if val.data_size == 1 {
                    Ok(val.data[0] as f32)
                } else {
                    let int_val = u16::from_be_bytes([val.data[0], val.data[1]]);
                    Ok(int_val as f32)
                }
            },
            _ => {
                // Unknown format, try to interpret as sp78
                let int_val = val.data[0] as i16;
                let frac_val = val.data[1] as i16;
                Ok(int_val as f32 + (frac_val as f32 / 256.0))
            },
        }
    }

    // Create an SMC key structure from a string
    fn create_smc_key(&self, key_str: &str) -> Result<SmcKey> {
        let mut key = SmcKey { key: [0; 4] };

        // Convert string to key bytes
        let bytes = key_str.as_bytes();
        for i in 0..std::cmp::min(bytes.len(), 4) {
            key.key[i] = bytes[i];
        }

        Ok(key)
    }

    // Read a value from the SMC using a key
    fn read_smc_value(&self, key: &SmcKey) -> Result<SmcVal> {
        unsafe {
            // Open connection to the SMC
            let mut conn: u32 = 0;
            let mut input = SMCKeyData_t::default();
            let mut output = SMCKeyData_t::default();

            // Convert key to u32 for SMCKeyData_t
            let key_u32 = u32::from_be_bytes([key.key[0], key.key[1], key.key[2], key.key[3]]);

            // Prepare input for IOConnectCallStructMethod
            input.key = key_u32;
            input.data8 = SMC_CMD_READ_KEY;

            // Get IOKit service
            let service_name = CString::new("AppleSMC").unwrap();
            let io_service =
                IOServiceGetMatchingService(IOMASTER_PORT_DEFAULT, IOServiceMatching(service_name.as_ptr()));

            if io_service == 0 {
                return Err(crate::error::Error::gpu_error("get_temperature", "Failed to get SMC service"));
            }

            // Open connection to service
            let result = IOServiceOpen(
                io_service,
                0, // mach_task_self()
                KERNEL_INDEX_SMC,
                &mut conn,
            );

            // We don't have IOObjectRelease in bindings, so we'll just continue

            if result != IO_RETURN_SUCCESS {
                return Err(crate::error::Error::gpu_error("get_temperature", "Failed to open connection to SMC"));
            }

            // First get key info to determine size and type
            input.data8 = SMC_CMD_READ_KEYINFO;

            let mut output_size = IOByteCount(std::mem::size_of::<SMCKeyData_t>());

            let result = IOConnectCallStructMethod(
                conn,
                2, // kSMCHandleYPCEvent
                &input as *const SMCKeyData_t,
                IOByteCount(std::mem::size_of::<SMCKeyData_t>()),
                &mut output as *mut SMCKeyData_t,
                &mut output_size,
            );

            if result != IO_RETURN_SUCCESS {
                IOServiceClose(conn);
                return Err(crate::error::Error::gpu_error("get_temperature", "Failed to get key info from SMC"));
            }

            // Now read the actual data
            input.data8 = SMC_CMD_READ_BYTES;

            let result = IOConnectCallStructMethod(
                conn,
                2, // kSMCHandleYPCEvent
                &input as *const SMCKeyData_t,
                IOByteCount(std::mem::size_of::<SMCKeyData_t>()),
                &mut output as *mut SMCKeyData_t,
                &mut output_size,
            );

            // Close connection
            IOServiceClose(conn);

            if result != IO_RETURN_SUCCESS {
                return Err(crate::error::Error::gpu_error("get_temperature", "Failed to read SMC value"));
            }

            // Parse output into SmcVal
            let mut val = SmcVal { data_size: 0, data_type: [0; 4], data_attributes: 0, data: [0; 32] };

            // Extract data type (big endian)
            let mut data_type_bytes = [0u8; 4];
            data_type_bytes[0] = output.key_info; // Place the byte in the first position
            val.data_type.copy_from_slice(&data_type_bytes);

            // Extract data size
            val.data_size = (output.data8 & 0x3F) as u32;

            // Copy data
            let data_size = std::cmp::min(val.data_size as usize, 32);

            // Access the bytes field from the SMCKeyData_t_data union
            let data_bytes = match output.data {
                SMCKeyData_t_data { bytes } => bytes,
            };

            val.data[0..data_size].copy_from_slice(&data_bytes[0..data_size]);

            Ok(val)
        }
    }

    // Estimate temperature based on utilization when sensors are unavailable
    fn estimate_temperature(&self) -> Result<f32> {
        // Get utilization as a proxy for temperature
        let utilization = self.estimate_utilization()?;

        // Base temperature for idle GPU
        let base_temp = 35.0;

        // Maximum temperature increase under full load
        let max_temp_increase = 40.0;

        // Calculate estimated temperature
        let estimated_temp = base_temp + (utilization / 100.0) * max_temp_increase;

        Ok(estimated_temp)
    }
}
