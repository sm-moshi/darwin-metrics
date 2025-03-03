use thiserror::Error;

/// Error type for darwin-metrics operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Service not found")]
    ServiceNotFound,
    #[error("Feature not available: {0}")]
    NotAvailable(String),
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    #[error("System error: {0}")]
    SystemError(String),
}

impl Error {
    pub(crate) fn not_available(msg: impl Into<String>) -> Self {
        Error::NotAvailable(msg.into())
    }

    pub(crate) fn not_implemented(msg: impl Into<String>) -> Self {
        Error::NotImplemented(msg.into())
    }

    pub(crate) fn system_error(msg: impl Into<String>) -> Self {
        Error::SystemError(msg.into())
    }
}

/// Result type for darwin-metrics operations
pub type Result<T> = std::result::Result<T, Error>;

// Public modules
pub mod battery;
pub mod cpu;
pub mod disk;
pub mod ffi;
pub mod gpu;
pub mod memory;
pub mod temperature;
pub mod iokit;

// Private modules
mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // We'll replace this with proper tests as we implement features
    }
}
