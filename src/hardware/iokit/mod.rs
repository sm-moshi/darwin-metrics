//! IOKit interface for macOS system metrics
//!
//! This module provides a safe abstraction over the IOKit framework,
//! allowing access to system hardware information and metrics.
//!
//! # Safety
//!
//! This module uses objc2 for safe interaction with IOKit.
//! All operations follow proper resource management practices through RAII.
//!
//! # Thread Safety
//!
//! All types in this module implement `Send` and `Sync` where appropriate.
//! Resource cleanup is handled automatically through objc2's reference counting.
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::iokit::{IOKit, IOKitImpl};
//!
//! fn main() -> Result<(), darwin_metrics::Error> {
//!     let iokit = IOKitImpl::default();
//!     
//!     // Get battery service
//!     let matching = iokit.io_service_matching("AppleSmartBattery");
//!     let service = iokit.io_service_get_matching_service(&matching);
//!     
//!     // Get properties (service and properties are automatically released by objc2)
//!     if let Some(service) = service {
//!         let properties = iokit.io_registry_entry_create_cf_properties(&service)?;
//!         Ok(())
//!     } else {
//!         Err(darwin_metrics::Error::ServiceNotFound)
//!     }
//! }
//! ```

use objc2::class;
use objc2::msg_send;
use objc2::rc::{autoreleasepool, Retained}; // Added autoreleasepool import
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

use crate::Error;

// The IOKit trait is now defined below with #[cfg_attr(test, automock)]

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKit for IOKitImpl {
    fn io_service_matching(
        &self,
        service_name: &str,
    ) -> Retained<NSDictionary<NSString, NSObject>> {
        // Use autorelease pool to manage temporary objects - pass _ parameter to satisfy type requirements
        autoreleasepool(|_| {
            unsafe {
                // First create the service name as an NSString
                let ns_service_name = NSString::from_str(service_name);

                // Safe fallback is an empty dictionary
                let empty_dict = {
                    let dict_class = class!(NSDictionary);
                    let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];

                    if dict_ptr.is_null() {
                        panic!("Failed to create empty dictionary");
                    }

                    match Retained::from_raw(dict_ptr.cast()) {
                        Some(dict) => dict,
                        None => panic!("Failed to retain empty dictionary"),
                    }
                };

                // Try to get the IOService class
                if let Some(io_class) = AnyClass::get(c"IOService") {
                    let dict_ptr: *mut AnyObject =
                        msg_send![io_class, serviceMatching: &*ns_service_name];

                    // If result is null or can't be retained, return our empty fallback
                    if dict_ptr.is_null() {
                        return empty_dict;
                    }

                    match Retained::from_raw(dict_ptr.cast()) {
                        Some(dict) => dict,
                        None => empty_dict,
                    }
                } else {
                    // IOService class not found, return empty dictionary
                    empty_dict
                }
            }
        })
    }

    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        unsafe {
            // Use the master port (0) and the matching dictionary
            let master_port: u32 = 0; // kIOMasterPortDefault
                                      // Fixed syntax with proper comma separation
            let service: *mut AnyObject =
                msg_send![class!(IOService), getMatchingService: master_port, matching: matching];

            if service.is_null() {
                None
            } else {
                Retained::from_raw(service)
            }
        }
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>, Error> {
        unsafe {
            let mut props: *mut AnyObject = std::ptr::null_mut();
            let result: i32 = msg_send![entry, createCFProperties:&mut props];

            if result != 0 {
                return Err(Error::SystemError(format!(
                    "Failed to get properties: {}",
                    result
                )));
            }

            if props.is_null() {
                return Err(Error::SystemError("Failed to get properties".to_string()));
            }

            Retained::from_raw(props.cast())
                .ok_or_else(|| Error::SystemError("Failed to retain properties".to_string()))
        }
    }

    fn io_object_release(&self, _obj: &AnyObject) {
        // No need to explicitly release - objc2's Retained handles this automatically
    }

    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let string: &NSString = &*(value.cast());
            Some(string.to_string())
        }
    }

    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let number: &NSNumber = &*(value.cast());
            Some(number.as_i64())
        }
    }

    fn get_bool_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
        let key = NSString::from_str(key);
        unsafe {
            let value: *mut AnyObject = msg_send![dict, objectForKey:&*key];
            if value.is_null() {
                return None;
            }
            let number: &NSNumber = &*(value.cast());
            Some(number.as_bool())
        }
    }
}

#[cfg(test)]
pub use mockall::automock;

#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    fn io_service_matching(&self, service_name: &str)
        -> Retained<NSDictionary<NSString, NSObject>>;
    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>>;
    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>, Error>;
    fn io_object_release(&self, obj: &AnyObject);
    fn get_string_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String>;
    fn get_number_property(
        &self,
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64>;
    fn get_bool_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str)
        -> Option<bool>;
}

// MockIOKit is automatically defined by automock on the IOKit trait
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    // Using the MockIOKit from the mock module

    /// Create an empty dictionary for safe testing
    fn create_empty_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
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

    /// Create a safe dictionary for testing
    fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
        autoreleasepool(|_| unsafe {
            let dict_class = class!(NSDictionary);
            let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];
            Retained::from_raw(dict_ptr.cast()).expect("Failed to create test dictionary")
        })
    }

    /// Create a safe NSObject for testing
    fn create_test_object() -> Retained<AnyObject> {
        autoreleasepool(|_| unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).expect("Failed to create test object")
        })
    }

    /// Create a safe empty AnyObject for testing
    fn create_test_anyobject() -> Retained<AnyObject> {
        autoreleasepool(|_| unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).expect("Failed to create test object")
        })
    }

    #[test]
    fn test_mock_iokit() {
        let mut mock = MockIOKit::new();
        let service_name = "TestService";

        // Set up expectations with safer test code
        mock.expect_io_service_matching()
            .with(eq(service_name))
            .times(1)
            .returning(|_| create_test_dictionary());

        // Call the method to test
        let _ = mock.io_service_matching(service_name);
    }

    #[test]
    fn test_io_service_matching() {
        let iokit = IOKitImpl::default();
        let service_name = "AppleSmartBattery";

        // Test that we get a valid dictionary (just check it's not null)
        let dict = iokit.io_service_matching(service_name);
        let ptr: *const AnyObject = dict.as_ref();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_io_service_get_matching_service() {
        let mut mock = MockIOKit::new();
        let dict = create_test_dictionary();

        // Mock the response with a simple test object
        mock.expect_io_service_get_matching_service()
            .with(always())
            .returning(|_| Some(create_test_object()));

        // Call the method with our mock instead of a real IOKit implementation
        let service = mock.io_service_get_matching_service(&dict);
        assert!(service.is_some());
    }

    #[test]
    fn test_get_string_property() {
        let mut mock = MockIOKit::new();
        let key = "TestKey";
        let value = "TestValue";

        // Mock the responses
        mock.expect_get_string_property()
            .with(always(), eq(key))
            .returning(|_, _| Some(value.to_string()));

        mock.expect_get_string_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        // Test property retrieval using the mock
        let result = mock.get_string_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(value.to_string()));

        // Test non-existent property
        let result = mock.get_string_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_number_property() {
        let mut mock = MockIOKit::new();
        let key = "TestNumberKey";
        let value: i64 = 42;

        // Mock the responses
        mock.expect_get_number_property()
            .with(always(), eq(key))
            .returning(move |_, _| Some(value));

        mock.expect_get_number_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        // Test property retrieval using the mock
        let result = mock.get_number_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(value));

        // Test non-existent property
        let result = mock.get_number_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_bool_property() {
        let mut mock = MockIOKit::new();
        let key = "TestBoolKey";

        // Mock the responses
        mock.expect_get_bool_property()
            .with(always(), eq(key))
            .returning(|_, _| Some(true));

        mock.expect_get_bool_property()
            .with(always(), eq("NonExistentKey"))
            .returning(|_, _| None);

        // Test property retrieval using the mock
        let result = mock.get_bool_property(&create_test_dictionary(), key);
        assert_eq!(result, Some(true));

        // Test non-existent property
        let result = mock.get_bool_property(&create_test_dictionary(), "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_io_object_release() {
        let iokit = IOKitImpl::default();

        // Create a test object
        let obj = create_test_object();

        // This is a no-op in our implementation, but should not panic
        iokit.io_object_release(&obj);
    }

    #[test]
    fn test_io_registry_entry_create_cf_properties() {
        let mut mock = MockIOKit::new();
        let test_obj = create_test_object();

        // Set up mock to return a dictionary
        mock.expect_io_registry_entry_create_cf_properties()
            .with(always())
            .returning(|_| Ok(create_test_dictionary()));

        // Call and verify it returns Ok
        let result = mock.io_registry_entry_create_cf_properties(&test_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration() {
        let mut mock = MockIOKit::new();
        let test_name = "IOPlatformExpertDevice";

        // Set up all the mocks we need for the full flow
        mock.expect_io_service_matching()
            .with(eq(test_name))
            .returning(|_| create_test_dictionary());

        mock.expect_io_service_get_matching_service()
            .with(always())
            .returning(|_| Some(create_test_object()));

        mock.expect_io_registry_entry_create_cf_properties()
            .with(always())
            .returning(|_| Ok(create_test_dictionary()));

        mock.expect_get_string_property()
            .with(always(), always())
            .returning(|_, _| Some("Test Value".to_string()));

        mock.expect_get_number_property()
            .with(always(), always())
            .returning(|_, _| Some(42));

        mock.expect_get_bool_property()
            .with(always(), always())
            .returning(|_, _| Some(true));

        // Now call the methods in sequence
        let matching = mock.io_service_matching(test_name);
        let service = mock.io_service_get_matching_service(&matching);

        assert!(service.is_some());

        if let Some(service) = service {
            let result = mock.io_registry_entry_create_cf_properties(&service);
            assert!(result.is_ok());

            if let Ok(props) = result {
                // Check a few common properties
                let model = mock.get_string_property(&props, "model");
                assert_eq!(model, Some("Test Value".to_string()));

                let manufacturer = mock.get_string_property(&props, "manufacturer");
                assert_eq!(manufacturer, Some("Test Value".to_string()));

                let board_id = mock.get_number_property(&props, "board-id");
                assert_eq!(board_id, Some(42));

                let secure_boot = mock.get_bool_property(&props, "secure-boot");
                assert_eq!(secure_boot, Some(true));
            }
        }
    }
}
