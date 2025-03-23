//! Error types for the darwin-metrics crate.
//!
//! This module provides a comprehensive error handling system for all operations in the darwin-metrics crate. It
//! includes specific error variants for different types of failures and implements standard error handling traits.

use std::ffi::NulError;
use std::sync::Arc;
use std::{fmt, io};

use thiserror::Error;
use tokio::task::JoinError;

/// A specialized Result type for darwin-metrics operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents all possible errors that can occur in darwin-metrics operations.
#[derive(Debug)]
pub enum Error {
    /// An error caused by an I/O operation
    IoError(io::Error),

    /// An error originating from IOKit
    IoKit(String),

    /// An error related to temperature sensors
    Temperature(String),

    /// An error related to network operations
    Network(String),

    /// An error caused by an invalid argument
    InvalidArgument(String),

    /// An error caused by invalid data
    InvalidData(String),

    /// An error caused by an invalid state
    InvalidState(String),

    /// An error caused by a null pointer
    NullPointer(String),

    /// An error caused by an invalid pointer
    InvalidPointer(String),

    /// An error caused by an invalid value
    InvalidValue(String),

    /// An error caused by a resource limit being exceeded
    ResourceLimitExceeded(String),

    /// An error caused by a lock operation
    LockError(String),

    /// An error caused by a channel operation
    ChannelError(String),

    /// A general system error
    SystemError(String),

    /// A resource that is not available
    NotAvailable(String),

    /// An error caused by an invalid monitor type
    InvalidMonitorType(String),

    /// A feature that is not implemented
    NotImplemented(String),

    /// A conversion error
    ConversionError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::SystemError(msg) => write!(f, "System error: {}", msg),
            Error::NotAvailable(msg) => write!(f, "Not available: {}", msg),
            Error::IoKit(msg) => write!(f, "IOKit error: {}", msg),
            Error::Temperature(msg) => write!(f, "Temperature error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Error::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Error::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Error::NullPointer(msg) => write!(f, "Null pointer: {}", msg),
            Error::InvalidPointer(msg) => write!(f, "Invalid pointer: {}", msg),
            Error::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Error::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {}", msg),
            Error::LockError(msg) => write!(f, "Lock error: {}", msg),
            Error::ChannelError(msg) => write!(f, "Channel error: {}", msg),
            Error::InvalidMonitorType(msg) => write!(f, "Invalid monitor type: {}", msg),
            Error::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Error::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

// Helper functions to create specific error types
impl Error {
    pub fn io_error<E: Into<io::Error>>(error: E) -> Self {
        Error::IoError(error.into())
    }

    pub fn iokit_error<S: Into<String>>(message: S) -> Self {
        Error::IoKit(message.into())
    }

    pub fn temperature_error<S: Into<String>>(message: S) -> Self {
        Error::Temperature(message.into())
    }

    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Error::Network(message.into())
    }

    pub fn invalid_argument<S: Into<String>>(message: S) -> Self {
        Error::InvalidArgument(message.into())
    }

    pub fn invalid_data<S: Into<String>>(message: S) -> Self {
        Error::InvalidData(message.into())
    }

    pub fn invalid_state<S: Into<String>>(message: S) -> Self {
        Error::InvalidState(message.into())
    }

    pub fn null_pointer<S: Into<String>>(message: S) -> Self {
        Error::NullPointer(message.into())
    }

    pub fn invalid_pointer<S: Into<String>>(message: S) -> Self {
        Error::InvalidPointer(message.into())
    }

    pub fn invalid_value<S: Into<String>>(message: S) -> Self {
        Error::InvalidValue(message.into())
    }

    pub fn resource_limit_exceeded<S: Into<String>>(message: S) -> Self {
        Error::ResourceLimitExceeded(message.into())
    }

    pub fn lock_error<S: Into<String>>(message: S) -> Self {
        Error::LockError(message.into())
    }

    pub fn channel_error<S: Into<String>>(message: S) -> Self {
        Error::ChannelError(message.into())
    }

    pub fn system_error<S: Into<String>>(message: S) -> Self {
        Error::SystemError(message.into())
    }

    pub fn not_available<S: Into<String>>(message: S) -> Self {
        Error::NotAvailable(message.into())
    }

    pub fn invalid_monitor_type<S: Into<String>>(message: S) -> Self {
        Error::InvalidMonitorType(message.into())
    }

    pub fn not_implemented<S: Into<String>>(message: S) -> Self {
        Error::NotImplemented(message.into())
    }

    pub fn conversion_error<S: Into<String>>(message: S) -> Self {
        Error::ConversionError(message.into())
    }

    pub fn service_not_found<S: Into<String>>(msg: S) -> Self {
        Self::system_error(msg.into())
    }

    pub fn mutex_lock_error<S: Into<String>>(msg: S) -> Self {
        Self::system_error(format!("Mutex lock error: {}", msg.into()))
    }

    pub fn process_error<P, M>(pid: Option<P>, message: M) -> Self
    where
        P: Into<u32> + std::fmt::Display,
        M: Into<String>,
    {
        Self::system_error(format!(
            "Process error{}: {}",
            pid.map(|p| format!(" (PID {})", p)).unwrap_or_default(),
            message.into()
        ))
    }

    pub fn gpu_error<O, M>(operation: O, message: M) -> Self
    where
        O: Into<String>,
        M: Into<String>,
    {
        Self::system_error(format!("GPU error during {}: {}", operation.into(), message.into()))
    }

    /// Check if the error is a permission denied error
    pub fn is_permission_denied(&self) -> bool {
        if let Error::IoError(io_err) = self {
            io_err.kind() == io::ErrorKind::PermissionDenied
        } else {
            false
        }
    }

    /// Check if the error is a not available error
    pub fn is_not_available(&self) -> bool {
        matches!(self, Error::NotAvailable(..))
    }
}

impl From<NulError> for Error {
    fn from(_err: NulError) -> Self {
        Error::InvalidData("Invalid null character in string".to_string())
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Error::system_error(format!("Task join error: {}", err))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::system_error(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::system_error(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation_methods() {
        let temp_err = Error::temperature_error("CPU");
        assert!(matches!(temp_err, Error::Temperature(..)));

        let net_err = Error::network_error("eth0");
        assert!(matches!(net_err, Error::Network(..)));

        let proc_err = Error::process_error(Some(123u32), "Not responding");
        assert!(matches!(proc_err, Error::SystemError(..)));

        let gpu_err = Error::gpu_error("render", "Out of memory");
        assert!(matches!(gpu_err, Error::SystemError(..)));

        let arg_err = Error::invalid_argument("port");
        assert!(matches!(arg_err, Error::InvalidArgument(..)));

        let sys_err = Error::system_error("sysctl");
        assert!(matches!(sys_err, Error::SystemError(..)));
    }

    #[test]
    fn test_error_is_permission_denied() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err = Error::IoError(err);
        assert!(err.is_permission_denied());

        let err = Error::SystemError("test".into());
        assert!(!err.is_permission_denied());
    }

    #[test]
    fn test_error_is_not_available() {
        let err = Error::NotAvailable("sensor".into());
        assert!(err.is_not_available());

        let err = Error::SystemError("test".into());
        assert!(!err.is_not_available());
    }

    #[test]
    fn test_error_display() {
        let err = Error::Temperature("Too hot".into());
        assert_eq!(format!("{}", err), "Temperature error: Too hot");

        let err = Error::SystemError("Not responding".into());
        assert_eq!(format!("{}", err), "System error: Not responding");

        let err = Error::InvalidData("Invalid port".into());
        assert_eq!(format!("{}", err), "Invalid data: Invalid port");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::IoError(io_err);
        assert!(matches!(err, Error::IoError(..)));

        let err = Error::from("test error");
        assert!(matches!(err, Error::SystemError(..)));
    }
}
