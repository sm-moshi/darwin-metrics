use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

/// Macro to generate property access methods
macro_rules! define_property_accessor {
    ($name:ident, $type:ty, $converter:expr) => {
        fn $name(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<$type> {
            let ns_key = NSString::from_str(key);
            unsafe { dict.valueForKey(&ns_key) }.and_then(|obj| obj.downcast::<NSNumber>().ok()).map($converter)
        }
    };
    ($name:ident, String) => {
        fn $name(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
            let ns_key = NSString::from_str(key);
            unsafe { dict.valueForKey(&ns_key) }.and_then(|obj| obj.downcast::<NSString>().ok()).map(|s| s.to_string())
        }
    };
}

/// Trait for common property access patterns in IOKit and Foundation
pub trait PropertyUtils {
    // Get a string property from a dictionary
    define_property_accessor!(get_string_property, String);

    // Get a number property from a dictionary
    define_property_accessor!(get_number_property, f64, |n| n.as_f64());

    // Get a boolean property from a dictionary
    define_property_accessor!(get_bool_property, bool, |n| n.as_bool());

    // Helper methods for external use
    fn get_string(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
        self.get_string_property(dict, key)
    }

    fn get_number(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        self.get_number_property(dict, key)
    }

    fn get_bool(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
        self.get_bool_property(dict, key)
    }
}

/// Default implementation of PropertyUtils
pub struct PropertyAccessor;

impl PropertyUtils for PropertyAccessor {}
