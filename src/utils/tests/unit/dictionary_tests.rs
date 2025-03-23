#[cfg(test)]
mod tests {

    use crate::utils::core::dictionary::SafeDictionary;

    #[test]
    fn test_dictionary_empty() {
        let dict = SafeDictionary::new();
        // Test that the dictionary is initially empty
        assert!(dict.get_string("test_key").is_none());
    }

    #[test]
    fn test_dictionary_set_get() {
        let mut dict = SafeDictionary::new();
        let key = "test_key";

        // Use set_f64 since that's what we have available
        dict.set_f64(key, 42.0);

        // Verify the value was set
        let result = dict.get_number(key);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 42.0);
    }

    #[test]
    fn test_dictionary_set_bool() {
        let mut dict = SafeDictionary::new();
        let key = "bool_key";

        // Set a boolean value
        dict.set_bool(key, true);

        // Verify the value was set
        let result = dict.get_bool(key);
        assert!(result.is_some());
        assert!(result.unwrap());
    }
}
