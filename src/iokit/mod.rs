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
//! use scopeguard::defer;
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

use objc2::runtime::{AnyObject, AnyClass};
use objc2::{msg_send, class};
use objc2_foundation::{NSString, NSObject, NSDictionary, NSNumber};
use objc2::rc::Retained;
use std::ffi::CStr;
use scopeguard::defer;

use crate::Error;

#[cfg(test)]
use mockall::automock;

/// Type alias for IOKit objects
type IOObject = Retained<AnyObject>;

#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    fn io_service_matching(&self, service_name: &str) -> Retained<NSDictionary<NSString, NSObject>>;
    fn io_service_get_matching_service(&self, matching: &NSDictionary<NSString, NSObject>) -> Option<Retained<AnyObject>>;
    fn io_registry_entry_create_cf_properties(
        &self,
        entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>, Error>;
    fn io_object_release(&self, obj: &AnyObject);
    fn get_string_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String>;
    fn get_number_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<i64>;
    fn get_bool_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool>;
}

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, service_name: &str) -> Retained<NSDictionary<NSString, NSObject>> {
        unsafe {
            let class = AnyClass::get(CStr::from_bytes_with_nul(b"IOService\0").unwrap()).unwrap();
            let service_name = NSString::from_str(service_name);
            let dict: *mut AnyObject = msg_send![class, serviceMatching:&*service_name];
            Retained::from_raw(dict.cast()).unwrap()
        }
    }

    fn io_service_get_matching_service(&self, matching: &NSDictionary<NSString, NSObject>) -> Option<Retained<AnyObject>> {
        unsafe {
            let class = AnyClass::get(CStr::from_bytes_with_nul(b"IOService\0").unwrap()).unwrap();
            let service: *mut AnyObject = msg_send![class, serviceMatching:&*matching];
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

    fn get_string_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
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

    fn get_number_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<i64> {
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

    fn get_bool_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
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

    #[test]
    fn test_mock_iokit() {
        let mut mock = MockIOKit::new();
        let service_name = "TestService";
        
        // Set up expectations
        mock.expect_io_service_matching()
            .with(eq(service_name))
            .times(1)
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Retained::from_raw(dict.cast()).unwrap()
            });

        // Call the method to test
        let _ = mock.io_service_matching(service_name);
    }
}
