use std::collections::HashMap;

/// A value type that can be stored in a MockDictionary
#[derive(Debug, Clone)]
pub enum MockValue {
    /// A string value stored in the mock dictionary
    String(String),
    /// A numeric value stored in the mock dictionary
    Number(f64),
    /// A boolean value stored in the mock dictionary
    Boolean(bool),
}

impl From<&str> for MockValue {
    fn from(s: &str) -> Self {
        MockValue::String(s.to_string())
    }
}

impl From<String> for MockValue {
    fn from(s: String) -> Self {
        MockValue::String(s)
    }
}

impl From<f64> for MockValue {
    fn from(n: f64) -> Self {
        MockValue::Number(n)
    }
}

impl From<i64> for MockValue {
    fn from(n: i64) -> Self {
        MockValue::Number(n as f64)
    }
}

impl From<bool> for MockValue {
    fn from(b: bool) -> Self {
        MockValue::Boolean(b)
    }
}

/// A pure Rust mock dictionary that can be used for testing without Objective-C
#[derive(Debug, Clone, Default)]
pub struct MockDictionary {
    entries: HashMap<String, MockValue>,
}

impl MockDictionary {
    /// Create a new empty mock dictionary
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Create a mock dictionary with the given entries
    pub fn with_entries<K, V>(entries: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: Into<MockValue> + Clone,
    {
        let mut dict = Self::new();
        for (key, value) in entries {
            dict.entries.insert(key.as_ref().to_string(), value.clone().into());
        }
        dict
    }

    /// Get a string value from the dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.entries.get(key) {
            Some(MockValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get a number value from the dictionary
    pub fn get_number(&self, key: &str) -> Option<f64> {
        match self.entries.get(key) {
            Some(MockValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    /// Get a boolean value from the dictionary
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(MockValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Insert a value into the dictionary
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<MockValue>,
    {
        self.entries.insert(key.as_ref().to_string(), value.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test data
    fn create_test_entries() -> Vec<(&'static str, MockValue)> {
        vec![
            ("string_key", MockValue::String("string_value".to_string())),
            ("number_key", MockValue::Number(42.5)),
            ("bool_key", MockValue::Boolean(true)),
        ]
    }

    #[test]
    fn test_mock_dictionary_creation() {
        let dict = MockDictionary::new();
        assert!(dict.entries.is_empty());
    }

    #[test]
    fn test_mock_dictionary_with_entries() {
        let entries = create_test_entries();
        let dict = MockDictionary::with_entries(&entries);

        assert_eq!(dict.entries.len(), 3);
        assert_eq!(dict.get_string("string_key"), Some("string_value".to_string()));
        assert_eq!(dict.get_number("number_key"), Some(42.5));
        assert_eq!(dict.get_bool("bool_key"), Some(true));
    }

    #[test]
    fn test_mock_dictionary_get_methods() {
        let mut dict = MockDictionary::new();
        let entries = create_test_entries();

        for (key, value) in entries {
            dict.insert(key, value);
        }

        // Test successful retrievals
        assert_eq!(dict.get_string("string_key"), Some("string_value".to_string()));
        assert_eq!(dict.get_number("number_key"), Some(42.5));
        assert_eq!(dict.get_bool("bool_key"), Some(true));

        // Test type mismatches
        assert_eq!(dict.get_string("number_key"), None);
        assert_eq!(dict.get_number("string_key"), None);
        assert_eq!(dict.get_bool("string_key"), None);

        // Test non-existent keys
        assert_eq!(dict.get_string("non_existent"), None);
        assert_eq!(dict.get_number("non_existent"), None);
        assert_eq!(dict.get_bool("non_existent"), None);
    }
}
