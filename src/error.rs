use std::ffi::NulError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IOKit error: {0}")]
    IOKit(String),
    
    #[error("Metal error: {0}")]
    Metal(String),
    
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Feature not available: {0}")]
    NotAvailable(String),
    
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    
    #[error("System error: {0}")]
    System(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Temperature sensor error: {0}")]
    Temperature(String),
    
    #[error("Disk error: {0}")]
    Disk(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error(transparent)]
    NulError(#[from] NulError),
    
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn io_kit(msg: impl Into<String>) -> Self {
        Self::IOKit(msg.into())
    }
    
    pub fn metal(msg: impl Into<String>) -> Self {
        Self::Metal(msg.into())
    }
    
    pub fn service_not_found(msg: impl Into<String>) -> Self {
        Self::ServiceNotFound(msg.into())
    }
    
    pub fn not_available(msg: impl Into<String>) -> Self {
        Self::NotAvailable(msg.into())
    }
    
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Self::NotImplemented(msg.into())
    }
    
    pub fn system(msg: impl Into<String>) -> Self {
        Self::System(msg.into())
    }
    
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }
    
    pub fn process_error(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
