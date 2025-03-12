use std::{io, result};

use thiserror::Error;

/// Specific error types for darwin-metrics
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Error originating from the system's IO subsystem
    #[error("IO error: {kind} - {message}")]
    Io { kind: io::ErrorKind, message: String },

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
            Error::Io { kind, message } => format!("{}: {}", message, kind),
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
            || matches!(self, Error::Io { kind, .. } if *kind == io::ErrorKind::PermissionDenied)
    }

    /// Check if this error indicates a feature is not available
    pub fn is_not_available(&self) -> bool {
        matches!(self, Error::NotAvailable(_))
    }
}

/// Result type for darwin-metrics
pub type Result<T> = result::Result<T, Error>;

/// Implement conversion from io::Error to Error
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io { kind: err.kind(), message: err.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_creation_methods() {
        // Test all error factory methods
        let e1 = Error::io_kit("test io_kit error");
        assert!(matches!(e1, Error::IOKit(s) if s == "test io_kit error"));

        let e2 = Error::system("test system error");
        assert!(matches!(e2, Error::System(s) if s == "test system error"));

        let e3 = Error::invalid_data("test invalid data");
        assert!(matches!(e3, Error::InvalidData(s) if s == "test invalid data"));

        let e4 = Error::not_implemented("test feature");
        assert!(matches!(e4, Error::NotImplemented(s) if s == "test feature"));

        let e5 = Error::not_available("test feature");
        assert!(matches!(e5, Error::NotAvailable(s) if s == "test feature"));

        let e6 = Error::permission_denied("test permission");
        assert!(matches!(e6, Error::PermissionDenied(s) if s == "test permission"));

        let e7 = Error::service_not_found("test service");
        assert!(matches!(e7, Error::ServiceNotFound(s) if s == "test service"));

        let e8 = Error::process_error("test process error");
        assert!(matches!(e8, Error::Process(s) if s == "test process error"));
    }

    #[test]
    fn test_error_details_io() {
        // Test IO error details formatting - using a simpler check that avoids exact string match
        let e1 = Error::Io { kind: ErrorKind::NotFound, message: "file not found".to_string() };
        let details = e1.details();
        assert!(details.contains("file not found"));
        assert!(details.contains("not found"));
    }

    #[test]
    fn test_error_details_permission() {
        // Test permission error details
        let e = Error::PermissionDenied("access denied".to_string());
        assert!(e.details().contains("Permission denied: access denied"));
        assert!(e.details().contains("elevated privileges"));
    }

    #[test]
    fn test_error_details_iokit() {
        // Test IOKit error details
        let e = Error::IOKit("service failed".to_string());
        assert!(e.details().contains("IOKit error: service failed"));
    }

    #[test]
    fn test_error_details_not_available() {
        // Test not available error details
        let e = Error::NotAvailable("GPU features".to_string());
        assert!(e.details().contains("Feature not available: GPU features"));
    }

    #[test]
    fn test_error_details_other() {
        // Test other error details
        let e = Error::Other("generic error".to_string());
        assert_eq!(e.details(), "Error: generic error");
    }

    #[test]
    fn test_error_is_permission_error() {
        // Test is_permission_error method
        let e1 = Error::PermissionDenied("test".to_string());
        assert!(e1.is_permission_error());

        let e2 = Error::Io { kind: ErrorKind::PermissionDenied, message: "test".to_string() };
        assert!(e2.is_permission_error());

        let e3 = Error::Io { kind: ErrorKind::NotFound, message: "test".to_string() };
        assert!(!e3.is_permission_error());

        let e4 = Error::Other("test".to_string());
        assert!(!e4.is_permission_error());
    }

    #[test]
    fn test_error_is_not_available() {
        // Test is_not_available method
        let e1 = Error::NotAvailable("test".to_string());
        assert!(e1.is_not_available());

        let e2 = Error::NotImplemented("test".to_string());
        assert!(!e2.is_not_available());
    }

    #[test]
    fn test_from_io_error() {
        // Test From<io::Error> implementation
        let io_err = IoError::new(ErrorKind::ConnectionRefused, "connection error");
        let err: Error = io_err.into();

        if let Error::Io { kind, message } = err {
            assert_eq!(kind, ErrorKind::ConnectionRefused);
            assert!(message.contains("connection error"));
        } else {
            panic!("Error was not converted to Error::Io variant");
        }
    }
}
