use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};

/// Trait for common property access patterns
pub trait PropertyAccess {
    /// Get a string property from a dictionary
    fn get_string_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        let ns_key = NSString::from_str(key);
        dict.get(&ns_key)
            .and_then(|obj| obj.downcast_ref::<NSString>())
            .map(|ns_str| ns_str.to_string())
    }

    /// Get a number property from a dictionary
    fn get_number_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64> {
        let ns_key = NSString::from_str(key);
        dict.get(&ns_key)
            .and_then(|obj| obj.downcast_ref::<NSNumber>())
            .map(|num| num.int_value())
    }

    /// Get a boolean property from a dictionary
    fn get_bool_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
        let ns_key = NSString::from_str(key);
        dict.get(&ns_key)
            .and_then(|obj| obj.downcast_ref::<NSNumber>())
            .map(|num| num.bool_value())
    }
}

/// Default implementation of PropertyAccess
pub struct PropertyAccessor;

impl PropertyAccess for PropertyAccessor {}
