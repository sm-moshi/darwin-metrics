/// GPU monitoring functionality
///
/// This module provides tools for monitoring GPU status and performance on macOS systems. It includes support for:
///
/// - GPU utilization
/// - Memory usage
/// - Temperature monitoring
/// - Performance metrics
pub mod gpu;

/// IOKit interface functionality
///
/// This module provides a low-level interface to macOS's IOKit framework, which is used for hardware monitoring and
/// control. It includes:
///
/// - Service discovery
/// - Property access
/// - Hardware monitoring
/// - Device control
pub mod iokit;

/// Temperature monitoring functionality
///
/// This module provides tools for monitoring system temperatures on macOS systems. It includes support for:
///
/// - CPU temperature
/// - GPU temperature
/// - Battery temperature
/// - Ambient temperature
/// - Fan control based on temperature
pub mod temperature;

pub mod cpu;
pub mod memory;

pub use cpu::CPU;
pub use gpu::Gpu;
pub use iokit::IOKit;
pub use memory::Memory;
