use std::fmt::Debug;
use std::ops::Deref;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex, Once};

use objc2::encode::{Encode, RefEncode};
use objc2::rc::Retained;
use objc2::runtime::{MessageReceiver, NSObject, AnyObject};
use objc2::{Message, class, msg_send};
use objc2_foundation::{NSArray, NSDictionary, NSMutableDictionary, NSNumber, NSString};

use crate::error::{Error, Result};
use crate::utils::core::property::{KeyWrapper, PropertyUtils};

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| {
        // Just load the classes without trying to cast
        let _ = class!(NSObject);
        let _ = class!(NSMutableDictionary);
        let _ = class!(NSNumber);
    });
}

/// Trait for accessing dictionary values in a type-safe way
pub trait DictionaryAccess {
    /// Get a string value for the given key
    fn get_string(&self, key: &str) -> Option<String>;

    /// Get a number value for the given key
    fn get_number(&self, key: &str) -> Option<f64>;

    /// Get a boolean value for the given key
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Get a dictionary value for the given key
    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary>;
}

/// Define the MockDictionary struct
pub struct DictionaryMock;

/// Implementation for MockDictionary
impl DictionaryAccess for DictionaryMock {
    fn get_string(&self, _key: &str) -> Option<String> {
        None
    }

    fn get_number(&self, _key: &str) -> Option<f64> {
        None
    }

    fn get_bool(&self, _key: &str) -> Option<bool> {
        None
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        None
    }
}

/// Implementation for NSDictionary using PropertyUtils
impl DictionaryAccess for NSDictionary<NSString, NSObject> {
    fn get_string(&self, key: &str) -> Option<String> {
        let key = NSString::from_str(key);
        // SAFETY: valueForKey is an Objective-C method that requires unsafe due to FFI.
        // The key is guaranteed to be valid as it's created from a str.
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(string) = value.downcast::<NSString>() {
            Some(string.to_string())
        } else {
            None
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        let key = NSString::from_str(key);
        // SAFETY: valueForKey is an Objective-C method that requires unsafe due to FFI.
        // The key is guaranteed to be valid as it's created from a str.
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_f64())
        } else {
            None
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        let key = NSString::from_str(key);
        // SAFETY: valueForKey is an Objective-C method that requires unsafe due to FFI.
        // The key is guaranteed to be valid as it's created from a str.
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_bool())
        } else {
            None
        }
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let key = NSString::from_str(key);
        // SAFETY: valueForKey is an Objective-C method that requires unsafe due to FFI.
        // The key is guaranteed to be valid as it's created from a str.
        let value = unsafe { self.valueForKey(&key) }?;
        value.downcast::<NSDictionary>().ok().map(|dict| {
            let ptr = Retained::<NSDictionary>::as_ptr(&dict);
            // SAFETY: Converting between NSDictionary types is safe as they have the same underlying representation.
            // The pointer is valid as it comes from a Retained<NSDictionary>.
            let typed_dict = unsafe { Retained::from_raw(ptr as *mut NSDictionary<NSString, NSObject>) };
            SafeDictionary::from(typed_dict.expect("Failed to convert dictionary"))
        })
    }
}

/// Implementation for Retained NSDictionary
impl DictionaryAccess for Retained<NSDictionary<NSString, NSObject>> {
    fn get_string(&self, key: &str) -> Option<String> {
        self.deref().get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.deref().get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.deref().get_bool(key)
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        self.deref().get_dictionary(key)
    }
}

/// A struct that can work with any dictionary implementation
pub struct DictionaryAccessor<T: DictionaryAccess> {
    dictionary: T,
}

impl<T: DictionaryAccess> DictionaryAccessor<T> {
    /// Create a new DictionaryAccessor with the given dictionary
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Get a string value from the dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.dictionary.get_string(key)
    }

    /// Get a number value from the dictionary
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.dictionary.get_number(key)
    }

    /// Get a boolean value from the dictionary
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.dictionary.get_bool(key)
    }

    /// Get a dictionary value from the dictionary
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        self.dictionary.get_dictionary(key)
    }
}

/// A thread-safe wrapper around NSMutableDictionary
#[derive(Debug)]
pub struct SafeDictionary {
    inner: Arc<Mutex<Retained<NSMutableDictionary<NSString, NSObject>>>>,
}

// SAFETY: These trait implementations are safe because:
// 1. SafeDictionary contains only Objective-C objects that implement these traits
// 2. The mutex ensures thread-safety
// 3. All access to the underlying dictionary is protected by the mutex
unsafe impl Encode for SafeDictionary {
    const ENCODING: objc2::encode::Encoding = <&NSObject>::ENCODING;
}

unsafe impl RefEncode for SafeDictionary {
    const ENCODING_REF: objc2::encode::Encoding = <&NSObject>::ENCODING;
}

unsafe impl Message for SafeDictionary {}

impl SafeDictionary {
    /// Create a new empty SafeDictionary
    pub fn new() -> Self {
        let dict = NSMutableDictionary::new();
        Self {
            inner: Arc::new(Mutex::new(dict)),
        }
    }

    /// Create a SafeDictionary from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid NSMutableDictionary pointer
    pub fn from_ptr(ptr: *mut NSMutableDictionary<NSString, NSObject>) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::null_pointer("Dictionary pointer is null"));
        }

        let dict = unsafe { Retained::from_raw(ptr) };
        Ok(Self {
            inner: Arc::new(Mutex::new(dict)),
        })
    }

    /// Get a string value from the dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let obj = unsafe { msg_send![dict, objectForKey:&*key] };
        if obj.is_null() {
            return None;
        }
        let obj = unsafe { &*(obj as *const NSString) };
        Some(obj.to_string())
    }

    /// Get a number value from the dictionary
    pub fn get_number(&self, key: &str) -> Option<f64> {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let obj = unsafe { msg_send![dict, objectForKey:&*key] };
        if obj.is_null() {
            return None;
        }
        let number = unsafe { &*(obj as *const NSNumber) };
        Some(number.as_f64())
    }

    /// Get a boolean value from the dictionary
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let obj = unsafe { msg_send![dict, objectForKey:&*key] };
        if obj.is_null() {
            return None;
        }
        let number = unsafe { &*(obj as *const NSNumber) };
        Some(number.as_bool())
    }

    /// Get a dictionary value from the dictionary
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let obj = unsafe { msg_send![dict, objectForKey:&*key] };
        if obj.is_null() {
            return None;
        }
        let dict_ptr = obj as *mut NSMutableDictionary<NSString, NSObject>;
        unsafe { SafeDictionary::from_ptr(dict_ptr).ok() }
    }

    /// Set a string value in the dictionary
    pub fn set_string(&self, key: &str, value: &str) {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let value = NSString::from_str(value);
        unsafe {
            let _: () = msg_send![dict, setObject: &*value, forKey: &*key];
        }
    }

    /// Set a number value in the dictionary
    pub fn set_number(&self, key: &str, value: f64) {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let value = NSNumber::from_f64(value);
        unsafe {
            let _: () = msg_send![dict, setObject: &*value, forKey: &*key];
        }
    }

    /// Set a boolean value in the dictionary
    pub fn set_bool(&self, key: &str, value: bool) {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let value = NSNumber::from_bool(value);
        unsafe {
            let _: () = msg_send![dict, setObject: &*value, forKey: &*key];
        }
    }

    /// Set an integer value in the dictionary
    pub fn set_i64(&self, key: &str, value: i64) {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let value = NSNumber::from_i64(value);
        unsafe {
            let _: () = msg_send![dict, setObject: &*value, forKey: &*key];
        }
    }

    /// Set a floating point value in the dictionary
    pub fn set_f64(&self, key: &str, value: f64) {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let value = NSNumber::from_f64(value);
        unsafe {
            let _: () = msg_send![dict, setObject: &*value, forKey: &*key];
        }
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let key = NSString::from_str(key);
        let obj = unsafe { msg_send![dict, objectForKey:&*key] };
        if obj.is_null() {
            return None;
        }
        let number = unsafe { &*(obj as *const NSNumber) };
        Some(number.as_i64())
    }
}

impl Clone for SafeDictionary {
    fn clone(&self) -> Self {
        let guard = self.inner.lock().expect("Failed to lock dictionary");
        let dict = guard.as_ref();
        let new_dict = unsafe {
            let ptr = msg_send![dict, mutableCopy];
            Retained::from_raw(ptr)
        };
        Self {
            inner: Arc::new(Mutex::new(new_dict)),
        }
    }
}

/// Implement Send and Sync for SafeDictionary as it uses Arc<Mutex<>>
/// SAFETY: These implementations are safe because:
/// 1. All access to the underlying NSMutableDictionary is protected by a mutex
/// 2. The Retained type ensures proper memory management
/// 3. NSMutableDictionary is thread-safe when accessed through the mutex
unsafe impl Send for SafeDictionary {}
unsafe impl Sync for SafeDictionary {}

/// Implementation of DictionaryAccess for SafeDictionary
impl DictionaryAccess for SafeDictionary {
    fn get_string(&self, key: &str) -> Option<String> {
        let dict = self.inner.lock().expect("Failed to lock dictionary");
        let key = NSString::from_str(key);
        unsafe {
            let value = dict.object_for(&key)?;
            if let Ok(string) = value.downcast::<NSString>() {
                return Some(string.to_string());
            }
        }
        None
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        let dict = self.inner.lock().expect("Failed to lock dictionary");
        let key = NSString::from_str(key);
        unsafe {
            let value = dict.object_for(&key)?;
            if let Ok(number) = value.downcast::<NSNumber>() {
                return Some(number.as_f64());
            }
        }
        None
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        let dict = self.inner.lock().expect("Failed to lock dictionary");
        let key = NSString::from_str(key);
        unsafe {
            let value = dict.object_for(&key)?;
            if let Ok(number) = value.downcast::<NSNumber>() {
                return Some(number.as_bool());
            }
        }
        None
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let dict = self.inner.lock().expect("Failed to lock dictionary");
        let key = NSString::from_str(key);
        unsafe {
            let value = dict.object_for(&key)?;
            let dict: Option<&NSMutableDictionary<NSString, NSObject>> = (*value).downcast_ref();
            dict.map(|d| Self {
                inner: Arc::new(Mutex::new(Retained::retain(d).expect("Failed to retain dictionary"))),
            })
        }
    }
}

impl PropertyUtils for SafeDictionary {
    fn get_string_property(&self, key: &KeyWrapper) -> Option<String> {
        let key_str = key.as_nsstring().to_string();
        self.get_string(&key_str)
    }

    fn get_number_property(&self, key: &KeyWrapper) -> Option<i64> {
        let key_str = key.as_nsstring().to_string();
        self.get_i64(&key_str)
    }

    fn set_bool(&mut self, key: &KeyWrapper, value: bool) {
        let key_str = key.as_nsstring().to_string();
        let key = NSString::from_str(&key_str);
        let value_obj = NSNumber::new_bool(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }

    fn set_i64(&mut self, key: &KeyWrapper, value: i64) {
        let key_str = key.as_nsstring().to_string();
        let key = NSString::from_str(&key_str);
        let value_obj = NSNumber::new_i64(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }

    fn set_f64(&mut self, key: &KeyWrapper, value: f64) {
        let key_str = key.as_nsstring().to_string();
        let key = NSString::from_str(&key_str);
        let value_obj = NSNumber::new_f64(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }
}

// Extension methods for SafeDictionary to work with string keys
impl SafeDictionary {
    pub fn set_bool_str(&mut self, key: &str, value: bool) {
        let key = NSString::from_str(key);
        let value_obj = NSNumber::new_bool(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }

    pub fn set_i64_str(&mut self, key: &str, value: i64) {
        let key = NSString::from_str(key);
        let value_obj = NSNumber::new_i64(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }

    pub fn set_f64_str(&mut self, key: &str, value: f64) {
        let key = NSString::from_str(key);
        let value_obj = NSNumber::new_f64(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }

    pub fn set_string_str(&mut self, key: &str, value: &str) {
        let key = NSString::from_str(key);
        let value_obj = NSString::from_str(value);

        let guard = &mut self.inner.lock().expect("Failed to lock dictionary");
        unsafe {
            let dict = guard.as_ref();
            let _: () = msg_send![dict, setObject: &*value_obj, forKey: &*key];
        }
    }
}

#[cfg(test)]
mod tests {
    use scopeguard::defer;

    use super::*;
    use crate::utils::core::property::{KeyWrapper, PropertyUtils};

    #[test]
    fn test_property_utils() {
        // Create an autorelease pool for the test
        unsafe {
            let pool: *mut NSObject = msg_send![class!(NSAutoreleasePool), new];
            defer! {
                let _: () = msg_send![pool, drain];
            }

            let mut dict = SafeDictionary::new();
            let key = KeyWrapper::new("test_key");

            // Test string property
            dict.set_string("test_key", "test_value");
            assert_eq!(dict.get_string_property(&key), Some("test_value".to_string()));

            // Test number property
            dict.set_i64("test_key", 42);
            assert_eq!(dict.get_number_property(&key), Some(42));

            // Test bool property
            dict.set_bool("test_key", true);
            assert_eq!(dict.get_bool("test_key"), Some(true));

            // Test f64 property
            dict.set_f64("test_key", 3.14);
            assert_eq!(dict.get_number("test_key"), Some(3.14));
        }
    }
}
