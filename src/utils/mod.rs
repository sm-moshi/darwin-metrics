/// Utility functions and modules for the darwin-metrics crate.
///
/// This module contains various utilities used throughout the crate, including:
///
/// - `bindings`: FFI bindings for macOS system APIs (sysctl, IOKit, etc.)
/// - `property_utils`: Utilities for working with property lists and dictionaries
/// - `test_utils`: Utilities for testing
/// - `mock_dictionary`: A pure Rust mock dictionary for testing
/// - `dictionary_access`: A trait for abstracting dictionary access operations

/// FFI bindings for macOS system APIs
pub mod bindings;

#[cfg(test)]
mod bindings_tests;

/// Dictionary access utilities for working with macOS dictionaries
pub mod dictionary_access;

/// Mock dictionary implementation for testing
pub mod mock_dictionary;

/// Property access utilities for working with macOS properties
pub mod property_utils;

/// Test utilities for mocking and testing macOS functionality
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

impl<'a> CStringConversion<'a> {
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
        if self.ptr.is_null() {
            return None;
        }
        CStr::from_ptr(self.ptr).to_str().ok().map(String::from)
    }
}

/// A wrapper for raw string slice conversions
#[derive(Debug)]
pub struct RawStrConversion<'a> {
    ptr: *const c_char,
    len: usize,
    _phantom: std::marker::PhantomData<&'a c_char>,
}

impl<'a> RawStrConversion<'a> {
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
        if self.ptr.is_null() || self.len == 0 {
            return None;
        }
        let slice = slice::from_raw_parts(self.ptr as *const u8, self.len);
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

impl<'a> F64SliceConversion<'a> {
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
        if self.ptr.is_null() || self.len == 0 {
            return None;
        }
        Some(slice::from_raw_parts(self.ptr, self.len).to_vec())
    }
}

/// Utility trait for accessing properties from macOS dictionaries
///
/// This trait provides methods for safely accessing string, number, and boolean properties from NSDictionary objects,
/// handling type conversions and null values.
pub trait PropertyUtils {
    /// Get a string property from a dictionary
    ///
    /// # Arguments
    /// * `dict` - The dictionary to get the property from
    /// * `key` - The key to look up in the dictionary
    ///
    /// # Returns
    /// * `Option<String>` - The string value if found and valid, None otherwise
    fn get_string_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String>;

    /// Get a number property from a dictionary
    ///
    /// # Arguments
    /// * `dict` - The dictionary to get the property from
    /// * `key` - The key to look up in the dictionary
    ///
    /// # Returns
    /// * `Option<f64>` - The number value if found and valid, None otherwise
    fn get_number_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64>;

    /// Get a boolean property from a dictionary
    ///
    /// # Arguments
    /// * `dict` - The dictionary to get the property from
    /// * `key` - The key to look up in the dictionary
    ///
    /// # Returns
    /// * `Option<bool>` - The boolean value if found and valid, None otherwise
    fn get_bool_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool>;
}

/// Property access implementation for macOS dictionaries
#[derive(Debug)]
pub struct PropertyAccessor;

impl PropertyUtils for PropertyAccessor {
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
    CStringConversion::new(ptr).convert()
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
    RawStrConversion::new(ptr, len).convert()
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
    F64SliceConversion::new(ptr, len).convert()
}

/// Retrieves the name of an Objective-C device.
pub fn get_name(device: *mut std::ffi::c_void) -> Result<String> {
    if device.is_null() {
        return Err(Error::NotAvailable {
            resource: "device".to_string(),
            reason: "No device available".to_string(),
        });
    }

    autorelease_pool(|| {
        objc_safe_exec(|| unsafe {
            let device_obj: *mut AnyObject = device.cast();
            let name_obj: *mut AnyObject = msg_send![device_obj, name];

            if name_obj.is_null() {
                return Err(Error::NotAvailable {
                    resource: "device".to_string(),
                    reason: "Could not get device name".to_string(),
                });
            }

            let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
            if utf8_string.is_null() {
                return Err(Error::NotAvailable {
                    resource: "device".to_string(),
                    reason: "Could not convert name to string".to_string(),
                });
            }

            let c_str = CStr::from_ptr(utf8_string as *const i8);
            Ok(c_str.to_string_lossy().into_owned())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        dictionary_access::DictionaryAccessor,
        mock_dictionary::{MockDictionary, MockValue},
    };
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
            Err(Error::NotAvailable { resource, reason }) => {
                assert!(reason.contains("No device available"))
            },
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_property_accessor() {
        // Test the struct can be created
        let _accessor = PropertyAccessor;
        // Simple sanity check - no need to test actual property access since we'd need a real Objective-C dictionary
    }

    #[test]
    fn test_dictionary_access_string_property() {
        let mock_dict = MockDictionary::with_entries(&[
            ("test_key", MockValue::String("test_value".to_string())),
            ("empty_key", MockValue::String("".to_string())),
        ]);
        let accessor = DictionaryAccessor::new(mock_dict);

        let result = accessor.get_string("test_key");
        assert_eq!(result, Some("test_value".to_string()));

        let empty_result = accessor.get_string("empty_key");
        assert_eq!(empty_result, Some("".to_string()));

        let missing_result = accessor.get_string("nonexistent_key");
        assert_eq!(missing_result, None);
    }

    #[test]
    fn test_dictionary_access_number_property() {
        let mock_dict = MockDictionary::with_entries(&[
            ("int_key", MockValue::Number(42.0)),
            ("float_key", MockValue::Number(std::f64::consts::PI)),
            ("zero_key", MockValue::Number(0.0)),
        ]);
        let accessor = DictionaryAccessor::new(mock_dict);

        let int_result = accessor.get_number("int_key");
        assert_eq!(int_result, Some(42.0));

        let float_result = accessor.get_number("float_key");
        assert_eq!(float_result, Some(std::f64::consts::PI));

        let zero_result = accessor.get_number("zero_key");
        assert_eq!(zero_result, Some(0.0));

        let missing_result = accessor.get_number("nonexistent_key");
        assert_eq!(missing_result, None);
    }

    #[test]
    fn test_dictionary_access_bool_property() {
        let mock_dict = MockDictionary::with_entries(&[
            ("true_key", MockValue::Boolean(true)),
            ("false_key", MockValue::Boolean(false)),
        ]);
        let accessor = DictionaryAccessor::new(mock_dict);

        let true_result = accessor.get_bool("true_key");
        assert_eq!(true_result, Some(true));

        let false_result = accessor.get_bool("false_key");
        assert_eq!(false_result, Some(false));

        let missing_result = accessor.get_bool("nonexistent_key");
        assert_eq!(missing_result, None);
    }

    #[test]
    fn test_dictionary_access_mixed_types() {
        let mock_dict = MockDictionary::with_entries(&[
            ("string_key", MockValue::String("string_value".to_string())),
            ("number_key", MockValue::Number(42.0)),
            ("bool_key", MockValue::Boolean(true)),
        ]);
        let accessor = DictionaryAccessor::new(mock_dict);

        // Test getting string as number
        let string_as_number = accessor.get_number("string_key");
        assert_eq!(string_as_number, None);

        // Test getting number as string
        let number_as_string = accessor.get_string("number_key");
        assert_eq!(number_as_string, None);

        // Test getting bool as string
        let bool_as_string = accessor.get_string("bool_key");
        assert_eq!(bool_as_string, None);
    }

    #[test]
    fn test_objc_safe_exec_panic() {
        let result = objc_safe_exec(|| {
            panic!("Test panic");
            #[allow(unreachable_code)]
            Ok::<_, Error>(())
        });
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Panic occurred during Objective-C operation"));
    }

    #[test]
    fn test_objc_safe_exec_error_propagation() {
        let custom_error = Error::system("Custom error message");
        let result: Result<()> = objc_safe_exec(|| Err(custom_error));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Custom error message"));
    }

    #[test]
    fn test_objc_safe_exec_nested() {
        let result = objc_safe_exec(|| objc_safe_exec(|| Ok::<_, Error>(42)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_objc_safe_exec_with_autorelease_pool() {
        let result =
            objc_safe_exec(|| autorelease_pool(|| Ok::<_, Error>("test string".to_string())));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test string");
    }

    #[test]
    fn test_c_str_to_string_non_utf8() {
        let invalid_utf8: Vec<u8> = vec![0xFF, 0xFF, 0x00]; // Invalid UTF-8 sequence
        let c_string = unsafe { CStr::from_bytes_with_nul_unchecked(&invalid_utf8) };
        let ptr = c_string.as_ptr();

        unsafe {
            let result = c_str_to_string(ptr);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_raw_str_to_string_non_utf8() {
        let invalid_utf8: Vec<u8> = vec![0xFF, 0xFF]; // Invalid UTF-8 sequence
        let ptr = invalid_utf8.as_ptr() as *const c_char;
        let len = invalid_utf8.len();

        unsafe {
            let result = raw_str_to_string(ptr, len);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_raw_str_to_string_empty() {
        let empty_str = "";
        let ptr = empty_str.as_ptr() as *const c_char;

        unsafe {
            let result = raw_str_to_string(ptr, 0);
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_raw_str_to_string_with_null() {
        let test_str = "test\0string";
        let ptr = test_str.as_ptr() as *const c_char;
        let len = test_str.len();

        unsafe {
            let result = raw_str_to_string(ptr, len);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "test\0string");
        }
    }

    #[test]
    fn test_raw_f64_slice_to_vec_large() {
        let test_data: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let ptr = test_data.as_ptr();
        let len = test_data.len();

        unsafe {
            let result = raw_f64_slice_to_vec(ptr, len);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), test_data);
        }
    }

    #[test]
    fn test_raw_f64_slice_to_vec_special_values() {
        let test_data: Vec<f64> =
            vec![f64::INFINITY, f64::NEG_INFINITY, f64::NAN, f64::MIN, f64::MAX, 0.0, -0.0];
        let ptr = test_data.as_ptr();
        let len = test_data.len();

        unsafe {
            let result = raw_f64_slice_to_vec(ptr, len);
            assert!(result.is_some());
            let result_vec = result.unwrap();

            assert!(result_vec[0].is_infinite() && result_vec[0].is_sign_positive());
            assert!(result_vec[1].is_infinite() && result_vec[1].is_sign_negative());
            assert!(result_vec[2].is_nan());
            assert_eq!(result_vec[3], f64::MIN);
            assert_eq!(result_vec[4], f64::MAX);
            assert_eq!(result_vec[5], 0.0);
            assert_eq!(result_vec[6], -0.0);
        }
    }
}
