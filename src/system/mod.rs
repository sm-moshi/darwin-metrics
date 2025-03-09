use crate::error::{Error, Result};
use crate::utils::bindings::{
    sysctl,
    sysctl_constants::{CTL_HW, HW_MACHINE},
};
use std::ffi::c_void;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArchitectureError {
    #[error("System call failed")]
    SystemCallFailed,
    #[error("Invalid string encoding")]
    InvalidStringEncoding,
}

impl From<ArchitectureError> for Error {
    fn from(err: ArchitectureError) -> Self {
        Error::system(err.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum Architecture {
    Intel,
    AppleSilicon,
    Unknown,
}

pub fn detect_architecture() -> Result<Architecture> {
    let mut mib = [CTL_HW, HW_MACHINE];
    let mut size = 0;

    unsafe {
        if sysctl(
            mib.as_mut_ptr(),
            2,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null(),
            0,
        ) != 0
        {
            return Err(Error::system("Failed to get architecture information"));
        }

        let mut buffer = vec![0u8; size];
        if sysctl(
            mib.as_mut_ptr(),
            2,
            buffer.as_mut_ptr() as *mut c_void,
            &mut size,
            std::ptr::null(),
            0,
        ) != 0
        {
            return Err(Error::system("Failed to retrieve architecture data"));
        }

        let cstr = std::ffi::CStr::from_bytes_with_nul(&buffer)
            .map_err(|_| ArchitectureError::InvalidStringEncoding)?;
        let arch = cstr
            .to_str()
            .map_err(|_| ArchitectureError::InvalidStringEncoding)?;

        Ok(match arch {
            "arm64" => Architecture::AppleSilicon,
            "x86_64" => Architecture::Intel,
            _ => Architecture::Unknown,
        })
    }
}

pub struct SystemMetrics {
    pub architecture: Architecture,
}

pub fn get_system_metrics() -> Result<SystemMetrics> {
    let architecture = detect_architecture()?;
    Ok(SystemMetrics { architecture })
}
