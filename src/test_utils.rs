use objc2::msg_send;
use objc2::class;
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};

use crate::hardware::iokit;

pub fn create_mock_iokit() -> iokit::MockIOKit {
    let mut mock = iokit::MockIOKit::new();
    mock.expect_io_service_matching()
        .returning(|_| create_test_dictionary());
    
    mock.expect_io_service_get_matching_service()
        .returning(|_| Some(create_test_object()));
    
    mock
}

pub fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
    autoreleasepool(|_| {
        unsafe {
            let dict_class = class!(NSDictionary);
            let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];
            Retained::from_raw(dict_ptr.cast()).expect("Failed to create test dictionary")
        }
    })
}

fn create_test_object() -> Retained<AnyObject> {
    autoreleasepool(|_| unsafe {
        let obj: *mut AnyObject = msg_send![class!(NSObject), new];
        Retained::from_raw(obj).expect("Failed to create test object")
    })
}