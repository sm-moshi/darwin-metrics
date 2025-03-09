//! Utility functions and modules for the darwin-metrics crate.
//!
//! This module contains various utilities used throughout the crate, including:
//!
//! - `bindings`: FFI bindings for macOS system APIs (sysctl, IOKit, etc.)
//! - `property_utils`: Utilities for working with property lists and
//!   dictionaries
//! - `test_utils`: Utilities for testing

// Conditional compilation flag for docs.rs environment
#[cfg(any(docsrs, use_stubs, not(target_os = "macos")))]
pub const IS_DOCS_RS: bool = true;
#[cfg(not(any(docsrs, use_stubs, not(target_os = "macos"))))]
pub const IS_DOCS_RS: bool = false;

pub mod bindings;
pub mod property_utils;
pub mod test_utils;

use std::{
    ffi::{c_char, CStr},
    os::raw::c_double,
    panic::AssertUnwindSafe,
    slice,
};

use objc2::{
    msg_send,
    rc::{autoreleasepool, Retained},
    runtime::AnyObject,
};
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

use crate::error::{Error, Result};

pub trait PropertyUtils {
    fn get_string_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSString>().ok())
            .map(|s| s.to_string())
    }

    fn get_number_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n: Retained<NSNumber>| n.as_f64())
    }

    fn get_bool_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n: Retained<NSNumber>| n.as_bool())
    }
}

pub struct PropertyAccessor;

impl PropertyUtils for PropertyAccessor {}

/// Executes a closure safely within an Objective-C autorelease pool.
pub fn autorelease_pool<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    autoreleasepool(|_| f())
}

/// Safely executes Objective-C code, catching any Rust panics.
pub fn objc_safe_exec<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let result = std::panic::catch_unwind(AssertUnwindSafe(f));
    match result {
        Ok(value) => value,
        Err(_) => Err(Error::System("Panic occurred during Objective-C operation".to_string())),
    }
}

/// Converts a C string pointer to a Rust `String`.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer is valid and properly aligned
/// - The C string is properly null-terminated
/// - The C string contains valid UTF-8 data
/// - The pointer remains valid for the duration of this function call
///
/// This function will return None if the pointer is null.
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(String::from)
}

/// Converts a raw string pointer and length to a `String`.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer is valid and properly aligned
/// - The memory range [ptr, ptr+len) is valid and contains initialized data
/// - The memory range contains valid UTF-8 data
/// - The pointer remains valid for the duration of this function call
/// - No other code will concurrently modify the memory being accessed
///
/// This function will return None if the pointer is null or length is zero.
pub unsafe fn raw_str_to_string(ptr: *const c_char, len: usize) -> Option<String> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    let slice = slice::from_raw_parts(ptr as *const u8, len);
    String::from_utf8(slice.to_vec()).ok()
}

/// Converts a raw f64 slice pointer and length into a `Vec<f64>`.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer is valid and properly aligned for f64 values
/// - The memory range [ptr, ptr+(len*sizeof(f64))) is valid and contains
///   initialized f64 values
/// - The pointer remains valid for the duration of this function call
/// - No other code will concurrently modify the memory being accessed
///
/// This function will return None if the pointer is null or length is zero.
pub unsafe fn raw_f64_slice_to_vec(ptr: *const c_double, len: usize) -> Option<Vec<f64>> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    Some(slice::from_raw_parts(ptr, len).to_vec())
}

/// Retrieves the name of an Objective-C device.
pub fn get_name(device: *mut std::ffi::c_void) -> Result<String> {
    if device.is_null() {
        return Err(Error::NotAvailable("No device available".to_string()));
    }

    autorelease_pool(|| {
        objc_safe_exec(|| unsafe {
            let device_obj: *mut AnyObject = device.cast();
            let name_obj: *mut AnyObject = msg_send![device_obj, name];

            if name_obj.is_null() {
                return Err(Error::NotAvailable("Could not get device name".to_string()));
            }

            let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
            if utf8_string.is_null() {
                return Err(Error::NotAvailable("Could not convert name to string".to_string()));
            }

            let c_str = CStr::from_ptr(utf8_string as *const i8);
            Ok(c_str.to_string_lossy().into_owned())
        })
    })
}
