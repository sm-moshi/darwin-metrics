#[cfg(test)]
mod tests {
    use crate::utils::core::dictionary::{DictionaryAccess, DictionaryAccessor, SafeDictionary};
    use crate::utils::tests::test_utils::{MockDictionary, MockValue};

    #[test]
    fn test_dictionary_access() {
        let mock_dict = MockDictionary::with_entries(&[
            ("string_key", MockValue::String("test_string".to_string())),
            ("number_key", MockValue::Number(42.5)),
            ("bool_key", MockValue::Boolean(true)),
        ]);

        // Test direct dictionary access
        assert_eq!(mock_dict.get_string("string_key"), Some("test_string".to_string()));
        assert_eq!(mock_dict.get_number("number_key"), Some(42.5));
        assert_eq!(mock_dict.get_bool("bool_key"), Some(true));

        // Test through DictionaryAccessor
        let accessor = DictionaryAccessor::new(mock_dict.clone());
        assert_eq!(accessor.get_string("string_key"), Some("test_string".to_string()));
        assert_eq!(accessor.get_number("number_key"), Some(42.5));
        assert_eq!(accessor.get_bool("bool_key"), Some(true));
    }

    #[test]
    fn test_dictionary_edge_cases() {
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
    fn test_dictionary_type_mismatches() {
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
    fn test_dictionary_special_values() {
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

    #[test]
    fn test_dictionary_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let mock_dict = Arc::new(MockDictionary::with_entries(&[
            ("str", MockValue::String("test".to_string())),
            ("num", MockValue::Number(42.0)),
            ("bool", MockValue::Boolean(true)),
        ]));

        let mut handles = vec![];

        for _ in 0..10 {
            let dict_clone: Arc<MockDictionary> = Arc::clone(&mock_dict);
            let handle = thread::spawn(move || {
                assert_eq!(dict_clone.get_string("str"), Some("test".to_string()));
                assert_eq!(dict_clone.get_number("num"), Some(42.0));
                assert_eq!(dict_clone.get_bool("bool"), Some(true));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
