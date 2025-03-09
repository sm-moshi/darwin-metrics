use std::{io, result};

use thiserror::Error;

/// Specific error types for darwin-metrics
#[derive(Error, Debug)]
pub enum Error {
    /// Error originating from the system's IO subsystem
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error related to IOKit operations
    #[error("IOKit error: {0}")]
    IOKit(String),

    /// Error related to temperature monitoring
    #[error("Temperature monitoring error: {0}")]
    Temperature(String),

    /// Error related to CPU monitoring
    #[error("CPU monitoring error: {0}")]
    Cpu(String),

    /// Error related to GPU monitoring
    #[error("GPU monitoring error: {0}")]
    Gpu(String),

    /// Error related to memory monitoring
    #[error("Memory monitoring error: {0}")]
    Memory(String),

    /// Error related to network monitoring
    #[error("Network monitoring error: {0}")]
    Network(String),

    /// Error related to process monitoring
    #[error("Process monitoring error: {0}")]
    Process(String),

    /// Error related to system information retrieval
    #[error("System info error: {0}")]
    SystemInfo(String),

    /// Error related to service discovery
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Error for invalid input or data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Error for feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Error for feature not available on current system
    #[error("Feature not available: {0}")]
    NotAvailable(String),

    /// Error for permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Generic system error
    #[error("System error: {0}")]
    System(String),

    /// Generic error
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Create a new IOKit error
    pub fn io_kit<S: Into<String>>(message: S) -> Self {
        Error::IOKit(message.into())
    }

    /// Create a new system error
    pub fn system<S: Into<String>>(message: S) -> Self {
        Error::System(message.into())
    }

    /// Create a new invalid data error
    pub fn invalid_data<S: Into<String>>(message: S) -> Self {
        Error::InvalidData(message.into())
    }

    /// Create a new "not implemented" error
    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Error::NotImplemented(feature.into())
    }

    /// Create a new "not available" error
    pub fn not_available<S: Into<String>>(feature: S) -> Self {
        Error::NotAvailable(feature.into())
    }

    /// Create a new "permission denied" error
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Error::PermissionDenied(message.into())
    }

    /// Create a new service not found error
    pub fn service_not_found<S: Into<String>>(service: S) -> Self {
        Error::ServiceNotFound(service.into())
    }

    /// Create a new process error
    pub fn process_error<S: Into<String>>(message: S) -> Self {
        Error::Process(message.into())
    }

    /// Get details about the error
    pub fn details(&self) -> String {
        match self {
            Error::Io(e) => format!("{}: {}", e, e.kind()),
            Error::PermissionDenied(msg) => {
                format!("Permission denied: {}. Try running with elevated privileges.", msg)
            },
            Error::IOKit(msg) => format!(
                "IOKit error: {}. This might be a compatibility issue with your macOS version.",
                msg
            ),
            Error::NotAvailable(msg) => format!(
                "Feature not available: {}. This feature might not be supported on your hardware.",
                msg
            ),
            _ => format!("{}", self),
        }
    }

    /// Determine if this error is caused by insufficient permissions
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Error::PermissionDenied(_))
            || matches!(self, Error::Io(e) if e.kind() == io::ErrorKind::PermissionDenied)
    }

    /// Check if this error indicates a feature is not available
    pub fn is_not_available(&self) -> bool {
        matches!(self, Error::NotAvailable(_))
    }
}

/// Result type for darwin-metrics
pub type Result<T> = result::Result<T, Error>;
