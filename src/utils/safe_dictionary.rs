use objc2::{msg_send, rc::Retained, runtime::NSObject, Message};
use objc2_foundation::{NSDictionary, NSNumber, NSString};
use std::fmt::Debug;
use std::sync::Mutex;

/// A thread-safe wrapper around NSDictionary that provides a safe interface
/// for accessing dictionary values.
#[derive(Debug)]
pub struct SafeDictionary {
    inner: Mutex<Retained<NSDictionary<NSString, NSObject>>>,
}

impl SafeDictionary {
    /// Creates a new empty SafeDictionary
    pub fn new() -> Self {
        Self { inner: Mutex::new(NSDictionary::new()) }
    }

    /// Creates a SafeDictionary from an existing NSDictionary
    pub fn from(dict: Retained<NSDictionary<NSString, NSObject>>) -> Self {
        Self { inner: Mutex::new(dict) }
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
        let dict = self.inner.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        if let Ok(string) = value.downcast::<NSString>() {
            Some(string.to_string())
        } else {
            None
        }
    }

    /// Gets a number value for the given key
    pub fn get_number(&self, key: &str) -> Option<f64> {
        let dict = self.inner.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_f64())
        } else {
            None
        }
    }

    /// Gets a boolean value for the given key
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let dict = self.inner.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_bool())
        } else {
            None
        }
    }

    /// Gets a dictionary value for the given key
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let dict = self.inner.lock().ok()?;
        let key = NSString::from_str(key);
        let value = unsafe { dict.valueForKey(&key) }?;
        value.downcast::<NSDictionary>().ok().map(|dict| {
            let ptr = Retained::<NSDictionary>::as_ptr(&dict);
            let typed_dict = unsafe { Retained::from_raw(ptr as *mut NSDictionary<NSString, NSObject>) };
            SafeDictionary::from(typed_dict.expect("Failed to convert dictionary"))
        })
    }

    /// Clones the SafeDictionary, creating a new reference to the same underlying dictionary
    pub fn clone(&self) -> Self {
        if let Ok(dict) = self.inner.lock() {
            Self { inner: Mutex::new(dict.retain()) }
        } else {
            Self::new()
        }
    }

    /// Gets the raw pointer to the underlying NSDictionary
    ///
    /// # Safety
    /// This function is unsafe because it returns a raw pointer that must be properly managed
    pub unsafe fn as_ptr(&self) -> *const NSObject {
        if let Ok(dict) = self.inner.lock() {
            Retained::<NSDictionary<NSString, NSObject>>::as_ptr(&dict) as *const NSObject
        } else {
            std::ptr::null()
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut NSObject {
        if let Ok(dict) = self.inner.lock() {
            Retained::<NSDictionary<NSString, NSObject>>::as_ptr(&dict) as *mut NSObject
        } else {
            std::ptr::null_mut()
        }
    }

    /// Sets a boolean value for the given key
    pub fn set_bool(&mut self, key: &str, value: bool) {
        if let Ok(dict) = self.inner.lock() {
            let key = NSString::from_str(key);
            let value = NSNumber::new_bool(value);
            unsafe {
                let _: () = msg_send![&*dict, setValue: &*value, forKey: &*key];
            }
        }
    }

    /// Sets an integer value for the given key
    pub fn set_i64(&mut self, key: &str, value: i64) {
        if let Ok(dict) = self.inner.lock() {
            let key = NSString::from_str(key);
            let value = NSNumber::new_i64(value);
            unsafe {
                let _: () = msg_send![&*dict, setValue: &*value, forKey: &*key];
            }
        }
    }

    /// Sets a floating point value for the given key
    pub fn set_f64(&mut self, key: &str, value: f64) {
        if let Ok(dict) = self.inner.lock() {
            let key = NSString::from_str(key);
            let value = NSNumber::new_f64(value);
            unsafe {
                let _: () = msg_send![&*dict, setValue: &*value, forKey: &*key];
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
