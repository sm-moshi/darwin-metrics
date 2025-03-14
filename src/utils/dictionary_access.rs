use crate::utils::SafeDictionary;
use objc2::rc::Retained;
use objc2_foundation::{NSDictionary, NSNumber, NSObject, NSString};
use std::ops::Deref;

use crate::utils::mock_dictionary::MockDictionary;
use crate::utils::property_utils::{PropertyAccessor, PropertyUtils};

/// Trait for accessing dictionary values in a type-safe way
pub trait DictionaryAccess {
    /// Get a string value for the given key
    fn get_string(&self, key: &str) -> Option<String>;

    /// Get a number value for the given key
    fn get_number(&self, key: &str) -> Option<f64>;

    /// Get a boolean value for the given key
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Get a dictionary value for the given key
    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary>;
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

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        None
    }
}

/// Implementation for NSDictionary using PropertyUtils
impl DictionaryAccess for NSDictionary<NSString, NSObject> {
    fn get_string(&self, key: &str) -> Option<String> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(string) = value.downcast::<NSString>() {
            Some(string.to_string())
        } else {
            None
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_f64())
        } else {
            None
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(number) = value.downcast::<NSNumber>() {
            Some(number.as_bool())
        } else {
            None
        }
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        let key = NSString::from_str(key);
        let value = unsafe { self.valueForKey(&key) }?;
        if let Ok(dict) = value.downcast::<NSDictionary>() {
            Some(SafeDictionary::from(dict))
        } else {
            None
        }
    }
}

/// Implementation for Retained NSDictionary
impl DictionaryAccess for Retained<NSDictionary<NSString, NSObject>> {
    fn get_string(&self, key: &str) -> Option<String> {
        self.deref().get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.deref().get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.deref().get_bool(key)
    }

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        self.deref().get_dictionary(key)
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

    /// Get a dictionary value from the dictionary
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        self.dictionary.get_dictionary(key)
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
