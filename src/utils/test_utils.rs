use crate::hardware::iokit::MockIOKit;
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};

/// Create a mock IOKit instance for testing
pub fn create_mock_iokit() -> MockIOKit {
    MockIOKit::new()
}

/// Create a test dictionary with sample values
pub fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    autoreleasepool(|_| {
        unsafe {
            let dict_class = class!(NSDictionary);
            let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];

            // Ensure we got a valid dictionary
            assert!(!dict_ptr.is_null(), "Failed to create test dictionary");

            // Convert to retained dictionary
            match Retained::from_raw(dict_ptr.cast()) {
                Some(dict) => dict,
                None => panic!("Could not retain dictionary"),
            }
        }
    })
}
