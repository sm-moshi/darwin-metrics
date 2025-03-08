# darwin-metrics - a macOS System Metrics Crate - Development Roadmap

---

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [x] Write a build script (`build.rs`) to compile and link macOS frameworks
- [x] Implement a minimal working example with IOKit bindings

---

## **Phase 1: Codebase Refactoring & Cleanup (0.1.0) - In Progress**

**Goal:** Improve structure, maintainability, and performance before implementing new features.

### **Code Refactoring & Cleanup**

- [x] **Process Monitoring**
  - [x] Improve modularity by separating concerns into dedicated functions and modules.
  - [x] Refactor function and variable names to follow Rust conventions (`snake_case` for functions, `PascalCase` for structs).
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

- [~] **CPU Metrics**
  - [~] Refactored CPU module (mod.rs, cpu.rs, frequency.rs).
  - [~] Implemented CPU frequency monitoring.
  - [~] Improved CPU temperature retrieval.

- [~] **GPU Metrics**
  - [x] Implemented CoreFoundation approach for basic GPU metrics.
  - [x] Added GPU utilization and VRAM usage tracking.
  - [x] Integrated with IOKit to access GPU performance information.
  - [~] Fixed memory safety issues during test teardown (SIGSEGV) by disabling problematic tests and using autoreleasepools.
  - [~] Improved cleanup of IOKit resources with autoreleasepools.
  - [ ] Create real-world examples to validate the implementation.
  - [ ] Enhance tests to be more resilient in different environments.
  - [ ] Integrate with Metal API for advanced GPU monitoring (future enhancement).

- [ ] **Network Monitoring**
  - [ ] Implement bandwidth tracking, ensuring accurate upload/download speed calculations.
  - [ ] Introduce real-time network state monitoring to detect active/inactive connections.
  - [ ] Improve packet loss and error tracking by gathering statistics from system interfaces.
  - [ ] Implement asynchronous network data collection to avoid blocking operations.
  - [ ] Write unit tests for different network conditions and interface types.

- [ ] **Temperature Monitoring**
  - [ ] Add fan speed tracking by retrieving RPM values from available system sensors.
  - [ ] Implement thermal zone monitoring to track system heat distribution.
  - [ ] Detect and report thermal throttling by monitoring CPU/GPU clock adjustments.
  - [ ] Improve efficiency of temperature polling to avoid unnecessary resource consumption.

- [ ] **General Code Cleanup** *(In Progress)*
  - [x] Refactored system module and merged architecture detection.
  - [x] Applied Rust idiomatic principles from `.windsurfrules`, ensuring consistent formatting.
  - [x] Removed redundant or unused code.
  - [x] Centralized FFI bindings in src/utils/bindings.rs for better maintainability.
  - [ ] Improve error handling and ensure meaningful propagation of system errors.
  - [x] Ensure existing tests pass after refactoring, updating them where necessary.

---

## **Phase 2: Enhanced System Metrics (0.2.0) - Planned**

**Goal:** Expand monitoring capabilities with additional system metrics.

### **New Features**

- [ ] **Enhance CPU and GPU monitoring with async processing**
- [ ] **GPU Performance Metrics**
  - [ ] Track GPU utilization over time to monitor workload distribution.
  - [ ] Measure GPU memory consumption and optimize reporting.

- [~] **Advanced Process Monitoring**
  - [x] Implement parent-child process hierarchy tracking for improved system visibility.
  - [x] Add per-process thread monitoring to gather insights on concurrency behavior.
  - [x] Improve resource usage tracking with additional data on CPU/memory consumption.
  - [ ] Add environment variable retrieval for processes.
  - [ ] Add command-line arguments retrieval for processes.

- [ ] **Disk & Storage Monitoring**
  - [ ] Track I/O performance by measuring read/write speeds for individual drives.
  - [ ] Monitor volume usage, including available/free space calculations.
  - [ ] Implement tracking for mounted disk devices and storage partitions.

- [ ] **Network Enhancements**
  - [ ] Implement packet loss and error tracking to improve network diagnostics.
  - [ ] Add connection state tracking to detect when network interfaces go up or down.

- [ ] **Testing & Stability**
  - [ ] Expand test coverage for new system metrics, ensuring accuracy in collected data.
  - [ ] Improve async testing to validate non-blocking behavior.

---

## **Phase 3: Optimization & Advanced Features (0.3.0) - Planned**

**Goal:** Optimize for performance and introduce advanced tracking.

- [ ] **Optimize CPU and memory metrics for lower overhead**
- [ ] **Hardware Monitoring**
  - [ ] Implement fan control features for supported macOS devices.
  - [ ] Improve power management insights by tracking energy consumption.
  
- [ ] **Performance Optimizations**
  - [ ] Reduce memory footprint by optimizing data structures.
  - [ ] Improve CPU efficiency by limiting unnecessary polling intervals.
  - [ ] Enhance async handling to ensure minimal blocking operations.

- [ ] **Event-based Monitoring**
  - [ ] Implement event-driven hooks for tracking system state changes.
  - [ ] Reduce reliance on polling where possible.

- [ ] **Testing Improvements**
  - [ ] Introduce performance benchmarks to measure impact on system resources.
  - [ ] Conduct stress testing under heavy system load conditions.

---

## **Phase 4: Final Optimizations & Production Release (1.0.0) - Planned**

**Goal:** Prepare for stable, production-ready release.

- [ ] **Ensure CPU and memory modules are feature-complete**
- [ ] **Full Documentation & Examples**

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

This roadmap reflects our **progress so far** and the **next planned tasks**. 🚀
