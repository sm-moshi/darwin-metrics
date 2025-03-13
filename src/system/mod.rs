use std::ffi::c_void;

use thiserror::Error;

use crate::{
    error::{Error, Result},
    utils::bindings::{
        sysctl,
        sysctl_constants::{CTL_HW, HW_MACHINE, CTL_KERN, KERN_HOSTNAME},
    },
};

/// Errors that can occur during architecture detection
///
/// This enum represents various error conditions that may arise when attempting to detect the system architecture.
#[derive(Debug, Error)]
pub enum ArchitectureError {
    /// A system call failed
    #[error("System call failed")]
    SystemCallFailed,
    /// String encoding error occurred
    #[error("Invalid string encoding")]
    InvalidStringEncoding,
}

impl From<ArchitectureError> for Error {
    fn from(err: ArchitectureError) -> Self {
        Error::system(err.to_string())
    }
}

/// System architecture type
///
/// This enum represents the different CPU architectures that can be detected on macOS systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    /// Intel x86_64 architecture
    Intel,
    /// Apple Silicon ARM architecture
    AppleSilicon,
    /// Unknown architecture
    Unknown,
}

/// Detects the current system architecture
///
/// Returns the detected CPU architecture of the system.
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

/// System metrics information
///
/// This struct holds various system-wide metrics and information.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// The detected CPU architecture
    pub architecture: Architecture,
}

/// Gets current system metrics
///
/// Returns a struct containing various system-wide metrics.
pub fn get_system_metrics() -> Result<SystemMetrics> {
    let architecture = detect_architecture()?;
    Ok(SystemMetrics { architecture })
}

/// System information and monitoring functionality
///
/// This struct provides access to system-wide information and metrics.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// The detected CPU architecture
    pub architecture: Architecture,
    /// The system hostname
    pub hostname: String,
}

impl SystemInfo {
    /// Creates a new SystemInfo instance
    pub fn new() -> Result<Self> {
        let mut info = Self {
            architecture: detect_architecture()?,
            hostname: String::new(),
        };
        info.update()?;
        Ok(info)
    }

    /// Updates system information
    pub fn update(&mut self) -> Result<()> {
        self.architecture = detect_architecture()?;
        
        // Get hostname
        let mut mib = [CTL_KERN, KERN_HOSTNAME];
        let mut size = 0;
        
        unsafe {
            if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(Error::system("Failed to get hostname size"));
            }

            let mut buffer = vec![0u8; size];
            if sysctl(
                mib.as_mut_ptr(),
                2,
                buffer.as_mut_ptr() as *mut c_void,
                &mut size,
                std::ptr::null(),
                0,
            ) != 0 {
                return Err(Error::system("Failed to retrieve hostname"));
            }

            // Convert to string, trimming null terminator
            if let Ok(hostname) = String::from_utf8(buffer[..size-1].to_vec()) {
                self.hostname = hostname;
            } else {
                return Err(Error::system("Invalid hostname encoding"));
            }
        }
        
        Ok(())
    }

    /// Returns the system hostname
    pub fn hostname(&self) -> &str {
        &self.hostname
    }
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

    #[test]
    fn test_system_info() {
        let info = SystemInfo::new().unwrap();
        
        // Verify architecture
        assert!(matches!(
            info.architecture,
            Architecture::Intel | Architecture::AppleSilicon | Architecture::Unknown
        ));

        // Verify hostname is not empty
        assert!(!info.hostname().is_empty(), "Hostname should not be empty");
        assert_eq!(info.hostname(), info.hostname);
    }
}
