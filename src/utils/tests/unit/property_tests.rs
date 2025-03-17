#[cfg(test)]
mod tests {
    use crate::utils::core::property::PropertyUtils;
    use crate::utils::tests::test_utils::{MockDictionary, MockValue};
    use objc2_foundation::{NSDictionary, NSObject, NSString};

    #[test]
    fn test_property_utils_trait() {
        let mock_dict = MockDictionary::with_entries(&[]);
        assert_eq!(mock_dict.get_string("nonexistent"), None);
    }

    #[test]
    fn test_property_utils_with_mock() {
        let mock_dict = MockDictionary::with_entries(&[
            ("string_key", MockValue::String("test_string".to_string())),
            ("number_key", MockValue::Number(42.5)),
            ("bool_key", MockValue::Boolean(true)),
        ]);

        assert_eq!(mock_dict.get_string("string_key"), Some("test_string".to_string()));
        assert_eq!(mock_dict.get_number("number_key"), Some(42.5));
        assert_eq!(mock_dict.get_bool("bool_key"), Some(true));

        // Test nonexistent keys
        assert_eq!(mock_dict.get_string("nonexistent"), None);
        assert_eq!(mock_dict.get_number("nonexistent"), None);
        assert_eq!(mock_dict.get_bool("nonexistent"), None);

        // Test type mismatches
        assert_eq!(mock_dict.get_string("number_key"), None);
        assert_eq!(mock_dict.get_number("string_key"), None);
        assert_eq!(mock_dict.get_bool("string_key"), None);
    }

    #[test]
    fn test_property_utils_edge_cases() {
        let mock_dict = MockDictionary::with_entries(&[
            ("empty", MockValue::String("".to_string())),
            ("special", MockValue::String("Hello\n\t\r\0World".to_string())),
            ("unicode", MockValue::String("Hello 世界 🦀".to_string())),
        ]);

        assert_eq!(mock_dict.get_string("empty"), Some("".to_string()));
        assert_eq!(mock_dict.get_string("special"), Some("Hello\n\t\r\0World".to_string()));
        assert_eq!(mock_dict.get_string("unicode"), Some("Hello 世界 🦀".to_string()));
    }

    #[test]
    fn test_property_utils_type_conversions() {
        let mock_dict = MockDictionary::with_entries(&[
            ("str", MockValue::String("42".to_string())),
            ("num", MockValue::Number(1.0)),
            ("bool", MockValue::Boolean(true)),
        ]);

        assert_eq!(mock_dict.get_number("str"), None);
        assert_eq!(mock_dict.get_bool("str"), None);
        assert_eq!(mock_dict.get_string("num"), None);
        assert_eq!(mock_dict.get_bool("num"), None);
        assert_eq!(mock_dict.get_string("bool"), None);
        assert_eq!(mock_dict.get_number("bool"), None);
    }

    #[test]
    fn test_property_utils_special_values() {
        let mock_dict = MockDictionary::with_entries(&[
            ("inf", MockValue::Number(f64::INFINITY)),
            ("neg_inf", MockValue::Number(f64::NEG_INFINITY)),
            ("nan", MockValue::Number(f64::NAN)),
            ("max", MockValue::Number(f64::MAX)),
            ("min", MockValue::Number(f64::MIN)),
        ]);

        assert!(mock_dict.get_number("inf").unwrap().is_infinite());
        assert!(mock_dict.get_number("neg_inf").unwrap().is_infinite());
        assert!(mock_dict.get_number("nan").unwrap().is_nan());
        assert_eq!(mock_dict.get_number("max"), Some(f64::MAX));
        assert_eq!(mock_dict.get_number("min"), Some(f64::MIN));
    }
}
