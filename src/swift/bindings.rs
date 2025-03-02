use swift_bridge::*;

// Battery Information
#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    pub struct BatteryInfoFFI {
        pub is_present: bool,
        pub is_charging: bool,
        pub percentage: f64,
        pub time_remaining: i32,
    }

    extern "Swift" {
        #[swift_bridge(swift_name = "getBatteryInfo")]
        fn get_battery_info() -> *mut BatteryInfoFFI;
    }
}

// CPU Information
#[swift_bridge::bridge]
mod ffi_cpu {
    #[swift_bridge(swift_repr = "struct")]
    pub struct CPUInfoFFI {
        pub cores: i32,
        pub frequency_mhz: f64,
    }

    extern "Swift" {
        #[swift_bridge(swift_name = "getCPUInfo")]
        fn get_cpu_info() -> *mut CPUInfoFFI;
    }
}

// Memory Information
#[swift_bridge::bridge]
mod ffi_memory {
    #[swift_bridge(swift_repr = "struct")]
    pub struct MemoryInfoFFI {
        pub total_gb: f64,
        pub used_gb: f64,
        pub free_gb: f64,
    }

    extern "Swift" {
        #[swift_bridge(swift_name = "getMemoryInfo")]
        fn get_memory_info() -> *mut MemoryInfoFFI;
    }
}

// Re-export the FFI types and functions
pub use ffi::*;
pub use ffi_cpu::*;
pub use ffi_memory::*; 