use objc2::rc::Retained;
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
pub trait PropertyUtils {
    fn get_string_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<String> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSString>().ok())
            .map(|s| s.to_string())
    }

    fn get_number_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n: Retained<NSNumber>| n.as_f64())
    }

    fn get_bool_property(dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
        let ns_key = NSString::from_str(key);
        unsafe { dict.valueForKey(&ns_key) }
            .and_then(|obj| obj.downcast::<NSNumber>().ok())
            .map(|n: Retained<NSNumber>| n.as_bool())
    }
}

pub struct PropertyAccessor;

impl PropertyUtils for PropertyAccessor {}
