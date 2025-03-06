use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{NSDictionary, NSObject, NSString};

/// Create an empty dictionary for safe testing
pub fn create_empty_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    autoreleasepool(|_| {
        unsafe {
            let dict_class = class!(NSDictionary);
            let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];

            // Ensure we got a valid dictionary
            assert!(!dict_ptr.is_null(), "Failed to create empty dictionary");

            // Convert to retained dictionary
            match Retained::from_raw(dict_ptr.cast()) {
                Some(dict) => dict,
                None => panic!("Could not retain dictionary"),
            }
        }
    })
}

/// Create a test AnyObject safely
pub fn create_test_anyobject() -> Retained<AnyObject> {
    autoreleasepool(|_| {
        unsafe {
            let obj_class = class!(NSObject);
            let obj_ptr: *mut AnyObject = msg_send![obj_class, new];

            // Ensure we got a valid object
            assert!(!obj_ptr.is_null(), "Failed to create test object");

            // Convert to retained object
            match Retained::from_raw(obj_ptr.cast()) {
                Some(obj) => obj,
                None => panic!("Could not retain object"),
            }
        }
    })
}
