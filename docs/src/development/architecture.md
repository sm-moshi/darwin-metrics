# Internal Architecture

This document explains the internal architecture and organization of the `darwin-metrics` codebase.

## Project Structure

The darwin-metrics library follows a domain-driven design approach, organizing code around system resources:

```text
src/
  battery/           # Battery monitoring
  disk/              # Storage monitoring
  docs_rs_stubs.rs   # Documentation stubs for docs.rs
  error.rs           # Centralized error handling
  hardware/          # Hardware-related metrics
    cpu/             # CPU monitoring (usage, frequency)
    gpu/             # GPU monitoring (usage, memory)
    iokit/           # IOKit bindings and abstractions
    memory/          # RAM and swap monitoring
    mod.rs           # Hardware module exports
    temperature/     # Temperature sensors
  lib.rs             # Public API and re-exports
  network/           # Network interface monitoring
    interface.rs     # Network interface information
    mod.rs           # Network module exports
    traffic.rs       # Network traffic monitoring
  power/             # Power management
  process/           # Process monitoring
  resource/          # Resource management
  system/            # System-wide information
  utils/             # Shared utilities
    bindings.rs      # Centralized FFI bindings
    mod.rs           # Utilities exports
    property_utils.rs # Property list utilities
    test_utils.rs    # Testing helpers
```

## Core Design Principles

### 1. Module Independence

Each module is designed to function independently while relying on shared utilities. This allows:

- Using only the features you need
- Minimizing dependency chain issues
- Isolated testing without the entire library

### 2. Layered Architecture

The codebase follows a layered approach:

```text
+-----------------------------------------+
|         Public API (lib.rs)             |
+-----------------------------------------+
|       Domain-specific Modules           |
|    (cpu, memory, process, etc.)         |
+-----------------------------------------+
|          Shared Utilities               |
|    (error handling, FFI helpers)        |
+-----------------------------------------+
|             FFI Layer                   |
|     (bindings.rs, unsafe code)          |
+-----------------------------------------+
```

### 3. FFI Strategy

Foreign Function Interface (FFI) calls to macOS APIs are centralized in `src/utils/bindings.rs`. This design:

- Contains unsafe code in a single location
- Promotes code reuse across modules
- Makes the codebase easier to audit
- Simplifies maintenance of platform-specific code

## Error Handling

The library uses a consistent error handling approach:

1. A centralized `Error` enum in `src/error.rs`
2. Module-specific error variants
3. The `Result<T, Error>` type alias for function returns
4. Use of `thiserror` for error derivation

Example:

```rust,no_run,ignore
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("System error: {0}")]
    System(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Resource not available: {0}")]
    NotAvailable(String),

    // ... other error variants
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Testing Strategy

The codebase uses multiple testing approaches:

1. **Unit Tests**: Focused on isolated functionality
2. **Integration Tests**: Testing module interactions
3. **Mock Objects**: For simulating hardware components
4. **Conditional Testing**: Platform-specific test cases

Test utilities in `src/utils/test_utils.rs` provide common testing functionality.

## Platform Compatibility

While the library targets macOS specifically:

- Platform checks ensure proper behavior
- API stability is maintained across macOS versions
- Version-specific code is isolated when needed
- Fallback mechanisms handle API differences
