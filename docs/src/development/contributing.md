# Contributing to darwin-metrics

Thank you for your interest in contributing to `darwin-metrics`! This guide will help you get started with the development process.

## Getting Started

### Prerequisites

- Rust 1.85 or later
- macOS 14.x or later
- Xcode Command Line Tools
- Git

### Setup

1. Fork the repository on GitHub
2. Clone your fork locally:

   ```bash
   git clone https://github.com/your-username/darwin-metrics.git
   cd darwin-metrics
   ```

3. Add the original repository as an upstream remote:

   ```bash
   git remote add upstream https://github.com/sm-moshi/darwin-metrics.git
   ```

## Development Workflow

### Building the Project

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features
```

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run a specific test
cargo test <test_name> -- --nocapture

# Run faster tests using nextest
cargo nextest run
```

### Code Coverage

```bash
# Generate coverage report
cargo llvm-cov
```

### Code Quality

Before submitting a pull request, run these checks:

```bash
# Format code
cargo +beta fmt

# Run linter
cargo +beta clippy --workspace --tests --all-targets --all-features
```

## Adding New Features

### Creating a New Module

1. Create a new directory in `src/` for your module (if applicable)
2. Create a `mod.rs` file within that directory
3. Add the module to `src/lib.rs` with a public export

Example of a new module:

```rust,no_run,ignore
// src/mynewmodule/mod.rs
// First import the error types from the main crate
use darwin_metrics::error::{Error, Result};

pub struct MyNewFeature {
    // Implementation details
    name: String,
    value: f64,
}

impl MyNewFeature {
    pub fn new() -> Result<Self> {
        // Initialize your feature
        Ok(Self {
            name: "My Feature".to_string(),
            value: 0.0,
        })
    }

    pub fn get_metrics(&self) -> Result<String> {
        // Implement your metrics collection logic
        Ok(format!("{}: {}", self.name, self.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Test implementation
        let feature = MyNewFeature::new().unwrap();
        assert!(feature.get_metrics().is_ok());
    }
}
```

### FFI Bindings

If your module requires FFI bindings:

1. Add your bindings to `src/utils/bindings.rs`
2. Group related bindings together
3. Add proper documentation
4. Provide helper functions for complex operations

## Pull Request Process

1. Create a new branch for your feature or fix:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit them with a clear message:

   ```bash
   git commit -m "feat: add new feature X"
   ```

3. Push your branch to your fork:

   ```bash
   git push origin feature/your-feature-name
   ```

4. Create a pull request on GitHub
5. Ensure your PR includes:
   - A clear description of the changes
   - Any relevant issue numbers
   - Documentation updates
   - Tests for new functionality

## Commit Style

We follow a simplified version of the Conventional Commits specification:

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding or fixing tests
- `perf`: Performance improvements
- `chore`: Other changes

Example: `feat: add CPU frequency monitoring`

## Code Style

We adhere to standard Rust style guidelines:

- Use `snake_case` for variables and functions
- Use `CamelCase` for types and structs
- Use `SCREAMING_CASE` for constants
- Use the `?` operator for error propagation
- Document public APIs with Rustdoc comments
- Write clear, concise comments for complex logic

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT license.

## Questions?

If you have any questions or need help with your contribution, please open an issue on GitHub.
