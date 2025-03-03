use std::ffi::{CStr, c_char};
use std::os::raw::c_double;
use std::slice;

/// Converts a C string pointer to a Rust String
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string
///
/// # Returns
/// * `Some(String)` if conversion is successful
/// * `None` if the pointer is null or the string is invalid UTF-8
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}

/// Converts a raw string pointer and length to a Rust String
///
/// # Safety
/// The pointer must be valid and point to a string of the specified length
///
/// # Returns
/// * `Some(String)` if conversion is successful
/// * `None` if the pointer is null, length is 0, or the string is invalid UTF-8
pub unsafe fn raw_str_to_string(ptr: *const c_char, len: usize) -> Option<String> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    unsafe {
        let slice = slice::from_raw_parts(ptr as *const u8, len);
        String::from_utf8(slice.to_vec()).ok()
    }
}

/// Converts a raw f64 pointer and length to a Rust Vec<f64>
///
/// # Safety
/// The pointer must be valid and point to an array of f64 values of the specified length
///
/// # Returns
/// * `Some(Vec<f64>)` if conversion is successful
/// * `None` if the pointer is null or length is 0
pub unsafe fn raw_f64_slice_to_vec(ptr: *const c_double, len: usize) -> Option<Vec<f64>> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    unsafe { Some(slice::from_raw_parts(ptr, len).to_vec()) }
}
