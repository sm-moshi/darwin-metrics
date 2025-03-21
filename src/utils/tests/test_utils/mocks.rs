use std::collections::HashMap;
use std::sync::Mutex;

use objc2::rc::Retained;
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

use crate::utils::core::dictionary::{DictionaryAccess, SafeDictionary};
use crate::utils::core::property::{KeyWrapper, PropertyUtils};

/// A value type that can be stored in a MockDictionary
#[derive(Debug, Clone)]
pub enum MockValue {
    /// A string value stored in the mock dictionary
    String(String),
    /// A numeric value stored in the mock dictionary
    Number(f64),
    /// A boolean value stored in the mock dictionary
    Boolean(bool),
}

impl From<&str> for MockValue {
    fn from(s: &str) -> Self {
        MockValue::String(s.to_string())
    }
}

impl From<String> for MockValue {
    fn from(s: String) -> Self {
        MockValue::String(s)
    }
}

impl From<f64> for MockValue {
    fn from(n: f64) -> Self {
        MockValue::Number(n)
    }
}

impl From<i64> for MockValue {
    fn from(n: i64) -> Self {
        MockValue::Number(n as f64)
    }
}

impl From<bool> for MockValue {
    fn from(b: bool) -> Self {
        MockValue::Boolean(b)
    }
}

/// A pure Rust mock dictionary that can be used for testing without Objective-C
#[derive(Debug)]
pub struct MockDictionary {
    entries: Mutex<HashMap<String, MockValue>>,
}

impl Clone for MockDictionary {
    fn clone(&self) -> Self {
        // Create a new dictionary with cloned entries
        Self {
            entries: Mutex::new(self.entries.lock().unwrap().clone()),
        }
    }
}

impl MockDictionary {
    /// Create a new empty mock dictionary
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Create a mock dictionary with the given entries
    pub fn with_entries(entries: &[(&str, MockValue)]) -> Self {
        let dict = Self::new();
        for (key, value) in entries {
            dict.entries.lock().unwrap().insert(key.to_string(), value.clone());
        }
        dict
    }

    /// Get a string value from the dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.entries.lock().unwrap().get(key) {
            Some(MockValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get a number value from the dictionary
    pub fn get_number(&self, key: &str) -> Option<f64> {
        match self.entries.lock().unwrap().get(key) {
            Some(MockValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    /// Get a boolean value from the dictionary
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.lock().unwrap().get(key) {
            Some(MockValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Insert a value into the dictionary
    pub fn insert<K, V>(&self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<MockValue>,
    {
        self.entries
            .lock()
            .unwrap()
            .insert(key.as_ref().to_string(), value.into());
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key.to_string(), MockValue::Boolean(value));
    }

    pub fn set_i64(&self, key: &str, value: i64) {
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key.to_string(), MockValue::Number(value as f64));
    }

    pub fn set_f64(&self, key: &str, value: f64) {
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key.to_string(), MockValue::Number(value));
    }
}

impl DictionaryAccess for MockDictionary {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_bool(key)
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        None
    }
}

impl PropertyUtils for MockDictionary {
    fn get_string_property(&self, key: &KeyWrapper) -> Option<String> {
        self.get_string(key.as_nsstring().to_string().as_str())
    }

    fn get_number_property(&self, key: &KeyWrapper) -> Option<i64> {
        self.get_number(key.as_nsstring().to_string().as_str())
            .map(|n| n as i64)
    }

    fn set_bool(&mut self, key: &KeyWrapper, value: bool) {
        let key_str = key.as_nsstring().to_string();
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key_str, MockValue::Boolean(value));
    }

    fn set_i64(&mut self, key: &KeyWrapper, value: i64) {
        let key_str = key.as_nsstring().to_string();
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key_str, MockValue::Number(value as f64));
    }

    fn set_f64(&mut self, key: &KeyWrapper, value: f64) {
        let key_str = key.as_nsstring().to_string();
        let mut dict = self.entries.lock().unwrap();
        dict.insert(key_str, MockValue::Number(value));
    }
}

/// Creates a test dictionary with no entries
pub fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    unsafe {
        Retained::from_raw(objc2::msg_send![objc2::class!(NSDictionary), dictionary])
            .expect("Failed to create dictionary")
    }
}

/// Creates a test dictionary with entries
///
/// This function supports creating dictionaries with different value types:
/// - String values: `create_test_dictionary_with_entries(&[("key", "value")])`
/// - Integer values: `create_test_dictionary_with_entries(&[("key", 42)])`
pub fn create_test_dictionary_with_entries<K, V, const N: usize>(
    entries: &[(K, V); N],
) -> Retained<NSDictionary<NSString, NSObject>>
where
    K: AsRef<str>,
    V: ToNSObject,
{
    unsafe {
        // Create arrays for keys and values
        let mut keys: Vec<*mut NSString> = Vec::with_capacity(N);
        let mut values: Vec<*mut NSObject> = Vec::with_capacity(N);

        for (k, v) in entries {
            let ns_string = NSString::from_str(k.as_ref());
            let ns_string_ptr = &ns_string as *const _ as *mut NSString;
            keys.push(ns_string_ptr);
            values.push(v.to_ns_object());
        }

        // Create dictionary with objects and keys
        let dict: *mut NSDictionary<NSString, NSObject> = objc2::msg_send![
            objc2::class!(NSDictionary),
            dictionaryWithObjects: values.as_ptr(),
            forKeys: keys.as_ptr(),
            count: N
        ];

        Retained::from_raw(dict).expect("Failed to create dictionary with entries")
    }
}

/// Creates a test object for testing
pub fn create_test_object() -> Retained<NSObject> {
    unsafe { Retained::from_raw(objc2::msg_send![objc2::class!(NSObject), new]).expect("Failed to create test object") }
}

/// Creates a test string
pub fn create_test_string(content: &str) -> Retained<NSString> {
    NSString::from_str(content)
}

/// Creates a test number
pub fn create_test_number(value: i64) -> Retained<objc2_foundation::NSNumber> {
    unsafe {
        Retained::from_raw(objc2::msg_send![
            objc2::class!(NSNumber),
            numberWithLongLong: value
        ])
        .expect("Failed to create test number")
    }
}

/// Trait for converting Rust types to NSObject
pub trait ToNSObject {
    fn to_ns_object(&self) -> *mut NSObject;
}

// Implement for string literals
impl ToNSObject for &str {
    fn to_ns_object(&self) -> *mut NSObject {
        let ns_string = NSString::from_str(self);
        &ns_string as *const _ as *mut NSObject
    }
}

// Implement for i64 values
impl ToNSObject for i64 {
    fn to_ns_object(&self) -> *mut NSObject {
        unsafe {
            let number: *mut objc2_foundation::NSNumber = objc2::msg_send![
                objc2::class!(NSNumber),
                numberWithLongLong: *self
            ];
            number as *mut NSObject
        }
    }
}

// Implement for f64 values
impl ToNSObject for f64 {
    fn to_ns_object(&self) -> *mut NSObject {
        unsafe {
            let number: *mut objc2_foundation::NSNumber = objc2::msg_send![
                objc2::class!(NSNumber),
                numberWithDouble: *self
            ];
            number as *mut NSObject
        }
    }
}

// Implement for bool values
impl ToNSObject for bool {
    fn to_ns_object(&self) -> *mut NSObject {
        unsafe {
            let number: *mut objc2_foundation::NSNumber = objc2::msg_send![
                objc2::class!(NSNumber),
                numberWithBool: *self
            ];
            number as *mut NSObject
        }
    }
}

// Allow passing references to objects
impl<T: ToNSObject> ToNSObject for &T {
    fn to_ns_object(&self) -> *mut NSObject {
        (*self).to_ns_object()
    }
}

// Allow passing Retained objects
impl<T: objc2::Message> ToNSObject for Retained<T> {
    fn to_ns_object(&self) -> *mut NSObject {
        let ptr = self as *const _ as *mut T;
        ptr as *mut NSObject
    }
}
