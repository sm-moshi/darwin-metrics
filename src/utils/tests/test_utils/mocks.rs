use objc2_foundation::{NSDictionary, NSObject, NSString};
use objc2::rc::Retained;
use std::ffi::c_void;

/// Create a simple test dictionary with string keys and values
pub fn create_string_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    unsafe {
        // Create key and value
        let key = NSString::from_str("key");
        let value = NSString::from_str("value");
        
        // Create a dictionary 
        let dict: *mut NSDictionary<NSString, NSObject> = objc2::msg_send![
            objc2::class!(NSDictionary),
            dictionaryWithObject: &*value,
            forKey: &*key
        ];
        
        // Retain it before returning
        Retained::from_raw(dict).expect("Failed to create string dictionary")
    }
}

/// Create a test dictionary with a test value
pub fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    unsafe {
        // Create a mutable dictionary
        let dict_class = objc2::class!(NSMutableDictionary);
        let dict: *mut objc2_foundation::NSMutableDictionary<NSString, NSObject> = 
            objc2::msg_send![dict_class, dictionary];
        
        // Add a simple key/value pair
        let key = NSString::from_str("TestKey");
        let value = NSString::from_str("TestValue");
        
        let _: () = objc2::msg_send![
            dict,
            setObject: &*value,
            forKey: &*key
        ];
        
        // Cast to immutable dictionary and return
        Retained::from_raw(dict as *mut NSDictionary<NSString, NSObject>)
            .expect("Failed to create test dictionary")
    }
}
