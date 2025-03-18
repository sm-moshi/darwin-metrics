use crate::error::{Error, Result};
use objc2::{class, msg_send};
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSMutableDictionary, NSDictionary, NSNumber, NSObject, NSString};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Mutex, Once, Arc};

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

/// Thread-safe wrapper around NSMutableDictionary for safe use of Core Foundation dictionaries across
/// threads
#[derive(Debug)]
pub struct SafeDictionary {
    dict: Mutex<Retained<NSMutableDictionary<NSString, NSObject>>>,
}

impl SafeDictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        ensure_classes_registered();
        Self { dict: Mutex::new(NSMutableDictionary::new()) }
    }

    /// Create from an existing NSDictionary
    pub fn from(dict: Retained<NSDictionary<NSString, NSObject>>) -> Self {
        ensure_classes_registered();
        
        // Convert to mutable if needed
        let mutable_dict = if dict.count() > 0 {
            let dict_class = class!(NSMutableDictionary);
            
            // Use from_raw instead of new and handle the Option return
            let dict_result: Option<Retained<NSMutableDictionary<NSString, NSObject>>> =
                unsafe { 
                    msg_send![dict_class, dictionaryWithCapacity: dict.count()]
                };
            
            match dict_result {
                Some(mutable_dict) => {
                    unsafe {
                        let _: () = msg_send![&*mutable_dict, addEntriesFromDictionary:&*dict];
                    }
                    mutable_dict
                },
                None => NSMutableDictionary::new()
            }
        } else {
            NSMutableDictionary::new()
        };
        
        Self { dict: Mutex::new(mutable_dict) }
    }

    /// Creates a SafeDictionary from a raw pointer
    /// 
    /// # Safety
    /// This function is unsafe because it takes ownership of a raw pointer
    pub unsafe fn from_ptr(ptr: *mut AnyObject) -> Self {
        ensure_classes_registered();
        
        // First check if the pointer is valid
        if ptr.is_null() {
            return Self::new();
        }
        
        // Try to convert the pointer to a dictionary
        // from_raw returns an Option, not a Result
        match Retained::from_raw(ptr as *mut NSDictionary<NSString, NSObject>) {
            Some(dict) => Self::from(dict),
            None => {
                // If conversion fails, return an empty dictionary
                // but log the error for debugging
                eprintln!("Warning: Failed to convert pointer to NSDictionary");
                Self::new()
            }
        }
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

    /// Get the raw pointer to the underlying dictionary
    /// 
    /// # Safety
    /// This returns a raw pointer that should be used with care
    pub unsafe fn as_ptr(&self) -> *const NSObject {
        let dict_lock = self.dict.lock().expect("Failed to lock dictionary mutex");
        let obj_ptr = &*dict_lock as &NSMutableDictionary<NSString, NSObject> as *const NSMutableDictionary<NSString, NSObject>;
        obj_ptr as *const NSObject
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
