# GitHub Copilot Instructions

These instructions consolidate the following three rulesets into a single, concise document, eliminating overlapping or redundant points:

1. **Comprehensive Development Guidelines**
2. **Rust Async Development Ruleset**
3. **Integrated Language Development Policy (C/ObjC/Swift)**

Copilot should follow these directives to maintain consistency, safety, and best practices across all relevant languages and frameworks for this macOS system metrics library.

---

## 1. Overall Code Quality & Fundamental Principles

- **SOLID Development Approach**
- **KISS (Keep It Simple, Stupid)**
  - Avoid unnecessary complexity
  - Solve problems with minimal, clear solutions
- **DRY (Don't Repeat Yourself)**
  - Modularize code
  - Eliminate duplication
- **YAGNI (You Aren't Gonna Need It)**
  - Implement only what's necessary right now

### Code Characteristics

- Prioritize readability, maintainability, and testability
- Ensure code is extendable, reusable, and performance-oriented
- Follow language-appropriate naming conventions:
  - `snake_case` for variables/functions in Rust
  - `PascalCase` for types/structs in Rust
  - `SCREAMING_CASE` for constants
- Organize code into logical, focused modules

### Documentation

- Maintain a `/docs` directory with:
  - `PROJECT.md`: Project overview and architecture
  - `TODO.md`: Task tracking
  - `CHANGELOG.md`: Detailed change log
  - `ARCHITECTURE.md`: System design details
- Keep mdBook documentation in `/docs/src/` up to date
- Write comprehensive Rustdoc comments with examples
- Document public APIs thoroughly

---

## 2. Testing Approach

- **Unit Testing**
  - Mandatory for all new features
  - Target 100% coverage for core logic
  - Use Rust's native testing framework
- **Integration Testing**
  - Required for FFI interactions and system API integrations
  - Test complex component interactions
- Use `tokio::test` for async testing
- Utilize `tokio::time::pause` for deterministic time-based tests
- Create mock implementations for external dependencies
- Design tests to cover edge cases and error paths

---

## 3. Git / Workflow

- Write descriptive commit messages:

```markdown
[Category]: Brief, descriptive summary

- Detailed change description
- Additional context if needed
```

- Maintain a clear, informative commit history
- Update changelog for significant features or fixes
- Use branch protection rules and CI checks to enforce code quality

---

## 4. Rust Async Guidelines

### Async Runtime & Concurrency

- Use `tokio` as the primary async runtime
- Implement async functions with `async fn` syntax
- Leverage structured concurrency patterns
- Use `tokio::spawn` for concurrent task execution
- Implement `tokio::select!` for task coordination
- Prefer scoped tasks with clear cancellation paths

### Channels & Synchronization

- Use `tokio::sync::mpsc` for multi-producer, single-consumer communication
- Use `tokio::sync::broadcast` for message broadcasting
- Use `tokio::sync::oneshot` for single-use communication
- Use bounded channels with graceful backpressure
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock`
- Minimize lock contention

### Error Handling

- Leverage `Result` and `Option` types
- Use `?` operator for error propagation
- Implement custom error types with:
  - `thiserror` for library errors
- Handle errors early and explicitly
- Provide meaningful error context

### Performance & Optimization

- Minimize async runtime overhead
- Use synchronous code when async is unnecessary
- Offload blocking operations to dedicated threads
- Utilize `tokio::task::yield_now` for cooperative multitasking
- Optimize data structures for async environments

---

## 5. Integrated Language Development (C, Objective-C, Swift)

### Memory Safety & Ownership

- Prefer automatic memory management (ARC in ObjC/Swift)
- Use `__attribute__((cleanup))` for C-based RAII patterns
- Validate all pointer operations and buffer sizes
- Minimize manual memory management
- Implement safe pointer handling with careful validation

### macOS-Specific Integration

- Code is macOS-specific using IOKit and Foundation frameworks
- For IOKit Implementation:
  - Implement proper bridging for Objective-C/Swift method calls
  - Use safe memory management practices with ARC
  - Leverage Grand Central Dispatch (GCD) for concurrency

### Concurrency & Thread Safety

- Use Grand Central Dispatch (GCD) for task scheduling
- Use `dispatch_queue_t` for concurrent operations
- Implement synchronization with:
  - `@synchronized`
  - `os_unfair_lock`
  - Atomic properties in Objective-C
- Design thread-safe interfaces
- Handle task cancellation and cleanup systematically

### Error Handling & Diagnostics

- Use `NSError` in Objective-C for error propagation
- Implement Swift's `try/catch` and `Result` type
- Use structured logging with `os_log`
- Provide meaningful error context
- Handle all error cases explicitly

### FFI Bindings

- All FFI bindings are centralized in `src/utils/bindings.rs`
- When adding new bindings:
  1. Update bindings.rs with proper documentation
  2. Group related bindings with section markers
  3. Create helper methods when appropriate
  4. Follow LLVM style guide for C/Objective-C code
- Test integration paths comprehensively

### Data and Buffer Handling

- Prefer `NSData`/`NSMutableData` over raw buffers
- Validate memory allocations and sizes
- Implement proper byte order handling
- Use safe string operations (`snprintf`, `strlcpy`)
- Validate input boundaries

### Security Practices

- Avoid unsafe pointer casts
- Validate all string and buffer operations
- Use secure random number generation
- Implement thorough input validation
- Follow platform security guidelines

---

## Final Note

- Treat these rules as living documents; evolve them alongside new best practices
- Ensure generated suggestions adhere to these guidelines across all languages
- Favor clarity, safety, and maintainability in every proposed solution
- Regularly review and update integration strategies
- Stay current with platform and Rust ecosystem developments