use std::sync::Arc;
use objc2_foundation::NSString;
use crate::utils::core::property::{KeyWrapper, PropertyUtils};
use super::mock_property::{MockPropertyAccessor, TestValue};

/// Test helper to create a KeyWrapper with a string
fn create_test_key(s: &str) -> KeyWrapper {
    KeyWrapper::new(s)
}

#[test]
fn test_key_wrapper_creation() {
    let key = create_test_key("test_key");
    assert_eq!(key.as_nsstring().to_string(), "test_key");
}

#[test]
fn test_key_wrapper_as_nsstring() {
    let key = create_test_key("test_key");
    let ns_string = key.as_nsstring();
    assert_eq!(ns_string.to_string(), "test_key");
}

#[test]
fn test_key_wrapper_as_copying() {
    let key = create_test_key("test_key");
    let _copying = key.as_copying();
    // If we get here without panicking, the conversion worked
}

#[tokio::test]
async fn test_get_string_property() {
    let accessor = MockPropertyAccessor::new();
    let key = create_test_key("string_key");
    let test_value = "test_value".to_string();
    
    accessor.set_test_value("string_key", TestValue::String(test_value.clone()));
    
    let result = accessor.get_string_property(&key).await.unwrap();
    assert_eq!(result, Some(test_value));
}

#[tokio::test]
async fn test_get_number_property() {
    let accessor = MockPropertyAccessor::new();
    let key = create_test_key("number_key");
    let test_value = 42.5;
    
    accessor.set_test_value("number_key", TestValue::Number(test_value));
    
    let result = accessor.get_number_property(&key).await.unwrap();
    assert_eq!(result, Some(test_value));
}

#[tokio::test]
async fn test_set_bool_property() {
    let mut accessor = MockPropertyAccessor::new();
    let key = create_test_key("bool_key");
    
    accessor.set_bool(&key, true).await.unwrap();
    
    match accessor.get_test_value("bool_key") {
        Some(TestValue::Bool(value)) => assert!(value),
        _ => panic!("Expected bool value"),
    }
}

#[tokio::test]
async fn test_set_i64_property() {
    let mut accessor = MockPropertyAccessor::new();
    let key = create_test_key("i64_key");
    let test_value = 42_i64;
    
    accessor.set_i64(&key, test_value).await.unwrap();
    
    match accessor.get_test_value("i64_key") {
        Some(TestValue::Number(value)) => assert_eq!(value, test_value as f64),
        _ => panic!("Expected number value"),
    }
}

#[tokio::test]
async fn test_set_f64_property() {
    let mut accessor = MockPropertyAccessor::new();
    let key = create_test_key("f64_key");
    let test_value = 42.5_f64;
    
    accessor.set_f64(&key, test_value).await.unwrap();
    
    match accessor.get_test_value("f64_key") {
        Some(TestValue::Number(value)) => assert_eq!(value, test_value),
        _ => panic!("Expected number value"),
    }
}

#[tokio::test]
async fn test_property_type_mismatch() {
    let accessor = MockPropertyAccessor::new();
    let key = create_test_key("key");
    
    // Set as string
    accessor.set_test_value("key", TestValue::String("not_a_number".to_string()));
    
    // Try to get as number
    let result = accessor.get_number_property(&key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_nonexistent_property() {
    let accessor = MockPropertyAccessor::new();
    let key = create_test_key("nonexistent");
    
    assert_eq!(accessor.get_string_property(&key).await.unwrap(), None);
    assert_eq!(accessor.get_number_property(&key).await.unwrap(), None);
} 