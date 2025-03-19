use std::sync::{Arc, Mutex};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::rc::Retained;
use objc2_foundation::{NSCopying, NSMutableDictionary, NSNumber, NSString};
use objc2::{msg_send, sel};

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

/// A thread-safe wrapper for NSString keys that implements proper NSCopying handling
pub struct KeyWrapper {
    inner: Arc<Retained<NSString>>
}

impl KeyWrapper {
    /// Creates a new KeyWrapper from a string slice
    pub fn new(key: &str) -> Self {
        Self {
            inner: Arc::new(NSString::from_str(key).to_owned())
        }
    }
    
    /// Returns a reference to the underlying NSString
    pub fn as_nsstring(&self) -> &NSString {
        &self.inner
    }

    /// Returns a reference to the NSString as a ProtocolObject<dyn NSCopying>
    /// This is useful when passing to methods that expect a NSCopying implementor
    pub fn as_copying(&self) -> &ProtocolObject<dyn NSCopying> {
        // Safety: NSString implements NSCopying, so this cast is safe
        unsafe { std::mem::transmute::<&NSString, &ProtocolObject<dyn NSCopying>>(&self.inner) }
    }
}

/// Trait for accessing properties in a dictionary
pub trait PropertyUtils {
    /// Get a string property from the dictionary
    fn get_string_property(&self, key: &str) -> Option<String>;
    
    /// Get a number property from the dictionary and convert it using the provided function
    fn get_number_property<T>(&self, key: &str, converter: impl Fn(&NSNumber) -> T) -> Option<T>;
    
    /// Get a boolean property from the dictionary
    fn get_bool_property(&self, key: &str) -> Option<bool>;
    
    /// Set a boolean property in the dictionary
    fn set_bool(&self, key: &str, value: bool);
    
    /// Set an i64 property in the dictionary
    fn set_i64(&self, key: &str, value: i64);
    
    /// Set an f64 property in the dictionary
    fn set_f64(&self, key: &str, value: f64);
}

/// A property accessor for interacting with property lists.
pub struct PropertyAccessor {
    dict: Arc<Mutex<NSMutableDictionary<NSString, NSObject>>>,
}

impl PropertyAccessor {
    /// Creates a new PropertyAccessor with the given dictionary
    pub fn new(dict: NSMutableDictionary<NSString, NSObject>) -> Self {
        Self {
            dict: Arc::new(Mutex::new(dict)),
        }
    }
}

impl PropertyUtils for PropertyAccessor {
    fn get_string_property(&self, key: &str) -> Option<String> {
        let key = KeyWrapper::new(key);
        let dict = self.dict.lock().unwrap();
        
        let obj = dict.objectForKey(key.as_nsstring())?;
        match obj.downcast::<NSString>() {
            Ok(string) => Some(string.to_string()),
            Err(_) => None
        }
    }
    
    fn get_number_property<T>(&self, key: &str, converter: impl Fn(&NSNumber) -> T) -> Option<T> {
        let key = KeyWrapper::new(key);
        let dict = self.dict.lock().unwrap();
        
        let obj = dict.objectForKey(key.as_nsstring())?;
        match obj.downcast::<NSNumber>() {
            Ok(number) => Some(converter(&number)),
            Err(_) => None
        }
    }
    
    fn get_bool_property(&self, key: &str) -> Option<bool> {
        self.get_number_property(key, |num| num.as_bool())
    }
    
    fn set_bool(&self, key: &str, value: bool) {
        let key = KeyWrapper::new(key);
        let value = NSNumber::new_bool(value);
        let dict = self.dict.lock().unwrap();
        
        unsafe {
            dict.setObject_forKey(&value, key.as_copying());
        }
    }
    
    fn set_i64(&self, key: &str, value: i64) {
        let key = KeyWrapper::new(key);
        let value = NSNumber::new_i64(value);
        let dict = self.dict.lock().unwrap();
        
        unsafe {
            dict.setObject_forKey(&value, key.as_copying());
        }
    }
    
    fn set_f64(&self, key: &str, value: f64) {
        let key = KeyWrapper::new(key);
        let value = NSNumber::new_f64(value);
        let dict = self.dict.lock().unwrap();
        
        unsafe {
            dict.setObject_forKey(&value, key.as_copying());
        }
    }
}
