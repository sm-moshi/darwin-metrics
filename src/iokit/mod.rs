//! IOKit interface for macOS system metrics
//!
//! This module provides a safe abstraction over the IOKit framework,
//! allowing access to system hardware information and metrics.
//!
//! # Safety
//!
//! This module contains unsafe FFI calls to IOKit and Core Foundation.
//! All unsafe operations are properly wrapped in safe abstractions and
//! follow proper resource management practices.
//!
//! # Thread Safety
//!
//! All types in this module implement `Send` and `Sync` where appropriate.
//! Resource cleanup is handled automatically through RAII patterns.
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
//!     let service = iokit.io_service_get_matching_service(matching);
//!     
//!     // Ensure service is released when we're done
//!     defer! { iokit.io_object_release(service); }
//!     
//!     // Get properties
//!     let properties = iokit.io_registry_entry_create_cf_properties(service)?;
//!     
//!     // Ensure properties are released when we're done
//!     defer! { iokit.cf_release(properties.cast()); }
//!     
//!     Ok(())
//! }
//! ```

use scopeguard;
use scopeguard::defer;

use crate::Error;
use core_foundation::base::{CFTypeRef, kCFAllocatorDefault};
use core_foundation::boolean::CFBooleanRef;
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::number::{CFBooleanGetValue, CFNumberGetValue, CFNumberRef, CFNumberType};
use core_foundation::string::CFStringRef;
use io_kit_sys::{
    IOObjectRelease, IORegistryEntryCreateCFProperties, IOServiceGetMatchingService,
    IOServiceMatching, kIOMasterPortDefault, types::io_service_t,
};
use mach_sys::kern_return::KERN_SUCCESS;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    fn io_service_matching(&self, service_name: &str) -> CFDictionaryRef;
    fn io_service_get_matching_service(&self, matching: CFDictionaryRef) -> io_service_t;
    fn io_registry_entry_create_cf_properties(
        &self,
        entry: io_service_t,
    ) -> Result<CFMutableDictionaryRef, Error>;
    fn io_object_release(&self, obj: io_service_t);
    fn cf_release(&self, cf: CFTypeRef);
    fn cf_dictionary_get_value(&self, dict: CFDictionaryRef, key: CFStringRef) -> CFTypeRef;
    fn cf_number_get_value(&self, number: CFNumberRef, number_type: CFNumberType) -> Option<i64>;
    fn cf_boolean_get_value(&self, boolean: CFBooleanRef) -> bool;
}

#[derive(Debug, Default)]
pub struct IOKitImpl;

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, service_name: &str) -> CFDictionaryRef {
        unsafe { IOServiceMatching(format!("{}\0", service_name).as_ptr() as *const i8) }
    }

    fn io_service_get_matching_service(&self, matching: CFDictionaryRef) -> io_service_t {
        unsafe { IOServiceGetMatchingService(kIOMasterPortDefault, matching) }
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        entry: io_service_t,
    ) -> Result<CFMutableDictionaryRef, Error> {
        let mut props: CFMutableDictionaryRef = std::ptr::null_mut();
        let kr =
            unsafe { IORegistryEntryCreateCFProperties(entry, &mut props, kCFAllocatorDefault, 0) };
        if kr != KERN_SUCCESS {
            return Err(Error::SystemError(format!(
                "Failed to get properties: {}",
                kr
            )));
        }
        Ok(props)
    }

    fn io_object_release(&self, obj: io_service_t) {
        unsafe { IOObjectRelease(obj) };
    }

    fn cf_release(&self, cf: CFTypeRef) {
        unsafe { core_foundation::base::CFRelease(cf) };
    }

    fn cf_dictionary_get_value(&self, dict: CFDictionaryRef, key: CFStringRef) -> CFTypeRef {
        unsafe { core_foundation::dictionary::CFDictionaryGetValue(dict, key as *const _) }
    }

    fn cf_number_get_value(&self, number: CFNumberRef, number_type: CFNumberType) -> Option<i64> {
        let mut value: i64 = 0;
        let success =
            unsafe { CFNumberGetValue(number, number_type, &mut value as *mut _ as *mut _) };
        if success { Some(value) } else { None }
    }

    fn cf_boolean_get_value(&self, boolean: CFBooleanRef) -> bool {
        unsafe { CFBooleanGetValue(boolean) }
    }
}
