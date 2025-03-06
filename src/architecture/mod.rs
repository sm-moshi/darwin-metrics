use std::ffi::c_void;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArchitectureError {
    #[error("System call failed")]
    SystemCallFailed,
    #[error("Invalid string encoding")]
    InvalidStringEncoding,
}

#[derive(Debug, PartialEq)]
pub enum Architecture {
    Intel,
    AppleSilicon,
    Unknown,
}

#[link(name = "System", kind = "framework")]
extern "C" {
    fn sysctl(
        name: *const i32,
        namelen: u32,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> i32;
}

const CTL_HW: i32 = 6;
const HW_MACHINE: i32 = 1;

pub fn detect_architecture() -> Result<Architecture, ArchitectureError> {
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
            return Err(ArchitectureError::SystemCallFailed);
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
            return Err(ArchitectureError::SystemCallFailed);
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
