# darwin-metrics - a macOS System Metrics Crate - Development Roadmap

---

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [x] Write a build script (`build.rs`) to compile and link macOS frameworks
- [x] Implement a minimal working example with IOKit bindings

---

## **Phase 1: Codebase Refactoring & Cleanup (0.1.0)**

**Goal:** Improve structure, maintainability, and performance before implementing new features.

### **Code Refactoring & Cleanup**

- [ ] **Process Monitoring**
  - [ ] Improve modularity by separating concerns into dedicated functions and modules.
  - [ ] Refactor function and variable names to follow Rust conventions (`snake_case` for functions, `PascalCase` for structs).
  - [ ] Implement async-friendly architecture where necessary, ensuring minimal blocking operations.
  - [ ] Enhance process I/O tracking by gathering read/write statistics.
  - [ ] Add structured error handling for process-related failures.
  - [ ] Write additional unit tests for process enumeration, resource usage, and error scenarios.

- [ ] **GPU Metrics**
  - [ ] Refactor Metal API usage for better VRAM tracking and GPU utilization reporting.
  - [ ] Implement multi-GPU support by iterating through all available GPUs.
  - [ ] Add structured error handling for failed Metal API calls.
  - [ ] Optimize memory usage by caching retrieved GPU information.
  - [ ] Improve documentation on GPU monitoring capabilities.

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

- [ ] **General Code Cleanup**
  - [ ] Apply Rust idiomatic principles from `.windsurfrules`, ensuring consistent formatting.
  - [ ] Enforce modular structure by separating concerns into dedicated files and modules.
  - [ ] Remove redundant or unused code.
  - [ ] Improve error handling and ensure meaningful propagation of system errors.
  - [ ] Ensure existing tests pass after refactoring, updating them where necessary.

---

## **Phase 2: Enhanced System Metrics (0.2.0)**

**Goal:** Expand monitoring capabilities with additional system metrics.

### **New Features**

- [ ] **GPU Performance Metrics**
  - [ ] Track GPU utilization over time to monitor workload distribution.
  - [ ] Measure GPU memory consumption and optimize reporting.

- [ ] **Advanced Process Monitoring**
  - [ ] Implement parent-child process hierarchy tracking for improved system visibility.
  - [ ] Add per-process thread monitoring to gather insights on concurrency behavior.
  - [ ] Improve resource usage tracking with additional data on CPU/memory consumption.

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

## **Phase 3: Optimization & Advanced Features (0.3.0)**

**Goal:** Optimize for performance and introduce advanced tracking.

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

## **Phase 4: Final Optimizations & Production Release (1.0.0)**

**Goal:** Prepare for stable, production-ready release.

- [ ] **Full Documentation & Examples**
  - [ ] Complete API documentation for all modules.
  - [ ] Provide real-world usage examples for developers.
  - [ ] Improve README with installation and usage instructions.

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

This more detailed roadmap ensures **clear action items and structured development** while maintaining readability and long-term maintainability.
