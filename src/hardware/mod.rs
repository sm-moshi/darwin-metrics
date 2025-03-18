/// Hardware monitoring functionality for macOS systems
///
/// This module provides a unified interface for monitoring various hardware components through
/// macOS's IOKit framework. It includes comprehensive monitoring capabilities for:
///
/// - CPU metrics (frequency, temperature, usage)
/// - GPU metrics (utilization, memory, temperature)
/// - Memory metrics (usage, pressure, swap)
/// - Battery metrics (charge, health, temperature)
/// - Thermal metrics (temperatures, fan control)
/// - Power metrics (consumption, state, management)
/// - Disk metrics (I/O, utilization, health, mount status)
///
/// The implementation is based on IOKit, which provides low-level access to hardware monitoring
/// and control capabilities on macOS systems.
///
/// # Example
///
/// ```rust
/// use darwin_metrics::hardware::{IOKitImpl, CpuMonitor, GpuMonitor, ThermalMonitor};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let monitor = IOKitImpl::new()?;
///
///     // Monitor CPU
///     let cpu_freq = monitor.frequency().await?;
///     println!("CPU Frequency: {:.2} GHz", cpu_freq / 1000.0);
///
///     // Monitor GPU
///     let gpu_util = monitor.utilization().await?;
///     println!("GPU Utilization: {:.1}%", gpu_util);
///
///     // Monitor temperatures
///     if let Some(temp) = monitor.cpu_temperature().await? {
///         println!("CPU Temperature: {:.1}°C", temp);
///     }
///
///     Ok(())
/// }
/// ```

// IOKit module for hardware interaction
#[cfg(any(test, feature = "testing"))]
/// IOKit module for hardware interaction with macOS system APIs
pub mod iokit;
#[cfg(not(any(test, feature = "testing")))]
pub(crate) mod iokit;

// Hardware component modules
// NOTE: These modules have been moved to the root level

/// Memory monitoring functionality
pub mod memory;
/// Temperature monitoring functionality
pub mod temperature;

// Prelude module for convenient imports
/// Prelude module for convenient imports of hardware monitoring types
pub mod prelude {}
