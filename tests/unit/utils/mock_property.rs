use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::pin::Pin;
use std::future::Future;

use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSMutableDictionary, NSNumber, NSString};

use crate::utils::core::property::{KeyWrapper, PropertyUtils};
use crate::error::Result;

/// Test value types that can be stored in the mock property accessor
#[derive(Clone, Debug)]
pub enum TestValue {
    String(String),
    Number(f64),
    Bool(bool),
}

/// Mock implementation of PropertyAccessor for testing
pub struct MockPropertyAccessor {
    dict: Arc<Mutex<NSMutableDictionary>>,
    test_values: Arc<Mutex<HashMap<String, TestValue>>>,
}

impl MockPropertyAccessor {
    /// Create a new mock property accessor
    pub fn new() -> Self {
        Self {
            dict: Arc::new(Mutex::new(NSMutableDictionary::new())),
            test_values: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set a test value for a given key
    pub fn set_test_value(&self, key: &str, value: TestValue) {
        self.test_values.lock().unwrap().insert(key.to_string(), value);
    }

    /// Get the underlying test value for a key
    pub fn get_test_value(&self, key: &str) -> Option<TestValue> {
        self.test_values.lock().unwrap().get(key).cloned()
    }
}

#[async_trait::async_trait]
impl PropertyUtils for MockPropertyAccessor {
    async fn get_string_property(&self, key: &KeyWrapper) -> Result<Option<String>> {
        Ok(self.test_values.lock().unwrap()
            .get(key.as_nsstring().to_string().as_str())
            .and_then(|v| match v {
                TestValue::String(s) => Some(s.clone()),
                _ => None,
            }))
    }

    async fn get_number_property(&self, key: &KeyWrapper) -> Result<Option<f64>> {
        Ok(self.test_values.lock().unwrap()
            .get(key.as_nsstring().to_string().as_str())
            .and_then(|v| match v {
                TestValue::Number(n) => Some(*n),
                _ => None,
            }))
    }

    async fn set_bool(&mut self, key: &KeyWrapper, value: bool) -> Result<()> {
        self.set_test_value(key.as_nsstring().to_string().as_str(), TestValue::Bool(value));
        Ok(())
    }

    async fn set_i64(&mut self, key: &KeyWrapper, value: i64) -> Result<()> {
        self.set_test_value(key.as_nsstring().to_string().as_str(), TestValue::Number(value as f64));
        Ok(())
    }

    async fn set_f64(&mut self, key: &KeyWrapper, value: f64) -> Result<()> {
        self.set_test_value(key.as_nsstring().to_string().as_str(), TestValue::Number(value));
        Ok(())
    }
}

impl Default for MockPropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
} 