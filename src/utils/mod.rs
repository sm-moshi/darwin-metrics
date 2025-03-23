//! Utility functions and modules for the darwin-metrics crate.
//!
//! This module provides various utilities used throughout the crate:
//!
//! - Core functionality for dictionary access and property manipulation
//! - FFI bindings and safe abstractions for macOS system APIs
//! - Conversion traits and implementations for FFI types
//! - Testing utilities and mocks
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::utils::{autorelease_pool, DictionaryAccess};
//!
//! // Use autorelease pool for safe Objective-C memory management
//! let result = autorelease_pool(|| {
//!     // Your Objective-C interop code here
//!     Ok(())
//! });
//! ```

/// Core utility functions and data structures
pub mod core;
/// FFI bindings and utilities for interacting with macOS system APIs
pub mod ffi;

// Re-export bindings module
pub use ffi::bindings;

// Test utilities - make public for both test and mock feature flags
#[cfg(any(test, feature = "testing", feature = "mock"))]
pub mod tests;

// Re-export core utilities
pub use core::dictionary::SafeDictionary;
// Selectively re-export FFI types and functions

// Re-export sysctl constants
use std::{
    ffi::{CStr, c_char},
    os::raw::c_double,
    panic::{self, AssertUnwindSafe, UnwindSafe},
    slice,
};

use objc2::msg_send;
use objc2::rc::autoreleasepool;
use objc2::runtime::AnyObject;

use crate::error::{Error, Result};

/// A trait for safely handling unsafe conversions from raw pointers
///
/// This trait provides a safe interface for converting from various raw pointer types
/// to Rust types, handling null pointers and validation appropriately.
pub trait UnsafeConversion<T> {
    /// Performs the conversion from the raw type to the target type
    ///
    /// # Safety
    ///
    /// Implementations must document their safety requirements and ensure
    /// all invariants are maintained.
    unsafe fn convert(&self) -> Option<T>;
}

/// A wrapper for C string conversions
#[derive(Debug)]
pub struct CStringConversion<'a> {
    ptr: *const c_char,
    _phantom: std::marker::PhantomData<&'a c_char>,
}

impl CStringConversion<'_> {
    /// Creates a new CStringConversion
    ///
    /// # Safety
    ///
    /// The pointer must be:
    /// - Valid for reads
    /// - Properly aligned
    /// - Pointing to a null-terminated C string
    /// - Valid for the lifetime 'a
    pub unsafe fn new(ptr: *const c_char) -> Self {
        Self {
            ptr,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> UnsafeConversion<String> for CStringConversion<'a> {
    unsafe fn convert(&self) -> Option<String> {
        unsafe { CStr::from_ptr(self.ptr).to_str().ok().map(String::from) }
    }
}

/// A wrapper for raw string slice conversions
#[derive(Debug)]
pub struct RawStrConversion<'a> {
    ptr: *const c_char,
    len: usize,
    _phantom: std::marker::PhantomData<&'a c_char>,
}

impl RawStrConversion<'_> {
    /// Creates a new RawStrConversion
    ///
    /// # Safety
    ///
    /// The pointer must be:
    /// - Valid for reads
    /// - Properly aligned
    /// - Valid for `len` bytes
    /// - Pointing to valid UTF-8 data
    /// - Valid for the lifetime 'a
    pub unsafe fn new(ptr: *const c_char, len: usize) -> Self {
        Self {
            ptr,
            len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> UnsafeConversion<String> for RawStrConversion<'a> {
    unsafe fn convert(&self) -> Option<String> {
        let slice = unsafe { slice::from_raw_parts(self.ptr as *const u8, self.len) };
        String::from_utf8(slice.to_vec()).ok()
    }
}

/// A wrapper for f64 slice conversions
#[derive(Debug)]
pub struct F64SliceConversion<'a> {
    ptr: *const c_double,
    len: usize,
    _phantom: std::marker::PhantomData<&'a c_double>,
}

impl F64SliceConversion<'_> {
    /// Creates a new F64SliceConversion
    ///
    /// # Safety
    ///
    /// The pointer must be:
    /// - Valid for reads
    /// - Properly aligned for f64
    /// - Valid for `len * size_of::<f64>()` bytes
    /// - Valid for the lifetime 'a
    pub unsafe fn new(ptr: *const c_double, len: usize) -> Self {
        Self {
            ptr,
            len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> UnsafeConversion<Vec<f64>> for F64SliceConversion<'a> {
    unsafe fn convert(&self) -> Option<Vec<f64>> {
        Some(unsafe { slice::from_raw_parts(self.ptr, self.len).to_vec() })
    }
}

/// Executes a closure safely within an Objective-C autorelease pool.
pub fn autorelease_pool<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    autoreleasepool(|_| f())
}

/// Safely executes Objective-C code, catching any Rust panics.
pub fn catch_unwind_result<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::system_error("Panic occurred during Objective-C operation")),
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
    unsafe { CStringConversion::new(ptr).convert() }
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
    unsafe { RawStrConversion::new(ptr, len).convert() }
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
    unsafe { F64SliceConversion::new(ptr, len).convert() }
}

/// Retrieves the name of an Objective-C device.
pub fn get_name(device: *mut std::ffi::c_void) -> Result<String> {
    if device.is_null() {
        return Err(Error::not_available("No device available"));
    }

    autorelease_pool(|| {
        catch_unwind_result(|| unsafe {
            let device_obj: *mut AnyObject = device.cast();
            let name_obj: *mut AnyObject = msg_send![device_obj, name];

            if name_obj.is_null() {
                return Err(Error::not_available("Device name not available"));
            }

            let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
            if utf8_string.is_null() {
                return Err(Error::not_available("Device name UTF8String not available"));
            }

            let c_str = CStr::from_ptr(utf8_string as *const i8);
            Ok(c_str.to_string_lossy().into_owned())
        })
    })?
}
