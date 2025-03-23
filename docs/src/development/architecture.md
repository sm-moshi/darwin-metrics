# Internal Architecture

This document explains the internal architecture and organization of the `darwin-metrics` codebase.

## Project Structure

The darwin-metrics library follows a domain-driven design approach, organizing code around system resources:

```
darwin-metrics/
├── Cargo.lock
├── Cargo.toml                     # Crate configuration
├── LICENSE
├── NOTICE
├── README.md                      # Project overview
├── build.rs                       # Native code build script
├── changelog-configuration.json   # Release configuration
├── clippy.toml                    # Linting configuration
├── coverage/                      # Code coverage reports
├── docs/                          # Documentation
│   ├── CHANGELOG.md               # Release history
│   ├── CHECKLIST.md               # Release checklist
│   ├── ROADMAP.md                 # Development roadmap
│   ├── RUST_API_CHECKLIST.md      # API design guidelines
│   ├── TODO.md                    # Development tasks
│   ├── book.toml                  # mdBook configuration
│   ├── book/                      # Generated documentation
│   ├── custom.css                 # Documentation styling
│   └── src/                       # Documentation source
│       ├── SUMMARY.md             # Documentation index
│       ├── advanced/              # Advanced topics
│       ├── development/           # Developer guides
│       ├── getting-started.md     # Quickstart guide
│       ├── introduction.md        # Project introduction
│       └── modules/               # Module documentation
├── examples/                      # Example applications
│   ├── disk_monitor.rs            # Disk monitoring example
│   ├── gpu_monitor_safe.rs        # Safe GPU monitoring
│   ├── gpu_monitor_simplified.rs  # Simplified GPU example
│   ├── gpu_static.rs              # Static GPU info example
│   ├── memory_monitor.rs          # Memory monitoring
│   ├── memory_monitor_async.rs    # Async memory monitoring
│   ├── network_async.rs           # Async network monitoring
│   └── network_info.rs            # Network info example
├── src/                           # Main source code
│   ├── battery/                   # Battery monitoring
│   │   └── mod.rs                 # Battery module implementation
│   ├── disk/                      # Disk monitoring
│   │   └── mod.rs                 # Disk module implementation
│   ├── docs_rs_stubs.rs           # Support for docs.rs
│   ├── error.rs                   # Error handling
│   ├── hardware/                  # Hardware-related modules
│   │   ├── cpu/                   # CPU metrics
│   │   │   ├── cpu_impl.rs        # CPU implementation
│   │   │   ├── frequency.rs       # CPU frequency tracking
│   │   │   └── mod.rs             # CPU module definition
│   │   ├── gpu/                   # GPU metrics
│   │   │   └── mod.rs             # GPU implementation
│   │   ├── iokit/                 # IOKit interface
│   │   │   ├── mock.rs            # Mock implementation for testing
│   │   │   ├── mod.rs             # Main implementation
│   │   │   └── tests.rs           # Tests for IOKit
│   │   ├── memory/                # Memory metrics
│   │   │   └── mod.rs             # Memory implementation
│   │   ├── mod.rs                 # Hardware module exports
│   │   └── temperature/           # Temperature sensors
│   │       └── mod.rs             # Temperature implementation
│   ├── lib.rs                     # Library entry point
│   ├── network/                   # Network monitoring
│   │   ├── interface.rs           # Network interfaces
│   │   ├── mod.rs                 # Network module exports
│   │   └── traffic.rs             # Network traffic
│   ├── power/                     # Power management
│   │   └── mod.rs                 # Power implementation
│   ├── process/                   # Process monitoring
│   │   └── mod.rs                 # Process implementation
│   ├── resource/                  # Resource monitoring
│   │   └── mod.rs                 # Resource implementation
│   ├── system/                    # System information
│   │   └── mod.rs                 # System implementation
│   └── utils/                     # Utility functions
│       ├── bindings.rs            # FFI bindings
│       ├── mod.rs                 # Utilities exports
│       ├── property_utils.rs      # Property access utilities
│       ├── property_utils_tests.rs # Tests for property utils
│       └── test_utils.rs          # Testing utilities
└── tests/                         # Integration tests
    └── version-sync.rs            # Version consistency tests
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
