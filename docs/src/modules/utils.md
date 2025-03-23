# Utils Module

The Utils module provides essential utility functions and submodules for the darwin-metrics crate,
focusing on safe FFI bindings, property list handling, and testing utilities.

## Core Components

### Bindings (`bindings`)

FFI bindings for macOS system APIs including:

- `sysctl` for system information
- `IOKit` for hardware access
- Mach host functions for memory statistics
- Network interface monitoring
- Process and filesystem operations

### Property Utils (`property_utils`)

Utilities for working with property lists and dictionaries:

```rust
pub trait PropertyUtils {
    fn get_string_property(...) -> Option<String>;
    fn get_number_property(...) -> Option<f64>;
    fn get_bool_property(...) -> Option<bool>;
}
```

### Dictionary Access (`dictionary_access`)

Abstract interface for dictionary operations:

```rust
pub trait DictionaryAccess {
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_number(&self, key: &str) -> Option<f64>;
    fn get_bool(&self, key: &str) -> Option<bool>;
}
```

### Mock Dictionary (`mock_dictionary`)

Pure Rust mock dictionary implementation for testing.

### Test Utils (`test_utils`)

Testing utilities including:

- Dictionary creation helpers
- Test data generators
- Type conversion utilities

## Safe FFI Functions

### String Handling

```rust
// Convert C string to Rust String
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String>

// Convert raw string to Rust String
pub unsafe fn raw_str_to_string(ptr: *const c_char, len: usize) -> Option<String>
```

### Numeric Conversions

```rust
// Convert raw f64 slice to Vec
pub unsafe fn raw_f64_slice_to_vec(ptr: *const c_double, len: usize) -> Option<Vec<f64>>
```

### Objective-C Integration

```rust
// Execute code in autorelease pool
pub fn autorelease_pool<T, F>(f: F) -> T
where
    F: FnOnce() -> T

// Safe execution of Objective-C code
pub fn objc_safe_exec<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>
```

## System Constants

### Process States

```rust
pub mod proc_state {
    pub const SIDL: u8 = 1;   // Process being created
    pub const SRUN: u8 = 2;   // Running
    pub const SSLEEP: u8 = 3; // Sleeping
    pub const SSTOP: u8 = 4;  // Debugging/suspended
    pub const SZOMB: u8 = 5;  // Zombie
}
```

### Network Constants

```rust
pub mod address_family {
    pub const AF_UNSPEC: u8 = 0;  // Unspecified
    pub const AF_INET: u8 = 2;    // IPv4
    pub const AF_INET6: u8 = 30;  // IPv6
    pub const AF_LINK: u8 = 18;   // Link level
}
```

## Thread Safety

All utility functions and types are designed with thread safety in mind:

- Safe concurrent access to dictionaries
- Thread-safe property access
- Proper handling of Objective-C autorelease pools
- Safe FFI boundary management

## Error Handling

The module uses the crate's error handling system:

- Returns `Result<T>` for operations that might fail
- Proper cleanup of system resources
- Safe handling of null pointers and invalid data
- Comprehensive error messages

## Performance Considerations

- Minimized memory allocations
- Efficient string conversions
- Zero-copy when possible
- Proper resource cleanup
- Cached values where appropriate

## Testing Support

### Mock Objects

```rust
// Create a mock dictionary for testing
let mock_dict = MockDictionary::with_entries(&[
    ("string", MockValue::String("test".to_string())),
    ("number", MockValue::Number(42.0)),
    ("boolean", MockValue::Boolean(true))
]);
```

### Test Utilities

```rust
// Create test objects
pub fn create_test_dictionary() -> NSDictionary<NSString, NSObject>;
pub fn create_test_string(content: &str) -> Retained<NSString>;
pub fn create_test_number(value: i64) -> Retained<NSNumber>;
```

## Best Practices

1. **FFI Safety**
   - Always use safe wrappers around unsafe FFI calls
   - Validate pointers before dereferencing
   - Handle null pointers gracefully
   - Clean up resources properly

2. **Error Handling**
   - Use Result for fallible operations
   - Provide detailed error messages
   - Clean up resources on error
   - Handle edge cases explicitly

3. **Testing**
   - Use mock objects for unit tests
   - Test edge cases and error conditions
   - Verify thread safety
   - Test with various data types and sizes

4. **Performance**
   - Minimize memory allocations
   - Use efficient data structures
   - Cache frequently accessed values
   - Clean up resources promptly

## Examples

### Using Property Utils

```rust
use darwin_metrics::utils::PropertyUtils;

// Get properties from a dictionary
let string_val = dict.get_string_property("key");
let number_val = dict.get_number_property("key");
let bool_val = dict.get_bool_property("key");
```

### Safe FFI Operations

```rust
use darwin_metrics::utils::{c_str_to_string, autorelease_pool};

// Convert C string safely
unsafe {
    if let Some(string) = c_str_to_string(ptr) {
        println!("String: {}", string);
    }
}

// Use autorelease pool
autorelease_pool(|| {
    // Objective-C operations here
});
```

### Dictionary Access

```rust
use darwin_metrics::utils::DictionaryAccess;

// Access dictionary values safely
if let Some(value) = dictionary.get_string("key") {
    println!("Value: {}", value);
}
```
