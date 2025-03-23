# darwin-metrics - a macOS System Metrics Crate - Development Roadmap

---

## **HIGH PRIORITY: Pre-Release Checklist for 0.1.0**

> **IMPORTANT**: These items MUST be completed before releasing to crates.io

- [x] **Version Consistency**

  - [x] Update version in Cargo.toml from "0.0.1" to "0.1.0" to match README
  - [x] Ensure consistent versioning across all documentation

- [x] **Feature Completion**

  - [x] Complete the Disk module (marked as "Complete" in CLAUDE.md)
  - [x] Fix memory management issues in GPU module
  - [x] Complete memory analysis module (with tests and async support)
  - [x] Address critical TODOs listed in CHANGELOG.md

- [x] **Documentation**

  - [x] Update module documentation to match actual implementations
  - [x] Add comprehensive documentation for the disk module
  - [x] Add comprehensive documentation for the memory module
  - [x] Ensure examples use the correct function signatures and have timeout limits
  - [x] Verify API examples match actual code behavior

- [x] **Testing Issues**

  - [x] Fix disabled GPU tests causing SIGSEGV crashes
  - [x] Resolve memory safety issues in IOKit interfaces
  - [x] Replace simulated data with real hardware access in tests
  - [x] Ensure all modules have basic test coverage

- [x] **CI/CD Setup**

  - [x] Create GitHub Actions workflow files
  - [x] Set up proper CI pipeline referenced in README badge
  - [x] Implement automated testing and release process

- [x] **Critical Bugs**

  - [x] Fix memory management issues with IOKit calls
  - [x] Implement real system API calls instead of simulations
  - [x] Fix remaining clippy warnings about non-snake case fields (Optional - can be addressed
    post-release)

- [x] **Final Steps**

  - [x] Update the changelog for 0.1.0 release
  - [x] Move items from "Unreleased" to the proper version
  - [x] Set up crates.io metadata and verify package information

---

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [x] Write a build script (`build.rs`) to compile and link macOS frameworks
- [x] Implement a minimal working example with IOKit bindings

---

## **Phase 1: Codebase Refactoring & Cleanup (0.1.0) - COMPLETED**

**Goal:** Improve structure, maintainability, and performance before implementing new features.

### **Code Refactoring & Cleanup**

- [x] **Process Monitoring**

  - [x] Improve modularity by separating concerns into dedicated functions and modules.
  - [x] Refactor function and variable names to follow Rust conventions (`snake_case` for functions,
            `PascalCase` for structs).
  - [x] Implement async-friendly architecture where necessary, ensuring minimal blocking operations.
  - [x] Enhance process I/O tracking by gathering read/write statistics.
  - [x] Add structured error handling for process-related failures.
  - [x] Write additional unit tests for process enumeration, resource usage, and error scenarios.
  - [x] Add parent-child process hierarchy tracking.
  - [x] Use efficient sysctl-based process retrieval for bulk operations (like Bottom).
  - [x] Add proper CPU usage calculation with history tracking.
  - [x] Add thread count and process state tracking.
  - [x] Implement graceful fallbacks for permission issues.
  - [ ] Add additional memory usage metrics (virtual memory, swapped memory) - future enhancement.
  - [ ] Add process environment variable retrieval - future enhancement.
  - [ ] Implement command-line arguments retrieval - future enhancement.

- [x] **CPU Metrics**

  - [x] Refactored CPU module (mod.rs, cpu.rs, frequency.rs).
  - [x] Implemented CPU frequency monitoring with min/max/available frequencies.
  - [x] Improved CPU temperature retrieval.
  - [x] Added comprehensive CPU metrics documentation.

- [x] **GPU Metrics**

  - [x] Implemented CoreFoundation approach for basic GPU metrics.
  - [x] Added GPU utilization and VRAM usage tracking.
  - [x] Integrated with IOKit to access GPU performance information.
  - [x] Fixed memory safety issues during test teardown (SIGSEGV) by making IOKitImpl properly
            cloneable.
  - [x] Improved cleanup of IOKit resources with autoreleasepools.
  - [x] Created real-world examples to validate the implementation.
  - [x] Enhanced tests to be more resilient in different environments.
  - [ ] Integrate with Metal API for advanced GPU monitoring (future enhancement).

- [x] **Network Monitoring**

  - [x] Implement bandwidth tracking, ensuring accurate upload/download speed calculations.
  - [x] Introduce real-time network state monitoring to detect active/inactive connections.
  - [x] Improve packet loss and error tracking by gathering statistics from system interfaces.
  - [x] Implement synchronous network data collection with efficient error handling.
  - [x] Write unit tests for network interface detection and updates.
  - [x] Add comprehensive documentation with usage examples.
  - [x] Create interface categorization (Ethernet, WiFi, Loopback, etc.).
  - [x] Implement IP/MAC address collection for each interface.
  - [x] Add error monitoring (errors, collisions) for network diagnostics.

- [x] **Temperature Monitoring**

  - [x] Add fan speed tracking by retrieving RPM values from available system sensors.
  - [x] Implement thermal zone monitoring to track system heat distribution.
  - [x] Detect and report thermal throttling by monitoring CPU/GPU clock adjustments.
  - [x] Improve efficiency of temperature polling to avoid unnecessary resource consumption.
  - [x] Add async temperature monitoring for non-blocking operation.

- [x] **General Code Cleanup**
  - [x] Refactored system module and merged architecture detection.
  - [x] Applied Rust idiomatic principles from `.windsurfrules`, ensuring consistent formatting.
  - [x] Removed redundant or unused code.
  - [x] Centralized FFI bindings in src/utils/bindings.rs for better maintainability.
  - [x] Improved error handling and ensure meaningful propagation of system errors.
  - [x] Added process-related error handler for better error context.
  - [x] Ensured existing tests pass after refactoring, updating them where necessary.
  - [x] Moved all Metal, statfs, and process-related FFI bindings to centralized location.

### GPU Implementation Issues

- [x] Improve GPU hardware detection for various Mac models
- [x] Create more comprehensive examples to validate the GPU implementation
- [x] Add support for multiple GPU configurations
- [ ] Add utilization tracking for specialized GPU components (like Media Engines)

---

## **Phase 2: Enhanced System Metrics (0.2.0) - ONGOING**

**Goal:** Expand monitoring capabilities with additional system metrics.

### **HIGH PRIORITY: Pre-Release Checklist for 0.2.0-alpha1**

> **IMPORTANT**: These items MUST be completed before releasing 0.2.0-alpha1 to crates.io

- [ ] **Code Quality**
  - [ ] Run and fix all issues from `cargo clippy --all-targets --all-features --workspace`
  - [ ] Focus on memory safety warnings in FFI code first
  - [ ] Document the reasoning behind any suppressed warnings
  - [ ] Use bindgen for generating Rust bindings with appropriate allow attributes

- [ ] **Code Cleanup**
  - [ ] Remove dead and unused code, especially in modules with recent changes
  - [ ] Clean up debug macros (println!, panic!) that shouldn't be in production code
  - [ ] Convert todo!() macros to GitHub issues or TODO.md entries
  - [ ] Review and document all unsafe blocks
  - [ ] Follow DRY principle by extracting repeated code into reusable functions

- [ ] **Documentation Enhancement**
  - [ ] Update README.md with clear usage examples for all modules
  - [ ] Document all public items with rustdoc, focusing on the 'why' not just 'what'
  - [ ] Add module-level documentation for new modules
  - [ ] Create or update CHANGELOG.md with 0.2.0-alpha1 changes
  - [ ] Review and document any remaining undocumented public APIs

- [ ] **Testing Verification**
  - [ ] Review and enable any ignored tests
  - [ ] Ensure 100% test pass rate with 0 warnings
  - [ ] Add tests for new functionality
  - [ ] Test FFI boundaries thoroughly
  - [ ] Verify test coverage for critical components
  - [ ] Prioritize memory safety and concurrency tests

- [ ] **Memory Module Testing**
  - [ ] Add comprehensive tests for memory/monitors.rs (currently 0% coverage)
  - [ ] Improve test coverage for memory/types.rs (currently 80.65% line coverage)
  - [ ] Add integration tests for memory monitoring functionality
  - [ ] Test async memory monitoring functions
  - [ ] Add stress tests for memory monitoring under load
  - [ ] Test error handling and edge cases
  - [ ] Verify memory safety in FFI code
  - [ ] Add benchmarks for memory monitoring performance

- [ ] **Memory Module Documentation**
  - [ ] Add comprehensive documentation for memory/monitors.rs
  - [ ] Document memory monitoring traits and implementations
  - [ ] Add usage examples for memory monitoring
  - [ ] Document async memory monitoring capabilities
  - [ ] Add performance considerations and best practices
  - [ ] Document error handling and recovery strategies

- [ ] **Memory Module Features**
  - [ ] Implement virtual memory monitoring
  - [ ] Add swap memory tracking
  - [ ] Implement memory pressure monitoring
  - [ ] Add memory compression statistics
  - [ ] Implement memory page fault tracking
  - [ ] Add memory bandwidth monitoring
  - [ ] Implement memory thermal monitoring
  - [ ] Add memory power consumption tracking

- [ ] **Dependency Management**
  - [ ] Review and update dependencies:
    - [ ] Update libc from 0.2.167/0.2.168 to latest
    - [ ] Review and update async-trait if needed
    - [ ] Check and update tokio dependencies
    - [ ] Ensure all dependencies have appropriate version constraints

- [ ] **Publication Steps**
  - [ ] Update version to 0.2.0-alpha1 in Cargo.toml
  - [ ] Create git tag v0.2.0-alpha1
  - [ ] Run cargo package to verify package builds correctly
  - [ ] Use cargo publish --dry-run before actual publication

### **New Features**

- [x] **Enhance CPU and GPU monitoring with async processing**

  - [x] Add async versions of Temperature monitoring functions
  - [x] Ensure non-blocking operation for temperature sensor access
  - [x] Implement proper tokio task handling for blocking operations

- [x] **GPU Performance Metrics**

  - [x] Track GPU utilization over time to monitor workload distribution.
  - [x] Measure GPU memory consumption and optimize reporting.
  - [x] Implement enhanced GPU hardware detection with Apple Silicon chip identification
  - [x] Add detailed GPU characteristics (core count, clock speed, architecture)
  - [x] Create comprehensive example program for GPU hardware information
  - [x] Add tests for GPU hardware detection (currently 64.66% coverage)
  - [ ] Investigate IOGraphicsLib.h for better GPU metrics access
            (<https://developer.apple.com/documentation/iokit/iographicslib_h> for reference)
  - [x] Add support for multiple GPUs

- [x] **Advanced Process Monitoring**

  - [x] Implement parent-child process hierarchy tracking for improved system visibility.
  - [x] Add per-process thread monitoring to gather insights on concurrency behavior.
  - [x] Improve resource usage tracking with additional data on CPU/memory consumption.

- [x] **Disk & Storage Monitoring**

  - [x] Track I/O performance by measuring read/write speeds for individual drives.
  - [x] Monitor volume usage, including available/free space calculations.
  - [x] Implement tracking for mounted disk devices and storage partitions.
  - [x] Fix linter error in disk/monitors.rs related to trace macro
  - [~] Improve test coverage (currently 59.29%)

- [~] **Network Enhancements**

  - [x] Implement packet loss and error tracking to improve network diagnostics.
  - [x] Add connection state tracking to detect when network interfaces go up or down.
  - [x] Add async versions of network monitoring functions
  - [x] Implement NetworkManager::update_async() using tokio::spawn_blocking for I/O
  - [x] Replace netstat with native sysctlbyname implementation
  - [x] Add 64-bit counter support for high-bandwidth interfaces
  - [x] Implement automatic fallback mechanism for reliability
  - [~] Create comprehensive documentation for native implementation
  - [ ] Implement connection-level monitoring with TCP/UDP connection tracking
  - [ ] Add DNS resolution capabilities for hostnames and IP addresses
  - [ ] Implement per-application network usage tracking using Network.framework
  - [ ] Add event-driven hooks for network state changes
  - [ ] Improve test coverage for network/interface (currently 21.46%)
  - [ ] Improve test coverage for network/traffic (currently 52.63%)

- [~] **Testing & Stability**
  - [x] Add tests for power module with mock implementation (96.98% coverage)
  - [x] Fix visibility warning for CpuInfo type (more private than retrieve_cpu_info)
  - [~] Improve coverage for hardware/iokit module (currently 16.67%)
  - [ ] Add tests for utils modules:
    - [ ] utils/property_utils.rs (0% coverage)
      - [ ] Add tests for NSString handling
      - [ ] Add tests for property access methods
      - [ ] Add tests for error conditions
      - [ ] Add tests for thread safety
    - [ ] utils/test_utils.rs (0% coverage)
    - [ ] utils/bindings.rs (90.70% coverage)
    - [~] utils/mod.rs (68.71% coverage)
  - [~] Improve async testing to validate non-blocking behavior
  - [ ] Add integration tests for property access patterns
  - [ ] Add stress tests for concurrent property access

### **Critical Coverage Areas Needing Attention**

Based on latest test results:

- [ ] battery/monitors.rs (0% coverage)
- [ ] battery/types.rs (0% coverage)
- [ ] core/metrics/* modules (0% coverage)
- [ ] cpu/cpu_impl.rs (0% coverage)
- [ ] disk/monitors.rs (0% coverage)
- [ ] gpu/gpu_impl.rs (0.63% coverage)
- [ ] memory/monitors.rs (0% coverage)
- [ ] network/interface.rs (0% coverage)
- [ ] network/traffic.rs (0% coverage)
- [ ] process/mod.rs (0% coverage)
- [ ] temperature/monitors.rs (0% coverage)

### **Well-Covered Modules (>70%)**

- [~] error.rs (66.07% line coverage, 67.86% function coverage)
- [~] lib.rs (78.38% line coverage, 77.78% function coverage)
- [~] memory/types.rs (80.65% line coverage, 66.67% function coverage)
- [~] network/mod.rs (100% coverage)
- [~] resource/mod.rs (73.58% line coverage, 57.78% function coverage)
- [~] utils/ffi/bindings.rs (80% line coverage, 76.92% function coverage)

---

## **Phase 3: Optimization & Advanced Features (0.3.0) - Planned**

**Goal:** Optimize for performance and introduce advanced tracking.

- [ ] **Optimize CPU and memory metrics for lower overhead**
- [x] **Hardware Monitoring**
  - [ ] Implement fan control features for supported macOS devices.
  - [x] Improve power management insights by tracking energy consumption.
  - [x] Implement component-level power tracking (CPU, GPU, memory, etc).
  - [x] Add asynchronous power monitoring support with tokio tasks
  - [x] Create example programs for power monitoring (synchronous and asynchronous)
- [ ] **Performance Optimizations**

  - [ ] Reduce memory footprint by optimizing data structures.
  - [ ] Improve CPU efficiency by limiting unnecessary polling intervals.
  - [ ] Enhance async handling to ensure minimal blocking operations.

- [ ] **Event-based Monitoring**

  - [ ] Implement event-driven hooks for tracking system state changes.
  - [ ] Reduce reliance on polling where possible.

- [ ] **Testing Improvements**
  - [ ] Improve test coverage for low-coverage modules:
    - [ ] Add tests for battery module (currently 0% coverage)
    - [x] Add tests for power module (currently 96.98% coverage)
    - [x] Add tests for system module (currently 90.70% coverage)
    - [~] Add tests for utils modules:
      - [~] utils/property_utils.rs (0% coverage)
      - [~] utils/test_utils.rs (0% coverage)
      - [ ] utils/bindings.rs (28% coverage)
      - [~] utils/mod.rs (68.71% coverage)
    - [~] Improve coverage for hardware/iokit (currently 16.67% coverage)
    - [ ] Improve coverage for network/interface (currently 21.46% coverage)
    - [ ] Improve coverage for hardware/cpu/frequency.rs (currently 42.06% coverage)
    - [ ] Improve coverage for hardware/temperature/mod.rs (currently 62.11% coverage)
    - [ ] Improve coverage for error.rs (currently 32.61% coverage)
  - [ ] Implement more robust async tests following tokio best practices:
    - [x] Test async power monitoring functions
    - [x] Test memory async functions
    - [ ] Test network async operations
    - [ ] Test disk async operations
  - [ ] Introduce performance benchmarks to measure impact on system resources.
  - [ ] Conduct stress testing under heavy system load conditions.
  - [ ] Set up coverage threshold in CI to maintain or improve coverage over time.
  - [ ] Add integration tests for common use cases and workflows.

---

## **Phase 4: Final Optimizations & Production Release (1.0.0) - Planned**

**Goal:** Prepare for stable, production-ready release.

- [ ] **Ensure CPU and memory modules are feature-complete**
- [x] **Full Documentation & Examples**

  - [~] Complete API documentation for all modules.
  - [~] Provide real-world usage examples for developers.
  - [~] Improve README with installation and usage instructions.
  - [x] Set up mdBook structure for comprehensive documentation
  - [~] Create module documentation with examples
  - [~] Document internal architecture and contributing guidelines

- [ ] **Comprehensive Test Coverage**

  - [ ] Ensure all modules have unit and integration tests.
  - [ ] Validate async correctness in real-world scenarios.

- [ ] **Performance Benchmarking**

  - [ ] Conduct final performance tests and fine-tune any remaining bottlenecks.

- [ ] **Ensure API Consistency Before `1.0.0`**

  - [ ] Conduct final API review to ensure consistency in function signatures.
  - [ ] Deprecate or remove unused experimental features.

- [ ] **Release on Crates.io**
  - [ ] Finalize release notes and versioning.
  - [ ] Publish `darwin-metrics` to `crates.io` for public use.

---

## **Dependencies & Maintenance**

Based on the latest dependency updates observed in similar projects:

- [ ] **Update Core Dependencies**
  - [ ] libc (current: ~0.2.167/168, target: 0.2.168+)
  - [ ] tokio (verify using latest version)
  - [ ] async-trait (check for compatibility with recent Rust updates)

- [ ] **Development Dependencies**
  - [ ] Update testing frameworks
  - [ ] Update code coverage tools
  - [ ] Consider adding Rust 2021 edition support if not already done

---

This roadmap reflects our **progress so far** and the **next planned tasks**. 🚀
