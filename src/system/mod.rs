use std::ffi::c_void;

use thiserror::Error;

use crate::{
    error::{Error, Result},
    utils::bindings::{
        sysctl,
        sysctl_constants::{CTL_HW, HW_MACHINE},
    },
};

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
        if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
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
        let arch = cstr.to_str().map_err(|_| ArchitectureError::InvalidStringEncoding)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_error_from() {
        // Test conversion from ArchitectureError to Error
        let arch_error = ArchitectureError::SystemCallFailed;
        let error: Error = arch_error.into();
        assert!(matches!(error, Error::System(_)));

        let arch_error = ArchitectureError::InvalidStringEncoding;
        let error: Error = arch_error.into();
        assert!(matches!(error, Error::System(_)));
    }

    #[test]
    fn test_architecture_enum() {
        // Test enum equality
        assert_eq!(Architecture::Intel, Architecture::Intel);
        assert_eq!(Architecture::AppleSilicon, Architecture::AppleSilicon);
        assert_eq!(Architecture::Unknown, Architecture::Unknown);

        // Test enum inequality
        assert_ne!(Architecture::Intel, Architecture::AppleSilicon);
        assert_ne!(Architecture::Intel, Architecture::Unknown);
        assert_ne!(Architecture::AppleSilicon, Architecture::Unknown);
    }

    #[test]
    fn test_detect_architecture() {
        // This should work on any Mac
        let result = detect_architecture();
        assert!(result.is_ok(), "Architecture detection should succeed");

        let arch = result.unwrap();
        // Architecture should be either Intel or AppleSilicon on real Mac hardware
        assert!(
            matches!(arch, Architecture::Intel | Architecture::AppleSilicon),
            "Architecture should be Intel or AppleSilicon, got: {arch:?}"
        );

        // On modern Macs, we expect aarch64/Apple Silicon or x86_64/Intel
        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch, Architecture::AppleSilicon, "On aarch64, should detect Apple Silicon");

        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch, Architecture::Intel, "On x86_64, should detect Intel");
    }

    #[test]
    fn test_get_system_metrics() {
        let result = get_system_metrics();
        assert!(result.is_ok(), "System metrics retrieval should succeed");

        let metrics = result.unwrap();
        // Verify that the architecture field is populated
        assert!(
            matches!(
                metrics.architecture,
                Architecture::Intel | Architecture::AppleSilicon | Architecture::Unknown
            ),
            "Architecture should be a valid value"
        );
    }

    #[test]
    fn test_system_metrics_struct() {
        // Test creating the struct directly
        let metrics = SystemMetrics { architecture: Architecture::Intel };
        assert_eq!(metrics.architecture, Architecture::Intel);

        let metrics = SystemMetrics { architecture: Architecture::AppleSilicon };
        assert_eq!(metrics.architecture, Architecture::AppleSilicon);
    }
}
