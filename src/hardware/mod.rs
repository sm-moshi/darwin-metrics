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
/// ```
use std::future::Future;

// IOKit module for hardware interaction
/// IOKit module for hardware interaction with macOS system APIs
pub mod iokit;

// Hardware component modules
// NOTE: These modules have been moved to the root level

/// Temperature monitoring functionality
// pub mod temperature;

// Prelude module for convenient imports
/// Prelude module for convenient imports of hardware monitoring types
pub mod prelude {}

/// Run a future to completion on the current thread
pub fn block_on<F: Future>(future: F) -> F::Output {
    futures::executor::block_on(future)
}
