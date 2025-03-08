pub mod property_utils;
pub use property_utils::PropertyUtils;
use crate::error::{Error, Result};

use std::ffi::{CStr, c_char};
use std::os::raw::c_double;
use std::slice;
use std::panic::AssertUnwindSafe;
use objc2::rc::autoreleasepool;
use objc2::msg_send;
use objc2::runtime::AnyObject;

pub fn objc_safe_exec<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        f()
    }));
    match result {
        Ok(value) => value,
        Err(_) => Err(Error::system("Panic occurred during Objective-C operation"))
    }
}

pub fn autorelease_pool<T, F>(f: F) -> T 
where 
    F: FnOnce() -> T
{
    autoreleasepool(|_| f())
}

#[allow(dead_code)]
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

#[allow(dead_code)]
pub unsafe fn raw_str_to_string(ptr: *const c_char, len: usize) -> Option<String> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    let slice = slice::from_raw_parts(ptr as *const u8, len);
    String::from_utf8(slice.to_vec()).ok()
}

#[allow(dead_code)]
pub unsafe fn raw_f64_slice_to_vec(ptr: *const c_double, len: usize) -> Option<Vec<f64>> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    Some(slice::from_raw_parts(ptr, len).to_vec())
}

pub fn get_name(device: *mut std::ffi::c_void) -> Result<String> {
    if device.is_null() {
        return Err(Error::not_available("No GPU device available"));
    }
    
    autorelease_pool(|| {
        objc_safe_exec(|| {
            unsafe {
                let device_obj: *mut AnyObject = device.cast();
                
                let name_obj: *mut AnyObject = msg_send![device_obj, name];
                
                if name_obj.is_null() {
                    return Err(Error::not_available("Could not get GPU name".to_string()));
                }
                
                let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
                if utf8_string.is_null() {
                    return Err(Error::not_available("Could not convert GPU name to string".to_string()));
                }
                
                let c_str = std::ffi::CStr::from_ptr(utf8_string as *const i8);
                let name = c_str.to_string_lossy().into_owned();
                
                Ok(name)
            }
        })
    })
}
