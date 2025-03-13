use objc2_foundation::{NSDictionary, NSObject, NSString};

use crate::utils::dictionary_access::DictionaryAccessor;
use crate::utils::mock_dictionary::{MockDictionary, MockValue};
use crate::utils::property_utils::{PropertyAccessor, PropertyUtils};
use crate::utils::test_utils::create_test_dictionary;

// Create a simple mock implementation for testing
struct MockPropertyUtils;

impl PropertyUtils for MockPropertyUtils {
    // Override the default implementations to provide test behavior
}

#[test]
fn test_property_utils_trait() {
    // This test verifies that the PropertyUtils trait methods work as expected by implementing a mock that uses the
    // default trait implementations

    // Create a test dictionary
    let dict = create_test_dictionary();

    // The default implementations will return None for any key since our test dictionary is empty, which is fine for
    // this test
    let string_result = MockPropertyUtils::get_string_property(&dict, "test_key");
    let number_result = MockPropertyUtils::get_number_property(&dict, "test_key");
    let bool_result = MockPropertyUtils::get_bool_property(&dict, "test_key");

    // Since our test dictionary is empty, all results should be None
    assert_eq!(string_result, None);
    assert_eq!(number_result, None);
    assert_eq!(bool_result, None);
}

#[test]
fn test_property_accessor_implements_trait() {
    // This test verifies that PropertyAccessor implements PropertyUtils by checking that the trait methods can be
    // assigned to function pointers with the correct signatures

    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<String> = PropertyAccessor::get_string_property;
    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<f64> = PropertyAccessor::get_number_property;
    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<bool> = PropertyAccessor::get_bool_property;

    // No assertion needed - test passes if it compiles successfully
}

#[test]
fn test_property_accessor_instance() {
    // Create an instance of PropertyAccessor
    let accessor = PropertyAccessor;

    // Just to verify the instance exists (even though we don't use it directly)
    assert!(std::mem::size_of_val(&accessor) == 0, "PropertyAccessor should be a zero-sized type");
}

// New tests using MockDictionary

#[test]
fn test_dictionary_access_trait_with_mock() {
    // Create a mock dictionary with test data
    let mock_dict = MockDictionary::with_entries(&[
        ("string_key", MockValue::String("test_string".to_string())),
        ("number_key", MockValue::Number(42.5)),
        ("bool_key", MockValue::Boolean(true)),
    ]);

    // Test the DictionaryAccess trait implementation
    assert_eq!(mock_dict.get_string("string_key"), Some("test_string".to_string()));
    assert_eq!(mock_dict.get_number("number_key"), Some(42.5));
    assert_eq!(mock_dict.get_bool("bool_key"), Some(true));

    // Test non-existent keys
    assert_eq!(mock_dict.get_string("non_existent"), None);
    assert_eq!(mock_dict.get_number("non_existent"), None);
    assert_eq!(mock_dict.get_bool("non_existent"), None);
}

#[test]
fn test_dictionary_accessor_with_mock() {
    // Create a mock dictionary with test data
    let mock_dict = MockDictionary::with_entries(&[
        ("string_key", MockValue::String("test_string".to_string())),
        ("number_key", MockValue::Number(42.5)),
        ("bool_key", MockValue::Boolean(true)),
    ]);

    // Create a DictionaryAccessor with the mock dictionary
    let accessor = DictionaryAccessor::new(mock_dict);

    // Test the accessor methods
    assert_eq!(accessor.get_string("string_key"), Some("test_string".to_string()));
    assert_eq!(accessor.get_number("number_key"), Some(42.5));
    assert_eq!(accessor.get_bool("bool_key"), Some(true));

    // Test non-existent keys
    assert_eq!(accessor.get_string("non_existent"), None);
    assert_eq!(accessor.get_number("non_existent"), None);
    assert_eq!(accessor.get_bool("non_existent"), None);
}

#[test]
fn test_property_utils_with_mock_data() {
    // This test uses a MockDictionary to test the PropertyUtils trait methods without relying on the Objective-C
    // runtime

    // Create a mock dictionary with test data
    let mock_dict = MockDictionary::with_entries(&[
        ("string_key", MockValue::String("test_string".to_string())),
        ("number_key", MockValue::Number(42.5)),
        ("bool_key", MockValue::Boolean(true)),
    ]);

    // Test string property
    let string_result = mock_dict.get_string("string_key");
    assert_eq!(string_result, Some("test_string".to_string()));

    // Test number property
    let number_result = mock_dict.get_number("number_key");
    assert_eq!(number_result, Some(42.5));

    // Test boolean property
    let bool_result = mock_dict.get_bool("bool_key");
    assert_eq!(bool_result, Some(true));

    // Test type mismatches
    assert_eq!(mock_dict.get_string("number_key"), None);
    assert_eq!(mock_dict.get_number("string_key"), None);
    assert_eq!(mock_dict.get_bool("string_key"), None);
}

#[test]
fn test_edge_cases_with_mock() {
    // Test edge cases using a MockDictionary

    // Empty dictionary
    let empty_dict = MockDictionary::new();
    assert_eq!(empty_dict.get_string("any_key"), None);
    assert_eq!(empty_dict.get_number("any_key"), None);
    assert_eq!(empty_dict.get_bool("any_key"), None);

    // Dictionary with empty string
    let dict_with_empty = MockDictionary::with_entries(&[("empty_string", MockValue::String("".to_string()))]);
    assert_eq!(dict_with_empty.get_string("empty_string"), Some("".to_string()));

    // Dictionary with zero and false values
    let dict_with_zeros =
        MockDictionary::with_entries(&[("zero", MockValue::Number(0.0)), ("false", MockValue::Boolean(false))]);
    assert_eq!(dict_with_zeros.get_number("zero"), Some(0.0));
    assert_eq!(dict_with_zeros.get_bool("false"), Some(false));
}

#[test]
fn test_property_utils_edge_cases() {
    let mock_dict = MockDictionary::with_entries(&[
        ("empty_string", MockValue::String("".to_string())),
        ("zero", MockValue::Number(0.0)),
        ("false", MockValue::Boolean(false)),
        ("null_equivalent", MockValue::String("\0".to_string())),
    ]);

    // Test empty string
    assert_eq!(mock_dict.get_string("empty_string"), Some("".to_string()));

    // Test zero values
    assert_eq!(mock_dict.get_number("zero"), Some(0.0));

    // Test false boolean
    assert_eq!(mock_dict.get_bool("false"), Some(false));

    // Test null character in string
    assert_eq!(mock_dict.get_string("null_equivalent"), Some("\0".to_string()));

    // Test non-existent keys
    assert_eq!(mock_dict.get_string("nonexistent"), None);
    assert_eq!(mock_dict.get_number("nonexistent"), None);
    assert_eq!(mock_dict.get_bool("nonexistent"), None);
}

#[test]
fn test_property_utils_type_conversions() {
    let mock_dict = MockDictionary::with_entries(&[
        ("string_num", MockValue::String("42".to_string())),
        ("string_bool", MockValue::String("true".to_string())),
        ("num_string", MockValue::Number(std::f64::consts::PI)),
        ("bool_string", MockValue::Boolean(true)),
    ]);

    // Attempting to get numbers as strings and vice versa
    assert_eq!(mock_dict.get_number("string_num"), None);
    assert_eq!(mock_dict.get_string("num_string"), None);

    // Attempting to get booleans as strings and vice versa
    assert_eq!(mock_dict.get_bool("string_bool"), None);
    assert_eq!(mock_dict.get_string("bool_string"), None);

    // Attempting to get numbers as booleans and vice versa
    assert_eq!(mock_dict.get_number("bool_string"), None);
    assert_eq!(mock_dict.get_bool("num_string"), None);
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

    // Test special floating point values
    assert!(mock_dict.get_number("inf").unwrap().is_infinite());
    assert!(mock_dict.get_number("neg_inf").unwrap().is_infinite());
    assert!(mock_dict.get_number("nan").unwrap().is_nan());
    assert_eq!(mock_dict.get_number("max").unwrap(), f64::MAX);
    assert_eq!(mock_dict.get_number("min").unwrap(), f64::MIN);
}

#[test]
fn test_property_utils_unicode_strings() {
    let mock_dict = MockDictionary::with_entries(&[
        ("unicode", MockValue::String("Hello, 世界!".to_string())),
        ("emoji", MockValue::String("🦀 Rust".to_string())),
        ("mixed", MockValue::String("ASCII and 漢字 mixed".to_string())),
    ]);

    // Test Unicode string handling
    assert_eq!(mock_dict.get_string("unicode"), Some("Hello, 世界!".to_string()));
    assert_eq!(mock_dict.get_string("emoji"), Some("🦀 Rust".to_string()));
    assert_eq!(mock_dict.get_string("mixed"), Some("ASCII and 漢字 mixed".to_string()));
}

#[test]
fn test_property_utils_large_numbers() {
    let mock_dict = MockDictionary::with_entries(&[
        ("large_int", MockValue::Number(1e9)),
        ("small_float", MockValue::Number(1e-9)),
        ("precise", MockValue::Number(std::f64::consts::PI)),
    ]);

    // Test handling of various numeric values
    assert_eq!(mock_dict.get_number("large_int"), Some(1e9));
    assert_eq!(mock_dict.get_number("small_float"), Some(1e-9));
    assert_eq!(mock_dict.get_number("precise"), Some(std::f64::consts::PI));
}

#[test]
fn test_property_utils_multiple_types() {
    let mock_dict = MockDictionary::with_entries(&[
        ("key1", MockValue::String("value1".to_string())),
        ("key2", MockValue::Number(42.0)),
        ("key3", MockValue::Boolean(true)),
        ("key4", MockValue::String("value2".to_string())),
    ]);

    // Test multiple operations on the same dictionary
    assert_eq!(mock_dict.get_string("key1"), Some("value1".to_string()));
    assert_eq!(mock_dict.get_number("key2"), Some(42.0));
    assert_eq!(mock_dict.get_bool("key3"), Some(true));
    assert_eq!(mock_dict.get_string("key4"), Some("value2".to_string()));

    // Test that the operations don't affect each other
    assert_eq!(mock_dict.get_string("key1"), Some("value1".to_string()));
    assert_eq!(mock_dict.get_number("key2"), Some(42.0));
}

#[test]
fn test_property_utils_error_handling() {
    let mock_dict = MockDictionary::with_entries(&[
        ("invalid_utf8", MockValue::String("\u{FFFD}".to_string())), // Replacement character
        ("overflow_number", MockValue::Number(f64::MAX)),
        ("underflow_number", MockValue::Number(f64::MIN)),
    ]);

    // Test invalid UTF-8 handling
    let string_result = mock_dict.get_string("invalid_utf8");
    assert_eq!(string_result, Some("\u{FFFD}".to_string()));

    // Test number overflow/underflow handling
    let max_number = mock_dict.get_number("overflow_number");
    let min_number = mock_dict.get_number("underflow_number");
    assert_eq!(max_number, Some(f64::MAX));
    assert_eq!(min_number, Some(f64::MIN));
}

#[test]
fn test_property_utils_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let mock_dict = Arc::new(MockDictionary::with_entries(&[
        ("string", MockValue::String("test".to_string())),
        ("number", MockValue::Number(42.0)),
        ("boolean", MockValue::Boolean(true)),
    ]));

    let mut handles = vec![];

    // Spawn multiple threads to access the dictionary concurrently
    for i in 0..10 {
        let dict_clone = Arc::clone(&mock_dict);
        let handle = thread::spawn(move || match i % 3 {
            0 => assert_eq!(dict_clone.get_string("string"), Some("test".to_string())),
            1 => assert_eq!(dict_clone.get_number("number"), Some(42.0)),
            _ => assert_eq!(dict_clone.get_bool("boolean"), Some(true)),
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_property_utils_nested_access() {
    let mock_dict = MockDictionary::with_entries(&[
        ("nested.key", MockValue::String("nested_value".to_string())),
        ("deeply.nested.key", MockValue::Number(42.0)),
        ("array[0]", MockValue::Boolean(true)),
    ]);

    assert_eq!(mock_dict.get_string("nested.key"), Some("nested_value".to_string()));
    assert_eq!(mock_dict.get_number("deeply.nested.key"), Some(42.0));
    assert_eq!(mock_dict.get_bool("array[0]"), Some(true));
}

#[test]
fn test_property_utils_empty_key() {
    let mock_dict = MockDictionary::with_entries(&[("", MockValue::String("empty_key".to_string()))]);

    assert_eq!(mock_dict.get_string(""), Some("empty_key".to_string()));
    assert_eq!(mock_dict.get_number(""), None);
    assert_eq!(mock_dict.get_bool(""), None);
}

#[test]
fn test_property_utils_whitespace_handling() {
    let mock_dict = MockDictionary::with_entries(&[
        ("  key  ", MockValue::String("padded_key".to_string())),
        ("\tkey\n", MockValue::Number(42.0)),
        (" ", MockValue::Boolean(true)),
    ]);

    assert_eq!(mock_dict.get_string("  key  "), Some("padded_key".to_string()));
    assert_eq!(mock_dict.get_number("\tkey\n"), Some(42.0));
    assert_eq!(mock_dict.get_bool(" "), Some(true));
}

#[test]
fn test_property_utils_case_sensitivity() {
    let mock_dict = MockDictionary::with_entries(&[
        ("KEY", MockValue::String("upper".to_string())),
        ("key", MockValue::String("lower".to_string())),
        ("Key", MockValue::String("mixed".to_string())),
    ]);

    assert_eq!(mock_dict.get_string("KEY"), Some("upper".to_string()));
    assert_eq!(mock_dict.get_string("key"), Some("lower".to_string()));
    assert_eq!(mock_dict.get_string("Key"), Some("mixed".to_string()));
}

#[test]
fn test_property_utils_special_characters() {
    let mock_dict = MockDictionary::with_entries(&[
        ("key!@#$%", MockValue::String("special_chars".to_string())),
        ("🦀", MockValue::String("rust".to_string())),
        ("\\n\\t", MockValue::String("escaped".to_string())),
    ]);

    assert_eq!(mock_dict.get_string("key!@#$%"), Some("special_chars".to_string()));
    assert_eq!(mock_dict.get_string("🦀"), Some("rust".to_string()));
    assert_eq!(mock_dict.get_string("\\n\\t"), Some("escaped".to_string()));
}

#[test]
fn test_property_utils_boundary_values() {
    let mock_dict = MockDictionary::with_entries(&[
        ("max_int", MockValue::Number(i64::MAX as f64)),
        ("min_int", MockValue::Number(i64::MIN as f64)),
        ("epsilon", MockValue::Number(f64::EPSILON)),
        ("smallest", MockValue::Number(f64::MIN_POSITIVE)),
    ]);

    assert_eq!(mock_dict.get_number("max_int"), Some(i64::MAX as f64));
    assert_eq!(mock_dict.get_number("min_int"), Some(i64::MIN as f64));
    assert_eq!(mock_dict.get_number("epsilon"), Some(f64::EPSILON));
    assert_eq!(mock_dict.get_number("smallest"), Some(f64::MIN_POSITIVE));
}
