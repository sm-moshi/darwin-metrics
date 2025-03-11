use objc2_foundation::{NSDictionary, NSObject, NSString};

use crate::utils::mock_dictionary::MockDictionary;
use crate::utils::property_utils::{PropertyAccessor, PropertyUtils};

/// A trait that abstracts dictionary access operations
pub trait DictionaryAccess {
    /// Get a string value from the dictionary
    fn get_string(&self, key: &str) -> Option<String>;

    /// Get a number value from the dictionary
    fn get_number(&self, key: &str) -> Option<f64>;

    /// Get a boolean value from the dictionary
    fn get_bool(&self, key: &str) -> Option<bool>;
}

/// Implementation for MockDictionary
impl DictionaryAccess for MockDictionary {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_bool(key)
    }
}

/// Implementation for NSDictionary using PropertyUtils
impl DictionaryAccess for NSDictionary<NSString, NSObject> {
    fn get_string(&self, key: &str) -> Option<String> {
        <PropertyAccessor as PropertyUtils>::get_string_property(self, key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        <PropertyAccessor as PropertyUtils>::get_number_property(self, key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        <PropertyAccessor as PropertyUtils>::get_bool_property(self, key)
    }
}

/// A struct that can work with any dictionary implementation
pub struct DictionaryAccessor<T: DictionaryAccess> {
    dictionary: T,
}

impl<T: DictionaryAccess> DictionaryAccessor<T> {
    /// Create a new DictionaryAccessor with the given dictionary
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Get a string value from the dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.dictionary.get_string(key)
    }

    /// Get a number value from the dictionary
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.dictionary.get_number(key)
    }

    /// Get a boolean value from the dictionary
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.dictionary.get_bool(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_dictionary::MockValue;

    #[test]
    fn test_dictionary_accessor_with_mock() {
        let mock_dict = MockDictionary::with_entries(&[
            ("string_key", MockValue::String("string_value".to_string())),
            ("number_key", MockValue::Number(42.5)),
            ("bool_key", MockValue::Boolean(true)),
        ]);

        let accessor = DictionaryAccessor::new(mock_dict);

        assert_eq!(accessor.get_string("string_key"), Some("string_value".to_string()));
        assert_eq!(accessor.get_number("number_key"), Some(42.5));
        assert_eq!(accessor.get_bool("bool_key"), Some(true));

        // Test non-existent keys
        assert_eq!(accessor.get_string("non_existent"), None);
        assert_eq!(accessor.get_number("non_existent"), None);
        assert_eq!(accessor.get_bool("non_existent"), None);
    }
}
