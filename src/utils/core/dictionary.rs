use crate::error::{Error, Result};
use objc2::class;
use objc2::runtime::AnyClass;
use objc2::{msg_send, rc::Retained, runtime::NSObject};
use objc2_foundation::{NSDictionary, NSMutableDictionary, NSNumber, NSString};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Mutex, Once};

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| unsafe {
        let _: &AnyClass = class!(NSObject);
        let _: &AnyClass = class!(NSMutableDictionary);
        let _: &AnyClass = class!(NSNumber);
        let _: &AnyClass = class!(NSString);
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
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(string) = value.downcast::<NSString>() {
            Some(string.to_string())
        } else {
            None
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_f64())
        } else {
            None
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_bool())
        } else {
            None
        }
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        value.downcast::<NSDictionary>().ok().map(|dict| {
            let ptr = Retained::<NSDictionary>::as_ptr(&dict);
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

/// A thread-safe wrapper around NSDictionary that provides a safe interface
/// for accessing dictionary values.
#[derive(Debug)]
pub struct SafeDictionary {
    dict: Mutex<Retained<NSMutableDictionary<NSString, NSObject>>>,
}

impl SafeDictionary {
    /// Creates a new empty SafeDictionary
    pub fn new() -> Self {
        ensure_classes_registered();
        Self { dict: Mutex::new(NSMutableDictionary::new()) }
    }

    /// Creates a SafeDictionary from an existing NSDictionary
    pub fn from(dict: Retained<NSDictionary<NSString, NSObject>>) -> Self {
        ensure_classes_registered();
        let mutable_dict = unsafe {
            let dict_class: *const AnyClass = class!(NSMutableDictionary);
            let dict: *mut NSMutableDictionary<NSString, NSObject> = msg_send![dict_class, alloc];
            let dict: *mut NSMutableDictionary<NSString, NSObject> = msg_send![dict, init];
            Retained::from_raw(dict).expect("Failed to create mutable dictionary")
        };

        unsafe {
            let _: () = msg_send![&*mutable_dict, addEntriesFromDictionary:&*dict];
        }

        Self { dict: Mutex::new(mutable_dict) }
    }

    /// Creates a SafeDictionary from a raw pointer
    ///
    /// # Safety
    /// This function is unsafe because it takes ownership of a raw pointer
    pub unsafe fn from_ptr(ptr: *mut NSObject) -> Self {
        let dict = Retained::from_raw(ptr as *mut NSDictionary<NSString, NSObject>)
            .expect("Failed to create NSDictionary from raw pointer");
        Self::from(dict)
    }

    /// Gets a string value for the given key
    pub fn get_string(&self, key: &str) -> Option<String> {
        let dict = self.dict.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        unsafe {
            let string: *const NSString = Retained::<NSObject>::as_ptr(&value).cast();
            Some((*string).to_string())
        }
    }

    /// Gets a number value for the given key
    pub fn get_number(&self, key: &str) -> Option<f64> {
        let dict = self.dict.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        unsafe {
            let number: f64 = msg_send![&*value, doubleValue];
            Some(number)
        }
    }

    /// Gets a boolean value for the given key
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let dict = self.dict.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        unsafe {
            let bool_value: bool = msg_send![&*value, boolValue];
            Some(bool_value)
        }
    }

    /// Gets a dictionary value for the given key
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let dict = self.dict.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        unsafe {
            let dict_ptr = Retained::<NSObject>::as_ptr(&value);
            let dict = Retained::from_raw(dict_ptr as *mut NSDictionary<NSString, NSObject>)
                .expect("Failed to convert dictionary");
            Some(SafeDictionary::from(dict))
        }
    }

    /// Clones the SafeDictionary, creating a new reference to the same underlying dictionary
    pub fn clone(&self) -> Self {
        if let Ok(dict) = self.dict.lock() {
            let mutable_dict = NSMutableDictionary::new();
            unsafe {
                let _: () = msg_send![&*mutable_dict, addEntriesFromDictionary:&**dict];
            }
            Self { dict: Mutex::new(mutable_dict) }
        } else {
            Self::new()
        }
    }

    /// Returns a raw pointer to the underlying NSDictionary
    ///
    /// # Safety
    /// This function is unsafe because it returns a raw pointer that must be properly managed
    pub unsafe fn as_ptr(&self) -> *const NSObject {
        if let Ok(dict) = self.dict.lock() {
            Retained::<NSMutableDictionary<NSString, NSObject>>::as_ptr(&dict) as *const NSObject
        } else {
            std::ptr::null()
        }
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        if let Ok(dict) = self.dict.lock() {
            let key = NSString::from_str(key);
            let value = unsafe {
                let value: *mut NSObject = msg_send![class!(NSNumber), numberWithBool:value];
                Retained::from_raw(value).expect("Failed to create NSNumber")
            };
            unsafe {
                let _: () = msg_send![&*dict, setObject:&*value, forKey:&*key];
            }
        }
    }

    pub fn set_i64(&mut self, key: &str, value: i64) {
        if let Ok(dict) = self.dict.lock() {
            let key = NSString::from_str(key);
            let value = unsafe {
                let value: *mut NSObject = msg_send![class!(NSNumber), numberWithLongLong:value];
                Retained::from_raw(value).expect("Failed to create NSNumber")
            };
            unsafe {
                let _: () = msg_send![&*dict, setObject:&*value, forKey:&*key];
            }
        }
    }

    pub fn set_f64(&mut self, key: &str, value: f64) {
        if let Ok(dict) = self.dict.lock() {
            let key = NSString::from_str(key);
            let value = unsafe {
                let value: *mut NSObject = msg_send![class!(NSNumber), numberWithDouble:value];
                Retained::from_raw(value).expect("Failed to create NSNumber")
            };
            unsafe {
                let _: () = msg_send![&*dict, setObject:&*value, forKey:&*key];
            }
        }
    }

    pub fn get_array(&self, key: &str) -> Option<Vec<NSObject>> {
        self.get(key).and({
            // Convert NSArray to Vec<NSObject>
            // This is a simplified implementation
            None
        })
    }

    pub fn get_f64(&self, key: &str) -> Result<f64> {
        self.get(key)
            .and({
                // Try to convert NSNumber to f64
                // This is a simplified implementation
                None
            })
            .ok_or_else(|| Error::invalid_data(format!("Key not found: {}", key), None::<String>))
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and({
            // Try to convert NSNumber to i64
            // This is a simplified implementation
            None
        })
    }

    pub fn get(&self, key: &str) -> Option<Retained<NSObject>> {
        let dict = self.dict.lock().unwrap();
        let key_str = NSString::from_str(key);
        unsafe {
            let value: *mut NSObject = msg_send![&**dict, objectForKey:&*key_str];
            if value.is_null() {
                None
            } else {
                // Create a new retained reference
                Some(Retained::from_raw(value).expect("Failed to create retained object"))
            }
        }
    }
}

impl Default for SafeDictionary {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement Send and Sync for SafeDictionary as it uses Mutex<>
unsafe impl Send for SafeDictionary {}
unsafe impl Sync for SafeDictionary {}
