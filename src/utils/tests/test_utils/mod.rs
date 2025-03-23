pub mod iokit_mock;
mod mocks;

// Explicitly re-export the mocks we need
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use iokit_mock::MockIOKit;
pub use mocks::{NetworkStats, create_string_dictionary, create_test_dictionary, get_network_stats_native};
use objc2::runtime::NSObject;
use objc2::{class, msg_send};
use scopeguard::defer;
use thiserror::Error;
use tokio::sync::OnceCell;

/// Errors that can occur during test execution
#[derive(Error, Debug)]
pub enum TestError {
    #[error("FFI resource error: {0}")]
    FFIError(String),
    #[error("Resource cleanup failed: {0}")]
    CleanupError(String),
    #[error("Initialization error: {0}")]
    InitError(String),
}

/// Global initialization state
static INIT: OnceCell<()> = OnceCell::const_new();
static CLEANUP_NEEDED: AtomicBool = AtomicBool::new(false);

/// RAII guard for test resources
pub struct TestGuard {
    pool: *mut NSObject,
}

impl TestGuard {
    /// Creates a new test guard, ensuring resources are properly initialized
    pub async fn new() -> Result<Self, TestError> {
        INIT.get_or_try_init(|| async {
            // Initialize test resources
            if let Err(e) = Self::initialize_resources().await {
                return Err(TestError::InitError(e.to_string()));
            }
            CLEANUP_NEEDED.store(true, Ordering::SeqCst);
            Ok(())
        })
        .await
        .map_err(|e| TestError::InitError(e.to_string()))?;

        // Create autorelease pool
        let pool = unsafe {
            let pool: *mut NSObject = msg_send![class!(NSAutoreleasePool), new];
            if pool.is_null() {
                return Err(TestError::InitError("Failed to create autorelease pool".into()));
            }
            pool
        };

        Ok(Self { pool })
    }

    /// Initialize any required test resources
    async fn initialize_resources() -> Result<(), TestError> {
        // Add resource initialization here
        Ok(())
    }

    /// Clean up any allocated resources
    fn cleanup_resources() -> Result<(), TestError> {
        // Add resource cleanup here
        Ok(())
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        unsafe {
            // Drain the autorelease pool
            let _: () = msg_send![self.pool, drain];
        }

        if CLEANUP_NEEDED.load(Ordering::SeqCst) {
            if let Err(e) = Self::cleanup_resources() {
                eprintln!("Resource cleanup failed: {}", e);
            }
            CLEANUP_NEEDED.store(false, Ordering::SeqCst);
        }
    }
}
