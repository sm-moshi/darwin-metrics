//! Property utilities for interacting with Objective-C property lists.
//!
//! This module provides safe wrappers around NSString and NSMutableDictionary
//! for working with property lists in a type-safe manner. It handles the conversion
//! between Rust types and Objective-C types, ensuring proper memory management and
//! type safety.

use std::sync::{Arc, Mutex};

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2_foundation::{NSCopying, NSMutableDictionary, NSNumber, NSString};

/// Macro to generate property access methods
macro_rules! define_property_accessor {
    ($name:ident, $type:ty, $converter:expr) => {
        fn $name(&self, dict: &NSMutableDictionary<NSString, NSObject>, key: &str) -> Option<$type> {
            let ns_key = NSString::from_str(key);
            unsafe { dict.objectForKey(&ns_key) }
                .and_then(|obj| obj.downcast::<NSNumber>().ok())
                .map($converter)
        };
        ($name:ident,String) => {
            fn $name(&self, dict: &NSMutableDictionary<NSString, NSObject>, key: &str) -> Option<String> {
                let ns_key = NSString::from_str(key);
                unsafe { dict.objectForKey(&ns_key) }
                    .and_then(|obj| obj.downcast::<NSString>().ok())
                    .map(|s| s.to_string())
            }
        };
    }
}

/// Wrapper around NSString that implements NSCopying for use as dictionary keys
#[derive(Debug)]
pub struct KeyWrapper {
    inner: Arc<Retained<NSString>>,
}

impl KeyWrapper {
    /// Creates a new KeyWrapper from a string
    pub fn new(s: &str) -> Self {
        Self {
            inner: Arc::new(NSString::from_str(s)),
        }
    }

    /// Returns a reference to the underlying NSString
    pub fn as_nsstring(&self) -> &NSString {
        &self.inner
    }

    /// Returns a reference to self as an NSCopying protocol object
    /// This is safe because NSString implements NSCopying
    pub fn as_copying(&self) -> &ProtocolObject<dyn NSCopying> {
        unsafe { std::mem::transmute::<&NSString, &ProtocolObject<dyn NSCopying>>(&self.inner) }
    }
}

/// Trait for getting and setting properties in a dictionary
pub trait PropertyUtils {
    /// Gets a string property from the dictionary
    fn get_string_property(&self, key: &KeyWrapper) -> Option<String>;

    /// Gets a number property from the dictionary
    fn get_number_property(&self, key: &KeyWrapper) -> Option<i64>;

    /// Sets a boolean property in the dictionary
    fn set_bool(&mut self, key: &KeyWrapper, value: bool);

    /// Sets an integer property in the dictionary
    fn set_i64(&mut self, key: &KeyWrapper, value: i64);

    /// Sets a float property in the dictionary
    fn set_f64(&mut self, key: &KeyWrapper, value: f64);
}

/// Accessor for properties stored in an NSMutableDictionary
#[derive(Debug)]
pub struct PropertyAccessor {
    /// The underlying dictionary wrapped in Arc<Mutex>
    dict: Arc<Mutex<Retained<NSMutableDictionary>>>,
}

impl Default for PropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAccessor {
    /// Creates a new PropertyAccessor with an empty dictionary
    pub fn new() -> Self {
        Self {
            dict: Arc::new(Mutex::new(NSMutableDictionary::new())),
        }
    }
}

impl PropertyUtils for PropertyAccessor {
    fn get_string_property(&self, key: &KeyWrapper) -> Option<String> {
        let dict = self.dict.lock().unwrap();
        let obj: Option<&NSObject> = unsafe { msg_send![&*dict, objectForKey: key.as_copying()] };
        obj.and_then(|obj| {
            let string: Option<&NSString> = obj.downcast_ref();
            string.map(|s| s.to_string())
        })
    }

    fn get_number_property(&self, key: &KeyWrapper) -> Option<i64> {
        let dict = self.dict.lock().unwrap();
        let obj: Option<&NSObject> = unsafe { msg_send![&*dict, objectForKey: key.as_copying()] };
        obj.and_then(|obj| {
            let num: Option<&NSNumber> = obj.downcast_ref();
            num.map(|n| n.as_i64())
        })
    }

    fn set_bool(&mut self, key: &KeyWrapper, value: bool) {
        let dict = self.dict.lock().unwrap();
        let num = NSNumber::new_bool(value);
        let _: () = unsafe { msg_send![&*dict, setObject: &*num, forKey: key.as_copying()] };
    }

    fn set_i64(&mut self, key: &KeyWrapper, value: i64) {
        let dict = self.dict.lock().unwrap();
        let num = NSNumber::new_i64(value);
        let _: () = unsafe { msg_send![&*dict, setObject: &*num, forKey: key.as_copying()] };
    }

    fn set_f64(&mut self, key: &KeyWrapper, value: f64) {
        let dict = self.dict.lock().unwrap();
        let num = NSNumber::new_f64(value);
        let _: () = unsafe { msg_send![&*dict, setObject: &*num, forKey: key.as_copying()] };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_accessor() {
        let mut accessor = PropertyAccessor::new();

        // Test string property
        let key = KeyWrapper::new("test_string");
        let test_str = "test value";
        let num = NSString::from_str(test_str);
        let _: () = unsafe { msg_send![&*accessor.dict.lock().unwrap(), setObject: &*num, forKey: key.as_copying()] };
        assert_eq!(accessor.get_string_property(&key), Some(test_str.to_string()));

        // Test number property
        let key = KeyWrapper::new("test_number");
        accessor.set_i64(&key, 42);
        assert_eq!(accessor.get_number_property(&key), Some(42));

        // Test bool property
        let key = KeyWrapper::new("test_bool");
        accessor.set_bool(&key, true);
        assert_eq!(accessor.get_number_property(&key), Some(1));
    }
}
