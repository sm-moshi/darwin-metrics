## **GENERAL Policy**

You are an expert in Rust, async programming, and concurrent systems.

**Key Principles**

- Write clear, concise, and idiomatic Rust code with accurate examples.
- Use async programming paradigms effectively, leveraging `tokio` for concurrency.
- Prioritize modularity, clean code organization, and efficient resource management.
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`).
- Adhere to Rust's naming conventions: snake_case for variables and functions, PascalCase for types and structs.
- Avoid code duplication; use functions and modules to encapsulate reusable logic.
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system.

**Async Programming**

- Use `tokio` as the async runtime for handling asynchronous tasks and I/O.
- Implement async functions using `async fn` syntax.
- Leverage `tokio::spawn` for task spawning and concurrency.
- Use `tokio::select!` for managing multiple async tasks and cancellations.
- Favor structured concurrency: prefer scoped tasks and clean cancellation paths.
- Implement timeouts, retries, and backoff strategies for robust async operations.

**Channels and Concurrency**

- Use Rust's `tokio::sync::mpsc` for asynchronous, multi-producer, single-consumer channels.
- Use `tokio::sync::broadcast` for broadcasting messages to multiple consumers.
- Implement `tokio::sync::oneshot` for one-time communication between tasks.
- Prefer bounded channels for backpressure; handle capacity limits gracefully.
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock` for shared state across tasks, avoiding deadlocks.

**Error Handling and Safety**

- Embrace Rust's Result and Option types for error handling.
- Use `?` operator to propagate errors in async functions.
- Implement custom error types using `thiserror` or `anyhow` for more descriptive errors.
- Handle errors and edge cases early, returning errors where appropriate.
- Use `.await` responsibly, ensuring safe points for context switching.

**Testing**

- Write unit tests with `tokio::test` for async tests.
- Use `tokio::time::pause` for testing time-dependent code without real delays.
- Implement integration tests to validate async behavior and concurrency.
- Use mocks and fakes for external dependencies in tests.

**Performance Optimization**

- Minimize async overhead; use sync code where async is not needed.
- Avoid blocking operations inside async functions; offload to dedicated blocking threads if necessary.
- Use `tokio::task::yield_now` to yield control in cooperative multitasking scenarios.
- Optimize data structures and algorithms for async use, reducing contention and lock duration.
- Use `tokio::time::sleep` and `tokio::time::interval` for efficient time-based operations.

**Key Conventions**

1. Structure the application into modules: separate concerns like networking, database, and business logic.
2. Use environment variables for configuration management (e.g., `dotenv` crate).
3. Ensure code is well-documented with inline comments and Rustdoc.

**Async Ecosystem**

- Use `tokio` for async runtime and task management.
- Leverage `hyper` or `reqwest` for async HTTP requests.
- Use `serde` for serialization/deserialization.
- Use `sqlx` or `tokio-postgres` for async database interactions.
- Utilize `tonic` for gRPC with async support.

Refer to Rust's async book and `tokio` documentation for in-depth information on async patterns, best practices, and advanced features.

## !! \*IMPORTANT CODE RULES\*\*

- ✅ Keep a **clean and simple** structure.
- ✅ Code must be **readable and maintainable** - refactor when necessary.
- ✅ Code must be **testable**.
- ✅ Code must be **easily extendable**.
- ✅ Code must be **easily reusable**.
- ✅ Consider **performance and reliability**.
- ✅ **KISS** (Keep it simple, stupid) - Avoid unnecessary complexity.
- ✅ **DRY** (Don't Repeat Yourself) - Avoid duplication, modularize code.
- ✅ **YAGNI** (You Aren't Gonna Need it) - Don't build features you don't need now.
- ✅ **SPOT** (Single Point of Truth) - Keep important values in one place.
- **Consider multiple solutions but stay consistent.**

## Documentation

1. Always document your code in markdown files, stored in a `/docs` folder:

   - `/docs/PROJECT.md` → High-level project structure and organization.
   - `/docs/TODO.md` → Task tracking.
   - `/docs/CHANGELOG.md` → Changelog.

   - All major changes to the program require a documentation update.

## **Testing Policy**

- All new features **must include unit tests**.
- Integration tests are required for **FFI or system API interactions**.
- Swift tests should use **Quick/Nimble**.
- All tests must pass before merging code.

## **Git Workflow**

    - Always create feature branches (`feature/xyz`) before merging into `main`.
    - Follow commit message format:

        ```bash
        [Category]: Brief Summary

        - Change 1
        - Change 2

        **Example**:

        [Feature]: Add CPU Temperature Monitoring

        - Implemented SMC temperature reading
        - Added FFI bridge for Rust integration
        ```

    - Push only after running all tests.
    - Merge via **pull requests (PRs)** after code review.
    - After implementing a major feature, write a commit description and update the **changelog**.
