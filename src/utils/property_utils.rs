use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

/// Trait for common property access patterns in IOKit and Foundation
pub trait PropertyUtils {
    /// Get a string property from a dictionary
    fn get_string_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSString>().ok())
            .map(|s| s.to_string())
    }

    /// Get a number property from a dictionary
    fn get_number_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n| n.as_f64())
    }

    /// Get a boolean property from a dictionary
    fn get_bool_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n| n.as_bool())
    }
}

/// Default implementation of PropertyUtils
pub struct PropertyAccessor;

impl PropertyUtils for PropertyAccessor {}
