/// FFI struct for battery information
#[repr(C)]
pub struct BatteryInfoFFI {
    pub is_present: bool,
    pub is_charging: bool,
    pub percentage: f64,
    pub time_remaining: u64,
}

/// FFI struct for CPU information
#[repr(C)]
pub struct CPUInfoFFI {
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub core_usage: *const f64,
    pub core_usage_len: usize,
    pub frequency_mhz: f64,
}

/// FFI struct for memory information
#[repr(C)]
pub struct MemoryInfoFFI {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub wired: u64,
    pub pressure: f64,
}

/// FFI struct for GPU information
#[repr(C)]
pub struct GPUInfoFFI {
    pub name: *const u8,
    pub name_len: usize,
    pub utilization: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f64,
}

/// FFI struct for disk information
#[repr(C)]
pub struct DiskInfoFFI {
    pub device: *const u8,
    pub device_len: usize,
    pub mount_point: *const u8,
    pub mount_point_len: usize,
    pub fs_type: *const u8,
    pub fs_type_len: usize,
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

/// FFI struct for temperature information
#[repr(C)]
pub struct TemperatureInfoFFI {
    pub sensor: *const u8,
    pub sensor_len: usize,
    pub celsius: f64,
    pub fahrenheit: f64,
}

unsafe extern "C" {
    pub fn get_battery_info() -> *mut BatteryInfoFFI;
    pub fn get_cpu_info() -> *mut CPUInfoFFI;
    pub fn get_memory_info() -> *mut MemoryInfoFFI;
    pub fn get_gpu_info() -> *mut GPUInfoFFI;
    pub fn get_disk_info() -> *mut *mut DiskInfoFFI;
    pub fn get_disk_info_len() -> usize;
    pub fn get_temperature_info() -> *mut *mut TemperatureInfoFFI;
    pub fn get_temperature_info_len() -> usize;
} 