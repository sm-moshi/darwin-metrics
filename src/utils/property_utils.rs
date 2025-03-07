use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
use objc2::rc::autoreleasepool;
use objc2::msg_send;

pub struct PropertyUtils;

impl PropertyUtils {
    pub fn get_string_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        autoreleasepool(|pool| {
            let ns_key = NSString::from_str(key);
            let value: *mut NSObject = unsafe { msg_send![dict, objectForKey:&*ns_key] };
            if value.is_null() {
                return None;
            }
            let ns_string: *mut NSString = unsafe { msg_send![value, description] };
            if ns_string.is_null() {
                return None;
            }
            Some(unsafe { (*ns_string).to_str(pool).to_string() })
        })
    }

    pub fn get_number_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<f64> {
        autoreleasepool(|_pool| {
            let ns_key = NSString::from_str(key);
            let value: *mut NSNumber = unsafe { msg_send![dict, objectForKey:&*ns_key] };
            if value.is_null() {
                return None;
            }
            Some(unsafe { msg_send![value, doubleValue] })
        })
    }

    pub fn get_bool_property(
        dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
        autoreleasepool(|_pool| {
            let ns_key = NSString::from_str(key);
            let value: *mut NSNumber = unsafe { msg_send![dict, objectForKey:&*ns_key] };
            if value.is_null() {
                return None;
            }
            Some(unsafe { msg_send![value, boolValue] })
        })
    }
}

pub struct PropertyAccessor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_access() {
    }
}
