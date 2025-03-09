//! Stub implementations for docs.rs
//!
//! This module provides minimal implementations of the crate's types
//! to allow docs.rs to generate documentation without requiring macOS APIs.
//! These stubs are only used when building on docs.rs (Linux environment).

// Re-export these to replace the real implementations
// when building on docs.rs
pub use self::stubs::*;

/// Stub implementations of the crate's public types
mod stubs {
    use crate::error::{Error, Result};

    // These stubs are conditionally included only when
    // building on docs.rs, and are not part of the actual crate

    // Stub implementations to make documentation work
    // without requiring macOS

    // Core modules
    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Battery;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Disk;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct DiskConfig;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub enum DiskType {
        Internal,
        External,
        Network,
        Virtual,
    }

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct CPU;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct FrequencyMetrics;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Gpu;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct GpuMetrics;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Memory;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct PageStates;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub enum PressureLevel {
        Normal,
        Warning,
        Critical,
    }

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct SwapUsage;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Temperature;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct ThermalMetrics;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Fan;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct NetworkInterface;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct NetworkTraffic;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct Process;

    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub struct ProcessInfo;

    // Implementation stubs for documentation
    impl Battery {
        #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
        pub fn new() -> Self {
            Self
        }
    }

    impl CPU {
        #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
        pub fn new() -> Self {
            Self
        }
    }

    impl Temperature {
        #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
        pub fn new() -> Self {
            Self
        }

        #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
        pub fn cpu_temperature(&mut self) -> Result<f64> {
            Err(Error::NotAvailable("Documentation only".to_string()))
        }

        #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
        pub fn get_thermal_metrics(&mut self) -> Result<ThermalMetrics> {
            Err(Error::NotAvailable("Documentation only".to_string()))
        }
    }
}
