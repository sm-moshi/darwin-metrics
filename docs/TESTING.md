# Testing Approach for darwin-metrics

## Overview

Testing hardware-dependent code like `darwin-metrics` presents unique challenges:

1. Unit tests shouldn't depend on real hardware
2. Tests need to be reproducible across different machines
3. FFI and system calls can cause segmentation faults if not handled properly
4. Tests should run in CI environments without real macOS hardware

## Testing Strategy

### 1. Mock-based Unit Testing

We use `mockall` to create mock implementations of our system interface traits:

```rust
#[cfg_attr(test, automock)]
pub trait IOKit: Send + Sync + std::fmt::Debug {
    // Interface methods...
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[test]
    fn test_with_mock() {
        let mut mock = MockIOKit::new();
        
        mock.expect_some_method()
            .returning(|_| Ok(()));
            
        // Test logic using mock
    }
}
```

### 2. Safe Test Utilities

The `test_utils` module provides safe helper functions to avoid direct hardware access:

```rust
#[cfg(test)]
pub fn create_test_battery() -> Battery {
    Battery::with_values(
        true,               // is_present
        false,              // is_charging
        75.5,               // percentage
        90,                 // time_remaining
        PowerSource::Battery,
        500,                // cycle_count 
        85.0,               // health_percentage
        35.0,               // temperature
    )
}
```

### 3. Conditional Compilation for Testing

Field visibility is controlled using conditional compilation:

```rust
pub struct CPU {
    #[cfg(not(test))]
    iokit: Box<dyn IOKit>,
    #[cfg(test)]
    pub iokit: Box<dyn IOKit>,
}
```

### 4. Integration Tests

Integration tests use the public API only and don't rely on internal implementation details:

```rust
#[test]
fn test_battery_with_values() {
    let battery = darwin_metrics::battery::Battery::with_values(
        true, false, 75.5, 90,
        PowerSource::Battery, 500, 85.0, 35.0
    );
    
    assert_eq!(battery.is_present, true);
    assert_eq!(battery.percentage, 75.5);
}
```

## Process Metrics

When testing process metrics, note that:

- CPU usage is normalized between 0-100%
- Permission errors are handled gracefully
- Tests cover both accessible and privileged processes

## Test Organization

- **Unit Tests**: Located alongside the code they test (`mod tests { ... }`)
- **Integration Tests**: Located in the `tests/` directory
- **Test Utilities**: Located in `src/test_utils.rs`

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo llvm-cov --lcov --output-path coverage/lcov.info

# Run specific tests
cargo test battery
```

## Test Debugging

When tests fail with segmentation faults, you can use:

```bash
RUST_BACKTRACE=1 cargo test -- --nocapture
```

The `setup_test_environment()` function in test_utils helps improve diagnostics by setting up panic handlers that provide more context.
