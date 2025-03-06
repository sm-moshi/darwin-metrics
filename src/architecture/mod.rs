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

#[derive(Debug)]
pub struct SystemInfo {
    pub architecture: Architecture,
    pub model_identifier: String,
    pub processor_name: String,
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
const HW_MODEL: i32 = 2;

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

pub fn get_system_info() -> Result<SystemInfo, ArchitectureError> {
    let architecture = detect_architecture()?;
    let model_identifier = get_model_identifier()?;
    let processor_name = get_processor_name()?;

    Ok(SystemInfo {
        architecture,
        model_identifier,
        processor_name,
    })
}

fn get_model_identifier() -> Result<String, ArchitectureError> {
    let mut mib = [CTL_HW, HW_MODEL];
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
        Ok(cstr.to_string_lossy().into_owned())
    }
}

fn get_processor_name() -> Result<String, ArchitectureError> {
    let mib = [CTL_HW, 25]; // HW_MODEL
    let mut size = 0;

    unsafe {
        if sysctl(
            mib.as_ptr(),
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
            mib.as_ptr(),
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
        Ok(cstr.to_string_lossy().into_owned())
    }
}
