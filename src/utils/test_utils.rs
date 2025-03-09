use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{NSDictionary, NSObject, NSString};

/// Creates a test NSDictionary instance for mock testing.
pub fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    autoreleasepool(|_| unsafe {
        let dict_class = class!(NSDictionary);
        let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];
        Retained::from_raw(dict_ptr.cast()).expect("Failed to create test dictionary")
    })
}

/// Creates a test NSObject instance for mock testing.
pub fn create_test_object() -> Retained<AnyObject> {
    autoreleasepool(|_| unsafe {
        let obj: *mut AnyObject = msg_send![class!(NSObject), new];
        Retained::from_raw(obj).expect("Failed to create test object")
    })
}
