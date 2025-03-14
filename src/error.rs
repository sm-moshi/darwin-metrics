//! Error types for the darwin-metrics crate.
//!
//! This module provides a comprehensive error handling system for all operations in the darwin-metrics crate. It
//! includes specific error variants for different types of failures and implements standard error handling traits.

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::ffi::NulError;
use thiserror::Error;

/// A specialized Result type for darwin-metrics operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents all possible errors that can occur in darwin-metrics operations.
#[derive(Debug)]
pub enum Error {
    /// IO error from std::io
    IoError {
        source: io::Error,
    },
    /// Error from IOKit operations
    IOKitError {
        code: i32,
        message: String,
    },
    /// Invalid data error
    InvalidData {
        message: String,
        details: String,
    },
    /// Service not found error
    ServiceNotFound {
        message: String,
    },
    /// System error
    System {
        message: String,
    },
    /// Temperature sensor error
    Temperature {
        sensor: String,
        message: String,
    },
    /// Network operation error
    Network {
        operation: String,
        message: String,
    },
    /// Mutex lock error
    MutexLockError {
        message: String,
    },
    /// Process error
    Process {
        pid: Option<u32>,
        message: String,
    },
    /// GPU error
    Gpu {
        operation: String,
        message: String,
    },
    /// Invalid argument error
    InvalidArgument {
        context: String,
        value: String,
    },
    /// System error with operation context
    SystemError {
        operation: String,
        message: String,
    },
    /// Permission denied error
    PermissionDenied {
        operation: String,
        required_permission: String,
    },
    /// Resource not available error
    NotAvailable {
        resource: String,
        reason: String,
    },
    /// Other error
    Other {
        message: String,
    },
}

impl Error {
    /// Creates a new IO error with context
    pub fn io_error<C>(_context: C, source: io::Error) -> Self
    where
        C: Into<String>,
    {
        Error::IoError { source }
    }

    /// Creates a new IOKit error
    pub fn iokit_error<S>(code: i32, operation: S) -> Self
    where
        S: Into<String>,
    {
        Error::IOKitError { code, message: operation.into() }
    }

    /// Creates a new Temperature error
    pub fn temperature_error<S, M>(sensor: S, message: M) -> Self
    where
        S: Into<String>,
        M: Into<String>,
    {
        Error::Temperature {
            sensor: sensor.into(),
            message: message.into(),
        }
    }

    /// Creates a new Network error
    pub fn network_error<O, M>(operation: O, message: M) -> Self
    where
        O: Into<String>,
        M: Into<String>,
    {
        Error::Network {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Creates a new InvalidData error
    pub fn invalid_data<S, D>(context: S, value: Option<D>) -> Self
    where
        S: Into<String>,
        D: Into<String>,
    {
        Error::InvalidData {
            message: context.into(),
            details: value.map(|v| v.into()).unwrap_or_default(),
        }
    }

    /// Creates a new ServiceNotFound error
    pub fn service_not_found<S: Into<String>>(msg: S) -> Self {
        Error::ServiceNotFound { message: msg.into() }
    }

    /// Creates a new MutexLockError
    pub fn mutex_lock_error<S: Into<String>>(msg: S) -> Self {
        Error::MutexLockError { message: msg.into() }
    }

    /// Creates a new System error
    pub fn system<S: Into<String>>(message: S) -> Self {
        Error::System { message: message.into() }
    }

    /// Creates a new Process error
    pub fn process_error<P, M>(pid: Option<P>, message: M) -> Self
    where
        P: Into<u32>,
        M: Into<String>,
    {
        Error::Process {
            pid: pid.map(|p| p.into()),
            message: message.into(),
        }
    }

    /// Creates a new GPU error
    pub fn gpu_error<O, M>(operation: O, message: M) -> Self
    where
        O: Into<String>,
        M: Into<String>,
    {
        Error::Gpu {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Creates a new invalid argument error
    pub fn invalid_argument<C, V>(context: C, value: Option<V>) -> Self
    where
        C: Into<String>,
        V: Into<String>,
    {
        Error::InvalidArgument {
            context: context.into(),
            value: value.map(|v| v.into()).unwrap_or_default(),
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

    /// Check if the error is a permission denied error
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Error::PermissionDenied { .. })
    }

    /// Check if the error is a not available error
    pub fn is_not_available(&self) -> bool {
        matches!(self, Error::NotAvailable { .. })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError { source } => write!(f, "IO error: {}", source),
            Error::IOKitError { code, message } => write!(f, "IOKit error {}: {}", code, message),
            Error::InvalidData { message, details } => write!(f, "Invalid data: {} ({})", message, details),
            Error::ServiceNotFound { message } => write!(f, "Service not found: {}", message),
            Error::System { message } => write!(f, "System error: {}", message),
            Error::Temperature { sensor, message } => write!(f, "Temperature error on sensor {}: {}", sensor, message),
            Error::Network { operation, message } => write!(f, "Network error during {}: {}", operation, message),
            Error::MutexLockError { message } => write!(f, "Mutex lock error: {}", message),
            Error::Process { pid, message } => {
                if let Some(pid) = pid {
                    write!(f, "Process error (PID {}): {}", pid, message)
                } else {
                    write!(f, "Process error: {}", message)
                }
            },
            Error::Gpu { operation, message } => write!(f, "GPU error during {}: {}", operation, message),
            Error::InvalidArgument { context, value } => write!(f, "Invalid argument for {}: {}", context, value),
            Error::SystemError { operation, message } => write!(f, "System error during {}: {}", operation, message),
            Error::PermissionDenied { operation, required_permission } => {
                write!(f, "Permission denied for {}: requires {}", operation, required_permission)
            },
            Error::NotAvailable { resource, reason } => write!(f, "{} not available: {}", resource, reason),
            Error::Other { message } => write!(f, "{}", message),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError { source } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError { source: err }
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Error::InvalidData {
            message: "Invalid null character in string".to_string(),
            details: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_creation_methods() {
        let temp_err = Error::temperature_error("CPU", "Too hot");
        assert!(matches!(temp_err, Error::Temperature { .. }));

        let net_err = Error::network_error("eth0", "Link down");
        assert!(matches!(net_err, Error::Network { .. }));

        let proc_err = Error::process_error(Some(123), "Not responding");
        assert!(matches!(proc_err, Error::Process { .. }));

        let gpu_err = Error::gpu_error("render", "Out of memory");
        assert!(matches!(gpu_err, Error::Gpu { .. }));

        let arg_err = Error::invalid_argument("port", Some("65536"));
        assert!(matches!(arg_err, Error::InvalidArgument { .. }));

        let sys_err = Error::system_error("sysctl", "Invalid value");
        assert!(matches!(sys_err, Error::SystemError { .. }));
    }

    #[test]
    fn test_error_is_permission_denied() {
        let err = Error::PermissionDenied {
            operation: "read".into(),
            required_permission: "root".into(),
        };
        assert!(err.is_permission_denied());

        let other_err = Error::Other { message: "test".into() };
        assert!(!other_err.is_permission_denied());
    }

    #[test]
    fn test_error_is_not_available() {
        let err = Error::NotAvailable {
            resource: "GPU".into(),
            reason: "Not installed".into(),
        };
        assert!(err.is_not_available());

        let other_err = Error::Other { message: "test".into() };
        assert!(!other_err.is_not_available());
    }

    #[test]
    fn test_error_display() {
        let err = Error::Temperature {
            sensor: "CPU".into(),
            message: "Too hot".into(),
        };
        assert_eq!(err.to_string(), "Temperature error on sensor CPU: Too hot");

        let err = Error::Process {
            pid: Some(123),
            message: "Not responding".into(),
        };
        assert_eq!(err.to_string(), "Process error (PID 123): Not responding");

        let err = Error::InvalidData {
            message: "Invalid port".into(),
            details: "65536".into(),
        };
        assert_eq!(err.to_string(), "Invalid data: Invalid port (65536)");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::IoError { .. }));

        let nul_err = std::ffi::CString::new(vec![0]).unwrap_err();
        let err: Error = nul_err.into();
        assert!(matches!(err, Error::InvalidData { .. }));
    }
}
