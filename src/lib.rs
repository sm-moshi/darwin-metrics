use std::error::Error as StdError;
use std::fmt;

/// Error type for darwin-metrics operations
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The requested feature is not implemented
    NotImplemented,
    /// The requested metric or feature is not available
    NotAvailable,
    /// Invalid data
    InvalidData,
    /// An error occurred in the system API
    SystemError(String),
}

impl Error {
    pub(crate) fn not_available(msg: impl Into<String>) -> Self {
        Error::NotAvailable
    }

    pub(crate) fn not_implemented(msg: impl Into<String>) -> Self {
        Error::NotImplemented
    }

    pub(crate) fn system_error(msg: impl Into<String>) -> Self {
        Error::SystemError(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotImplemented => write!(f, "Feature not implemented"),
            Error::NotAvailable => write!(f, "Feature not available"),
            Error::InvalidData => write!(f, "Invalid data"),
            Error::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl StdError for Error {}

/// Result type for darwin-metrics operations
pub type Result<T> = std::result::Result<T, Error>;

// Module declarations
pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod temperature;

// Re-exports for convenience
pub use battery::Battery;
pub use cpu::CPU;
pub use disk::Disk;
pub use gpu::GPU;
pub use memory::Memory;
pub use temperature::Temperature;

// Private modules
mod utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // We'll replace this with proper tests as we implement features
    }
}

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type Error;
        type Battery;
        
        #[swift_bridge(associated_to = Battery)]
        fn get_info() -> Result<Battery, Error>;
        
        #[swift_bridge(init)]
        fn new(is_present: bool, is_charging: bool, percentage: f64, time_remaining: i32) -> Battery;
    }

    extern "Rust" {
        type CPU;
        
        #[swift_bridge(associated_to = CPU)]
        fn get_info() -> Result<CPU, Error>;
    }

    extern "Rust" {
        type Memory;
        
        #[swift_bridge(associated_to = Memory)]
        fn get_info() -> Result<Memory, Error>;
        
        #[swift_bridge(init)]
        fn new(total: u64, available: u64, used: u64, wired: u64, pressure: f64) -> Memory;
    }

    extern "Rust" {
        type GPU;
        
        #[swift_bridge(associated_to = GPU)]
        fn get_info() -> Result<GPU, Error>;
        
        #[swift_bridge(init)]
        fn new(name: String, utilization: f64, memory_used: u64, memory_total: u64, temperature: f64) -> GPU;
    }

    extern "Rust" {
        type Disk;
        
        #[swift_bridge(associated_to = Disk)]
        fn get_info() -> Result<Disk, Error>;
        
        #[swift_bridge(init)]
        fn new(device: String, mount_point: String, fs_type: String, total: u64, available: u64, used: u64) -> Disk;
    }

    extern "Rust" {
        type Temperature;
        
        #[swift_bridge(associated_to = Temperature)]
        fn get_info() -> Result<Temperature, Error>;
        
        #[swift_bridge(init)]
        fn from_fahrenheit(sensor: String, fahrenheit: f64) -> Temperature;
    }
}
