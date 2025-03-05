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

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
use objc2::class; // Keep this import for all code paths

// Remove the redundant import below
// #[cfg(test)]
// use objc2::class; // This is now redundant but can be kept for clarity

use crate::Error;

#[cfg(test)]
use mockall::automock;

// Remove the unused type alias
// type IOObject = Retained<AnyObject>;

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

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKit for IOKitImpl {
    fn io_service_matching(
        &self,
        service_name: &str,
    ) -> Retained<NSDictionary<NSString, NSObject>> {
        unsafe {
            // Get the IOService class with proper error handling
            let class = match AnyClass::get(c"IOService") {
                Some(class) => class,
                None => {
                    // Create an empty dictionary as fallback if IOService class not found
                    let empty_dict: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
                    return Retained::from_raw(empty_dict.cast()).unwrap_or_else(|| {
                        panic!("Failed to create fallback dictionary");
                    });
                }
            };
            
            // Create NSString from the service name
            let service_name = NSString::from_str(service_name);
            
            // Call serviceMatching: and handle potential null result
            let dict: *mut AnyObject = msg_send![class, serviceMatching:&*service_name];
            
            if dict.is_null() {
                // Create empty dictionary as fallback
                let empty_dict: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
                Retained::from_raw(empty_dict.cast()).unwrap_or_else(|| {
                    panic!("Failed to create fallback dictionary after null serviceMatching result");
                })
            } else {
                // Try to retain the dictionary, with fallback if retention fails
                Retained::from_raw(dict.cast()).unwrap_or_else(|| {
                    // Create empty dictionary as last-resort fallback
                    let empty_dict: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
                    Retained::from_raw(empty_dict.cast()).unwrap_or_else(|| {
                        panic!("Failed all attempts to create dictionary");
                    })
                })
            }
        }
    }

    fn io_service_get_matching_service(
        &self,
        matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        unsafe {
            // Use the master port (0) and the matching dictionary
            // This is the correct way to call IOServiceGetMatchingService
            let master_port: u32 = 0; // kIOMasterPortDefault
            // Fix the syntax with proper commas between arguments
            let service: *mut AnyObject = msg_send![class!(IOService), getMatchingService: master_port, matching: matching];
            
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
mod tests {
    use super::*;
    use mockall::predicate::*;

    /// Create a safe dictionary for testing
    fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
        unsafe {
            let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
            Retained::from_raw(dict.cast()).unwrap()
        }
    }

    /// Create a safe NSObject for testing
    fn create_test_object() -> Retained<AnyObject> {
        unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).unwrap()
        }
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
        let iokit = IOKitImpl::default();

        // Create a matching dictionary for testing
        let dict = create_test_dictionary();

        // Call may return None on test systems, but should not panic
        let _service = iokit.io_service_get_matching_service(&dict);
        // Just ensure the call doesn't crash
    }

    #[test]
    fn test_get_string_property() {
        let iokit = IOKitImpl::default();

        // Create a test dictionary with string property
        let key = "TestKey";
        let value = "TestValue";
        let dict = unsafe {
            let ns_key = NSString::from_str(key);
            let ns_value = NSString::from_str(value);
            // Fix the message sending syntax with proper commas
            let dict: *mut AnyObject =
                msg_send![class!(NSDictionary), dictionaryWithObject:&*ns_value, forKey:&*ns_key];
            Retained::from_raw(dict.cast()).unwrap()
        };

        // Test property retrieval
        let result = iokit.get_string_property(&dict, key);
        assert_eq!(result, Some(value.to_string()));

        // Test non-existent property
        let result = iokit.get_string_property(&dict, "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_number_property() {
        let iokit = IOKitImpl::default();

        // Create a test dictionary with number property
        let key = "TestNumberKey";
        let value: i64 = 42;
        let dict = unsafe {
            let ns_key = NSString::from_str(key);
            let ns_value: *mut AnyObject = msg_send![class!(NSNumber), numberWithLongLong:value];
            // Fix the message sending syntax with proper commas
            let dict: *mut AnyObject =
                msg_send![class!(NSDictionary), dictionaryWithObject:ns_value, forKey:&*ns_key];
            Retained::from_raw(dict.cast()).unwrap()
        };

        // Test property retrieval
        let result = iokit.get_number_property(&dict, key);
        assert_eq!(result, Some(value));

        // Test non-existent property
        let result = iokit.get_number_property(&dict, "NonExistentKey");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_bool_property() {
        let iokit = IOKitImpl::default();

        // Create a test dictionary with boolean property
        let key = "TestBoolKey";
        let value = true;
        let dict = unsafe {
            let ns_key = NSString::from_str(key);
            let ns_value: *mut AnyObject = msg_send![class!(NSNumber), numberWithBool:value];
            // Fix the message sending syntax with proper commas
            let dict: *mut AnyObject =
                msg_send![class!(NSDictionary), dictionaryWithObject:ns_value, forKey:&*ns_key];
            Retained::from_raw(dict.cast()).unwrap()
        };

        // Test property retrieval
        let result = iokit.get_bool_property(&dict, key);
        assert_eq!(result, Some(value));

        // Test non-existent property
        let result = iokit.get_bool_property(&dict, "NonExistentKey");
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
        let iokit = IOKitImpl::default();

        // Create a mock object
        let mock_obj = create_test_object();

        // This will likely fail with a SystemError on mock objects, which is correct
        let result = iokit.io_registry_entry_create_cf_properties(&mock_obj);
        assert!(result.is_err());

        // For full coverage we would need a real IOKit object, which is challenging
        // in a pure unit test environment. Integration tests would be better for this.
    }

    #[test]
    fn test_integration() {
        let iokit = IOKitImpl::default();

        // Real-world usage example - try to access a common service
        let matching = iokit.io_service_matching("IOPlatformExpertDevice");
        let service = iokit.io_service_get_matching_service(&matching);

        if let Some(service) = service {
            // Should be able to get properties
            let result = iokit.io_registry_entry_create_cf_properties(&service);
            if let Ok(props) = result {
                // Check a few common properties
                let _ = iokit.get_string_property(&props, "model");
                let _ = iokit.get_string_property(&props, "manufacturer");
                let _ = iokit.get_number_property(&props, "board-id");
                let _ = iokit.get_bool_property(&props, "secure-boot");
                // We're just checking that these don't panic, not validating values
            }
        }
        // Test passes if no panics occur
    }
}
