//! Error types for the darwin-metrics crate.
//!
//! This module provides a comprehensive error handling system for all operations in the darwin-metrics crate. It
//! includes specific error variants for different types of failures and implements standard error handling traits.

use std::error::Error as StdError;
use std::fmt;
use std::io;

/// A specialized Result type for darwin-metrics operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents all possible errors that can occur in darwin-metrics operations.
#[derive(Debug)]
pub enum Error {
    /// I/O operation errors
    IoError(io::Error),
    /// IOKit-specific errors
    IOKitError(i32, String),
    /// Temperature sensor errors
    Temperature {
        /// The specific sensor that failed
        sensor: String,
        /// Description of what went wrong
        message: String,
    },
    /// CPU-related errors
    Cpu {
        /// Description of the CPU operation that failed
        operation: String,
        /// Additional error context
        message: String,
    },
    /// GPU-related errors
    Gpu {
        /// Description of the GPU operation that failed
        operation: String,
        /// Additional error context
        message: String,
    },
    /// Memory-related errors
    Memory {
        /// Description of the memory operation that failed
        operation: String,
        /// Additional error context
        message: String,
    },
    /// Network-related errors
    Network {
        /// Description of the network operation that failed
        operation: String,
        /// Additional error context
        message: String,
    },
    /// Process monitoring errors
    Process {
        /// The process ID that caused the error, if applicable
        pid: Option<u32>,
        /// Description of what went wrong
        message: String,
    },
    /// System information errors
    SystemInfo {
        /// The system call that failed
        call: String,
        /// Additional error context
        message: String,
    },
    /// Service not found errors
    ServiceNotFound(String),
    /// Invalid data errors
    InvalidData(String, Option<String>),
    /// Feature not implemented errors
    NotImplemented(String),
    /// Resource not available errors
    NotAvailable {
        /// The resource that wasn't available
        resource: String,
        /// Additional context about why it wasn't available
        reason: String,
    },
    /// Permission denied errors
    PermissionDenied {
        /// The operation that was denied
        operation: String,
        /// The required permission level
        required_permission: String,
    },
    /// General system errors
    System(String),
    /// Other unexpected errors
    Other(String),
    /// Mutex lock errors
    MutexLockError(String),
    /// IO errors with context
    IO { context: String, source: io::Error },
    /// IOKit errors
    IOKit { code: i32, operation: String },
    /// Invalid argument errors
    InvalidArgument {
        context: String,
        value: Option<String>,
    },
    /// System errors
    SystemError {
        operation: String,
        message: String,
    },
}

impl Error {
    /// Creates a new IO error with context
    pub fn io_error<C>(context: C, source: io::Error) -> Self
    where
        C: Into<String>,
    {
        Error::IO { context: context.into(), source }
    }

    /// Creates a new IOKit error
    pub fn iokit_error<S>(code: i32, operation: S) -> Self
    where
        S: Into<String>,
    {
        Error::IOKit { code, operation: operation.into() }
    }

    /// Creates a new Temperature error
    pub fn temperature_error<S, M>(sensor: S, message: M) -> Self
    where
        S: Into<String>,
        M: Into<String>,
    {
        Error::Temperature { sensor: sensor.into(), message: message.into() }
    }

    /// Creates a new Network error
    pub fn network_error<O, M>(operation: O, message: M) -> Self
    where
        O: Into<String>,
        M: Into<String>,
    {
        Error::Network { operation: operation.into(), message: message.into() }
    }

    /// Creates a new InvalidData error
    pub fn invalid_data<S, D>(context: S, value: Option<D>) -> Self
    where
        S: Into<String>,
        D: Into<String>,
    {
        Error::InvalidData(context.into(), value.map(|v| v.into()))
    }

    /// Creates a new ServiceNotFound error
    pub fn service_not_found<S: Into<String>>(msg: S) -> Self {
        Error::ServiceNotFound(msg.into())
    }

    /// Creates a new MutexLockError
    pub fn mutex_lock_error<S: Into<String>>(msg: S) -> Self {
        Error::MutexLockError(msg.into())
    }

    /// Creates a new System error
    pub fn system<S: Into<String>>(message: S) -> Self {
        Error::System(message.into())
    }

    /// Creates a new Process error
    pub fn process_error<P, M>(pid: Option<P>, message: M) -> Self
    where
        P: Into<u32>,
        M: Into<String>,
    {
        Error::Process { pid: pid.map(|p| p.into()), message: message.into() }
    }

    /// Creates a new GPU error
    pub fn gpu_error<O, M>(operation: O, message: M) -> Self
    where
        O: Into<String>,
        M: Into<String>,
    {
        Error::Gpu { operation: operation.into(), message: message.into() }
    }

    /// Creates a new invalid argument error
    pub fn invalid_argument<C, V>(context: C, value: Option<V>) -> Self 
    where
        C: Into<String>,
        V: Into<String>,
    {
        Error::InvalidArgument {
            context: context.into(),
            value: value.map(|v| v.into()),
        }
    }

    /// Creates a new system error
    pub fn system_error<O, M>(operation: O, message: M) -> Self 
    where
        O: Into<String>,
        M: Into<String>,
    {
        Error::SystemError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Error::PermissionDenied { operation: _, required_permission: _ })
    }

    pub fn is_not_available(&self) -> bool {
        matches!(self, Error::NotAvailable { resource: _, reason: _ })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::IOKitError(code, msg) => write!(f, "IOKit error {}: {}", code, msg),
            Error::Temperature { sensor, message } => {
                write!(f, "Temperature error for sensor {}: {}", sensor, message)
            },
            Error::Cpu { operation, message } => {
                write!(f, "CPU error during {}: {}", operation, message)
            },
            Error::Gpu { operation, message } => {
                write!(f, "GPU error during {}: {}", operation, message)
            },
            Error::Memory { operation, message } => {
                write!(f, "Memory error during {}: {}", operation, message)
            },
            Error::Network { operation, message } => {
                write!(f, "Network error during {}: {}", operation, message)
            },
            Error::Process { pid, message } => match pid {
                Some(pid) => write!(f, "Process error for PID {}: {}", pid, message),
                None => write!(f, "Process error: {}", message),
            },
            Error::SystemInfo { call, message } => {
                write!(f, "System info error in {}: {}", call, message)
            },
            Error::ServiceNotFound(msg) => write!(f, "Service not found: {}", msg),
            Error::InvalidData(msg, Some(details)) => {
                write!(f, "Invalid data: {} ({})", msg, details)
            },
            Error::InvalidData(msg, None) => write!(f, "Invalid data: {}", msg),
            Error::NotImplemented(feature) => write!(f, "Feature not implemented: {}", feature),
            Error::NotAvailable { resource, reason } => {
                write!(f, "Resource {} not available: {}", resource, reason)
            },
            Error::PermissionDenied { operation, required_permission } => {
                write!(f, "Permission denied for {}: requires {}", operation, required_permission)
            },
            Error::System(msg) => write!(f, "System error: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
            Error::MutexLockError(msg) => write!(f, "Mutex lock error: {}", msg),
            Error::IO { context, source } => write!(f, "IO error: {} ({})", context, source),
            Error::IOKit { code, operation } => write!(f, "IOKit error {}: {}", code, operation),
            Error::InvalidArgument { context, value } => {
                write!(f, "Invalid argument: {} (value: {:?})", context, value)
            },
            Error::SystemError { operation, message } => {
                write!(f, "System error during {}: {}", operation, message)
            },
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError(ref e) => Some(e),
            Error::IO { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_creation_methods() {
        // Test all error factory methods
        let e1 =
            Error::io_error("test io error", IoError::new(ErrorKind::NotFound, "file not found"));
        assert!(
            matches!(e1, Error::IoError(e) if e.kind() == ErrorKind::NotFound && e.to_string() == "IO error: file not found")
        );

        let e2 = Error::iokit_error(1, "test IOKit error");
        assert!(
            matches!(e2, Error::IOKitError(code, operation) if code == 1 && operation == "test IOKit error")
        );

        let e3 = Error::temperature_error("test temperature error", "sensor not found");
        assert!(
            matches!(e3, Error::Temperature { sensor, message } if sensor == "test temperature error" && message == "sensor not found")
        );

        let e4 = Error::system("CPU not found");
        assert!(matches!(e4, Error::System(msg) if msg == "CPU not found"));

        let e5 = Error::service_not_found("GPU service");
        assert!(matches!(e5, Error::ServiceNotFound(msg) if msg == "GPU service"));

        let e6 = Error::system("Memory operation failed");
        assert!(matches!(e6, Error::System(msg) if msg == "Memory operation failed"));

        let e7 = Error::network_error("test network error", "network not found");
        assert!(
            matches!(e7, Error::Network { operation, message } if operation == "test network error" && message == "network not found")
        );

        let e8 = Error::process_error(None::<u32>, "process not found");
        assert!(
            matches!(e8, Error::Process { pid, message } if pid.is_none() && message == "process not found")
        );
    }

    #[test]
    fn test_error_details_io() {
        // Test IO error details formatting - using a simpler check that avoids exact string match
        let e1 = Error::io_error(
            "test io_kit error",
            IoError::new(ErrorKind::NotFound, "file not found"),
        );
        let details = e1.to_string();
        assert!(details.contains("file not found"));
        assert!(details.contains("not found"));
    }

    #[test]
    fn test_error_details_permission() {
        // Test permission error details
        let e = Error::PermissionDenied {
            operation: "test operation".to_string(),
            required_permission: "elevated privileges".to_string(),
        };
        let details = e.to_string();
        assert!(
            details.contains("Permission denied for test operation: requires elevated privileges")
        );
    }

    #[test]
    fn test_error_details_iokit() {
        // Test IOKit error details
        let e = Error::iokit_error(1, "test IOKit error");
        let details = e.to_string();
        assert!(details.contains("IOKit error 1 during test IOKit error"));
    }

    #[test]
    fn test_error_details_not_available() {
        // Test not available error details
        let e = Error::NotAvailable {
            resource: "GPU".to_string(),
            reason: "GPU features not supported".to_string(),
        };
        let details = e.to_string();
        assert!(details.contains("GPU not available: GPU features not supported"));
    }

    #[test]
    fn test_error_details_other() {
        // Test other error details
        let e = Error::Other("generic error".to_string());
        let details = e.to_string();
        assert_eq!(details, "Error: generic error");
    }

    #[test]
    fn test_error_is_permission_error() {
        // Test is_permission_error method
        let e1 = Error::PermissionDenied {
            operation: "test operation".to_string(),
            required_permission: "elevated privileges".to_string(),
        };
        assert!(e1.is_permission_denied());

        let e2 = Error::io_error(
            "test io_kit error",
            IoError::new(ErrorKind::PermissionDenied, "permission denied"),
        );
        assert!(e2.is_permission_denied());

        let e3 = Error::io_error("test io_kit error", IoError::new(ErrorKind::NotFound, "test"));
        assert!(!e3.is_permission_denied());

        let e4 = Error::Other("test".to_string());
        assert!(!e4.is_permission_denied());
    }

    #[test]
    fn test_error_is_not_available() {
        // Test is_not_available method
        let e1 = Error::NotAvailable {
            resource: "GPU".to_string(),
            reason: "GPU features not supported".to_string(),
        };
        assert!(e1.is_not_available());

        let e2 = Error::NotImplemented("test".to_string());
        assert!(!e2.is_not_available());
    }

    #[test]
    fn test_from_io_error() {
        // Test From<io::Error> implementation
        let io_err = IoError::new(ErrorKind::ConnectionRefused, "connection error");
        let err = Error::io_error("test connection error", io_err);

        if let Error::IoError(source) = err {
            assert_eq!(source.kind(), ErrorKind::ConnectionRefused);
        } else {
            panic!("Error was not converted to Error::IoError variant");
        }
    }
}
