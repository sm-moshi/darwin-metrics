use std::sync::Mutex;

use once_cell::sync::Lazy;

// Initialize the test mutex
pub static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// Initialize a static semaphore with only one permit to prevent disk tests from running in parallel
pub static DISK_TEST_LOCK: Mutex<()> = Mutex::new(());

// Re-export test modules
mod monitors;
// Empty module to satisfy references
mod integration {}

// Re-export common test utilities
pub use crate::common::builders::disk::*;
