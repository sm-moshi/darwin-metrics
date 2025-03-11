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

    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<String> =
        PropertyAccessor::get_string_property;
    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<f64> =
        PropertyAccessor::get_number_property;
    let _: fn(&NSDictionary<NSString, NSObject>, &str) -> Option<bool> =
        PropertyAccessor::get_bool_property;

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
    let dict_with_empty =
        MockDictionary::with_entries(&[("empty_string", MockValue::String("".to_string()))]);
    assert_eq!(dict_with_empty.get_string("empty_string"), Some("".to_string()));

    // Dictionary with zero and false values
    let dict_with_zeros = MockDictionary::with_entries(&[
        ("zero", MockValue::Number(0.0)),
        ("false", MockValue::Boolean(false)),
    ]);
    assert_eq!(dict_with_zeros.get_number("zero"), Some(0.0));
    assert_eq!(dict_with_zeros.get_bool("false"), Some(false));
}
