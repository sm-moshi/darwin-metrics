#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("System call error: {0}")]
    System(String),

    #[error("Swift FFI error: {0}")]
    SwiftFFI(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Feature not available: {0}")]
    NotAvailable(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl Error {
    pub(crate) fn system<S: Into<String>>(msg: S) -> Self {
        Error::System(msg.into())
    }

    pub(crate) fn swift_ffi<S: Into<String>>(msg: S) -> Self {
        Error::SwiftFFI(msg.into())
    }

    pub(crate) fn invalid_data<S: Into<String>>(msg: S) -> Self {
        Error::InvalidData(msg.into())
    }

    pub(crate) fn not_available<S: Into<String>>(msg: S) -> Self {
        Error::NotAvailable(msg.into())
    }

    pub(crate) fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Error::PermissionDenied(msg.into())
    }
} 