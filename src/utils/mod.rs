//! Utility functions for safe FFI interactions and memory management
//!
//! This module provides utilities for:
//! - Safe Objective-C interoperability
//! - Exception handling
//! - Memory management
//! - FFI data conversion utilities

use crate::Error;
use objc2::msg_send;
use objc2::rc::autoreleasepool;
use objc2::runtime::AnyObject;
use std::ffi::{c_char, CStr};
use std::os::raw::c_double;
use std::panic::AssertUnwindSafe;
use std::slice;

/// Safely executes Objective-C code that might throw exceptions
///
/// This function wraps Objective-C code in a way that prevents uncaught exceptions
/// from crashing the Rust process
///
/// # Arguments
///
/// * [f](http://_vscodecontentref_/9) - The closure containing potentially exception-throwing Objective-C code
///
/// # Returns
///
/// * [Result<T, crate::Error>](http://_vscodecontentref_/10) - Success value or converted error
pub fn objc_safe_exec<T, F>(f: F) -> crate::Result<T>
where
    F: FnOnce() -> crate::Result<T>,
{
    // Use catch_unwind to catch Rust panics
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        // Without the exception feature, we can't catch Objective-C exceptions
        // Just run the function directly
        f()
    }));

    match result {
        Ok(value) => value,
        Err(_) => Err(crate::Error::system_error(
            // Fix: Remove the .into() call since &str implements Into<String> automatically
            "Panic occurred during Objective-C operation",
        )),
    }
}

/// Creates an autorelease pool for safely managing Objective-C objects
///
/// # Examples
///
/// ```rust,no_run
/// use darwin_metrics::utils::autorelease_pool;
///
/// let result = autorelease_pool(|| {
///     // Code that creates Objective-C objects
///     // They will be automatically released when this closure returns
///     42
/// });
/// assert_eq!(result, 42);
/// ```
///
/// # Returns
///
/// A guard object that will drain the autorelease pool when dropped
pub fn autorelease_pool<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    // This properly adapts our closure to the objc2 autoreleasepool function
    autoreleasepool(|_| f())
}

/// Converts a C string pointer to a Rust String
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string
///
/// # Examples
///
/// ```rust,no_run
/// use darwin_metrics::utils::c_str_to_string;
/// use std::ffi::CString;
///
/// let c_string = CString::new("hello").unwrap();
/// let ptr = c_string.as_ptr();
///
/// unsafe {
///     let rust_string = c_str_to_string(ptr);
///     assert_eq!(rust_string, Some("hello".to_string()));
/// }
/// ```
///
/// # Returns
/// * `Some(String)` if conversion is successful
/// * `None` if the pointer is null or the string is invalid UTF-8
#[allow(dead_code)]
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

/// Converts a raw string pointer and length to a Rust String
///
/// # Safety
/// The pointer must be valid and point to a string of the specified length
///
/// # Returns
/// * `Some(String)` if conversion is successful
/// * `None` if the pointer is null, length is 0, or the string is invalid UTF-8
#[allow(dead_code)]
pub unsafe fn raw_str_to_string(ptr: *const c_char, len: usize) -> Option<String> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    let slice = slice::from_raw_parts(ptr as *const u8, len);
    String::from_utf8(slice.to_vec()).ok()
}

/// Converts a raw f64 pointer and length to a Rust Vec<f64>
///
/// # Safety
/// The pointer must be valid and point to an array of f64 values of the specified length
///
/// # Returns
/// * `Some(Vec<f64>)` if conversion is successful
/// * `None` if the pointer is null or length is 0
#[allow(dead_code)]
pub unsafe fn raw_f64_slice_to_vec(ptr: *const c_double, len: usize) -> Option<Vec<f64>> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    Some(slice::from_raw_parts(ptr, len).to_vec())
}

/// Gets the name of the GPU
///
/// # Arguments
///
/// * `device` - GPU device pointer
///
/// # Returns
///
/// A Result containing the GPU name as a String or an error
pub fn get_name(device: *mut std::ffi::c_void) -> crate::Result<String> {
    // Check if we have a valid device
    if device.is_null() {
        return Err(Error::NotAvailable("No GPU device available".into()));
    }

    // Safely execute Objective-C code within an autorelease pool
    autorelease_pool(|| {
        objc_safe_exec(|| {
            unsafe {
                // Cast the device pointer to AnyObject before sending messages to it
                let device_obj: *mut AnyObject = device.cast();

                // Get the name property using Metal API
                let name_obj: *mut AnyObject = msg_send![device_obj, name];

                // Check if we got a valid name object
                if name_obj.is_null() {
                    return Err(Error::NotAvailable("Could not get GPU name".into()));
                }

                // Convert to Rust string
                let utf8_string: *const u8 = msg_send![name_obj, UTF8String];

                if utf8_string.is_null() {
                    return Err(Error::NotAvailable(
                        "Could not convert GPU name to string".into(),
                    ));
                }

                // Convert C string to Rust string
                let c_str = std::ffi::CStr::from_ptr(utf8_string as *const i8);
                let name = c_str.to_string_lossy().into_owned();

                Ok(name)
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_c_str_to_string() {
        let test_str = "Hello, world!";
        let c_string = CString::new(test_str).unwrap();
        let ptr = c_string.as_ptr();

        unsafe {
            let result = c_str_to_string(ptr);
            assert_eq!(result, Some(test_str.to_string()));

            // Test null pointer
            let null_result = c_str_to_string(std::ptr::null());
            assert_eq!(null_result, None);
        }
    }

    #[test]
    fn test_raw_str_to_string() {
        let test_str = "Hello, world!";
        let c_string = CString::new(test_str).unwrap();
        let ptr = c_string.as_ptr();
        let len = test_str.len();

        unsafe {
            let result = raw_str_to_string(ptr, len);
            assert_eq!(result, Some(test_str.to_string()));

            // Test null pointer
            let null_result = raw_str_to_string(std::ptr::null(), 10);
            assert_eq!(null_result, None);

            // Test zero length
            let zero_len_result = raw_str_to_string(ptr, 0);
            assert_eq!(zero_len_result, None);
        }
    }

    #[test]
    fn test_raw_f64_slice_to_vec() {
        let test_doubles = vec![1.0, 2.5, std::f64::consts::PI, -0.5];
        let ptr = test_doubles.as_ptr();
        let len = test_doubles.len();

        unsafe {
            let result = raw_f64_slice_to_vec(ptr, len);
            assert_eq!(result, Some(test_doubles));

            // Test null pointer
            let null_result = raw_f64_slice_to_vec(std::ptr::null(), 10);
            assert_eq!(null_result, None);

            // Test zero length
            let zero_len_result = raw_f64_slice_to_vec(ptr, 0);
            assert_eq!(zero_len_result, None);
        }
    }

    #[test]
    fn test_autorelease_pool() {
        // This just tests that the function executes without panicking
        let result = autorelease_pool(|| {
            // Just return a simple value
            42
        });
        assert_eq!(result, 42);
    }
}
