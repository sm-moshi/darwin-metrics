# Error Module

The Error module provides a comprehensive error handling system for the darwin-metrics crate. It
defines a central `Error` type and `Result` type alias that are used throughout the crate for
consistent error handling.

## Error Types

The main error enum includes various error types specific to different monitoring domains:

```rust
pub enum Error {
    // IO-related errors
    Io { kind: io::ErrorKind, message: String },
    
    // Hardware monitoring errors
    IOKit(String),
    Temperature(String),
    Cpu(String),
    Gpu(String),
    Memory(String),
    
    // System monitoring errors
    Network(String),
    Process(String),
    SystemInfo(String),
    
    // Service and resource errors
    ServiceNotFound(String),
    InvalidData(String),
    NotImplemented(String),
    NotAvailable(String),
    PermissionDenied(String),
    
    // Generic errors
    System(String),
    Other(String),
}
```

## Error Creation Methods

The module provides convenient methods for creating specific error types:

```rust
impl Error {
    // IOKit errors
    pub fn io_kit(message: impl Into<String>) -> Self
    
    // System errors
    pub fn system(message: impl Into<String>) -> Self
    
    // Data validation errors
    pub fn invalid_data(message: impl Into<String>) -> Self
    
    // Feature availability errors
    pub fn not_implemented(feature: impl Into<String>) -> Self
    pub fn not_available(feature: impl Into<String>) -> Self
    
    // Permission errors
    pub fn permission_denied(message: impl Into<String>) -> Self
    
    // Service errors
    pub fn service_not_found(service: impl Into<String>) -> Self
    
    // Process errors
    pub fn process_error(message: impl Into<String>) -> Self
}
```

## Error Details and Classification

The Error type provides methods to get detailed error information and classify errors:

```rust
impl Error {
    // Get detailed error information
    pub fn details(&self) -> String
    
    // Check error types
    pub fn is_permission_error(&self) -> bool
    pub fn is_not_available(&self) -> bool
}
```

## Result Type

The module defines a convenient Result type alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Usage Examples

### Basic Error Handling

```rust
use darwin_metrics::{Error, Result};

fn example() -> Result<()> {
    // Return a permission error
    if !has_permissions() {
        return Err(Error::permission_denied("Insufficient permissions"));
    }
    
    // Return a service error
    if service_not_found() {
        return Err(Error::service_not_found("Required service unavailable"));
    }
    
    Ok(())
}
```

### Error Details and Classification

```rust
use darwin_metrics::Error;

fn handle_error(error: Error) {
    // Get detailed error information
    println!("Error details: {}", error.details());
    
    // Check error type
    if error.is_permission_error() {
        println!("This is a permission error");
    }
    
    if error.is_not_available() {
        println!("This feature is not available");
    }
}
```

### Converting from IO Errors

The Error type implements From<io::Error> for automatic conversion:

```rust
use std::fs::File;
use darwin_metrics::Result;

fn read_file(path: &str) -> Result<String> {
    let mut file = File::open(path)?;  // io::Error automatically converts to Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // io::Error automatically converts to Error
    Ok(contents)
}
```

## Error Messages

Error messages are designed to be:

- Clear and descriptive
- Action-oriented when possible
- Informative about the root cause
- Helpful for troubleshooting

For example:

- Permission errors include a hint about running with elevated privileges
- IOKit errors mention potential compatibility issues
- Feature availability errors explain hardware support requirements

## Thread Safety

The Error type implements Send and Sync, making it safe to use across thread boundaries. This is
particularly important for async operations and multi-threaded monitoring scenarios.
