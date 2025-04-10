/// Utility functions and modules for the darwin-metrics crate.
///
/// This module contains various utilities used throughout the crate, including:
///
/// - `bindings`: FFI bindings for macOS system APIs (sysctl, IOKit, etc.)
/// - `property_utils`: Utilities for working with property lists and dictionaries
/// - `test_utils`: Utilities for testing
/// - `mock_dictionary`: A pure Rust mock dictionary for testing
/// - `dictionary_access`: A trait for abstracting dictionary access operations
pub mod bindings;
#[cfg(test)]
mod bindings_tests;
pub mod dictionary_access;
pub mod mock_dictionary;
pub mod property_utils;
pub mod test_utils;

#[cfg(test)]
mod property_utils_tests;

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
/// - The memory range [ptr, ptr+(len*sizeof(f64))) is valid and contains initialized f64 values
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_c_str_to_string() {
        let test_str = "test string";
        let c_string = CString::new(test_str).unwrap();
        let ptr = c_string.as_ptr();

        unsafe {
            let result = c_str_to_string(ptr);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), test_str);

            // Test null pointer
            let result = c_str_to_string(std::ptr::null());
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_raw_str_to_string() {
        let test_str = "test string";
        let c_string = CString::new(test_str).unwrap();
        let ptr = c_string.as_ptr();
        let len = test_str.len();

        unsafe {
            let result = raw_str_to_string(ptr, len);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), test_str);

            // Test null pointer
            let result = raw_str_to_string(std::ptr::null(), len);
            assert!(result.is_none());

            // Test zero length
            let result = raw_str_to_string(ptr, 0);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_raw_f64_slice_to_vec() {
        let test_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ptr = test_data.as_ptr();
        let len = test_data.len();

        unsafe {
            let result = raw_f64_slice_to_vec(ptr, len);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), test_data);

            // Test null pointer
            let result = raw_f64_slice_to_vec(std::ptr::null(), len);
            assert!(result.is_none());

            // Test zero length
            let result = raw_f64_slice_to_vec(ptr, 0);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_autorelease_pool() {
        // Simple test to ensure the function works
        let value = autorelease_pool(|| 42);
        assert_eq!(value, 42);

        // Test with a string
        let str_value = autorelease_pool(|| "test string".to_string());
        assert_eq!(str_value, "test string");
    }

    #[test]
    fn test_objc_safe_exec() {
        // Test successful execution
        let result = objc_safe_exec(|| Ok::<_, Error>(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test with an error
        let result = objc_safe_exec(|| Err::<i32, Error>(Error::system("test error")));
        assert!(result.is_err());
        match result {
            Err(Error::System(msg)) => assert!(msg.contains("test error")),
            _ => panic!("Unexpected error type"),
        }

        // We can't easily test the panic case without actually panicking
    }

    #[test]
    fn test_get_name_null_device() {
        let result = get_name(std::ptr::null_mut());
        assert!(result.is_err());
        match result {
            Err(Error::NotAvailable(msg)) => assert!(msg.contains("No device available")),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_property_accessor() {
        // Test the struct can be created
        let _accessor = PropertyAccessor;
        // Simple sanity check - no need to test actual property access since we'd need a real Objective-C dictionary
    }
}
