# Changelog

All notable changes to the darwin-metrics project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Fixed

### Changed

## [0.2.0-alpha1] - 2025-03-23

### Added

- Enhanced GPU capabilities:
  - Improved hardware detection for Apple Silicon chips
  - Added comprehensive characteristics (core count, clock speed, raytracing)
  - Added support for multiple GPU configurations
  - Improved Metal API integration
  - Added example program and tests for hardware detection
- New Power module features:
  - Component-level power consumption tracking
  - Asynchronous monitoring support with tokio tasks
  - Stable mock implementation for testing
  - Added power_monitor.rs and power_monitor_async.rs examples
- Enhanced network capabilities:
  - Native API implementation for traffic statistics
  - 64-bit network counters for high-bandwidth interfaces
  - Fallback mechanism for macOS compatibility
  - Added network_monitor.rs example
- Memory monitoring improvements:
  - Memory pressure monitoring with configurable thresholds
  - Virtual memory and swap usage tracking
  - Page fault tracking and statistics
  - Memory bandwidth monitoring
- Enhanced async support throughout codebase

### Fixed

- IOKit improvements:
  - Resolved critical segmentation fault in service acquisition
  - Enhanced service handling robustness
  - Improved memory safety in interface implementation
  - Added Default implementation for SMCKeyData_t
  - Simplified get_service implementation
- Network enhancements:
  - Migrated to native sysctlbyname API
  - Improved traffic statistics reliability
  - Added comprehensive tests for native implementation
  - Implemented dual-approach system with fallback
- Memory safety:
  - Improved FFI code safety
  - Enhanced error handling in monitoring
  - Improved async monitoring reliability
- Improved process tree error handling

### Changed

- Architecture improvements:
  - Enhanced memory management in Objective-C interfaces and test environments
  - Improved error handling in IOKit bindings
  - Modified IOKit trait with safer SMC key access
  - Refactored network module to use native macOS APIs
  - Added SystemConfiguration framework bindings
- Code organization:
  - Improved code formatting and organization
  - Reorganized imports and interfaces
  - Enhanced documentation
  - Improved metrics collection efficiency

## [0.1.5] - 2025-03-10

### 0.1.5 - Fixed

- Fixed memory management in IOKit interfaces with proper CoreFoundation object handling
- Replaced Objective-C retain message sending with CFRetain for CoreFoundation objects
- Added autoreleasepool wrappers around memory-sensitive operations in IOKit module
- Fixed get_string_property and io_service_get_matching_service methods with safer memory handling
- Improved io_registry_entry_get_parent with autoreleasepool for consistent memory management
- Added proper test safety features to skip crash-prone tests when using coverage tools

## [0.1.4] - 2025-03-10

### 0.1.4 - Fixed

- Fixed warnings about non-snake_case fields in bindings.rs
- Fixed SIGSEGV crashes during test execution
- Improved test safety by fixing memory management issues
- Reorganized tests into dedicated files to improve maintainability
- Fixed missing imports in test modules (msg_send and autoreleasepool)
- Fixed clippy error about test module placement in bindings.rs
- Improved test coverage for hardware/iokit module (from 16.67% to 22.18%)

## [0.1.3] - 2025-03-09

### 0.1.3 - Fixed

- Fixed docs.rs build issues by ensuring proper configuration flags are set
- Made toolchain usage consistent (beta for linting, stable for building)
- Improved GitHub CI workflow to properly handle linting and formatting
- Fixed HTML root URL version consistency
- Updated build.rs to correctly handle docs.rs environment

## [0.1.2] - 2025-03-09

### 0.1.2 - Fixed

- Fixed docs.rs compatibility issues
- Improved release workflow for macOS-specific crate

## [0.1.1] - 2025-03-09

### 0.1.1 - Changed

- Applied formatting via rustfmt
- Improved GitHub CI workflow

## [0.1.0] - 2025-03-09

### 0.1.0 - Added

- Completed Network module implementation with:
  - Network interface discovery using getifaddrs
  - Traffic statistics tracking using netstat
  - Interface categorization (Ethernet, WiFi, Loopback, etc.)
  - IP and MAC address collection
  - Traffic monitoring (bytes/packets sent/received)
  - Error tracking (errors, collisions)
  - Upload/download speed calculations
  - Interface state monitoring
- Added comprehensive network module documentation with usage examples
- Added comprehensive Temperature module implementation with:
  - Fan speed tracking with RPM values and utilization percentage
  - Multiple thermal zone monitoring (CPU, GPU, heatsink, ambient, battery)
  - Thermal throttling detection via SMC keys
  - Efficient temperature polling with configurable intervals
  - Comprehensive thermal metrics collection API
  - Fan control and monitoring capabilities
- Added comprehensive Memory module implementation with:
  - System memory tracking with detailed page state information
  - Memory pressure monitoring with configurable thresholds and callbacks
  - Swap usage tracking with pressure indicators
  - Asynchronous memory metrics collection capabilities
  - Resilient fallbacks for test environments
  - Comprehensive tests for all memory metrics
  - Example programs for both synchronous and async memory monitoring
- First public release with core functionality for CPU, GPU, memory, network, and thermal monitoring
- GitHub Actions CI/CD pipeline for automated testing and releases

### 0.1.0 - Changed

- Refactored GPU module to be more resilient against hardware access failures
- Simplified Objective-C message sending pattern to avoid UnwindSafe trait issues
- Updated CLAUDE.md with improved development guidelines from Cursor rules
- Enhanced CPU module with testable implementation patterns
- Completely reimplemented IOKit interface to properly bridge with macOS APIs
- Improved IOKit interface with proper SMC key temperature reading
- Refactored Temperature module to use the new IOKit SMC implementation
- Enhanced hardware temperature module with comprehensive sensor reading capabilities
- Reimplemented GPU metrics collection to reduce dependency on Metal API
- Improved GPU module error handling with proper fallbacks for different hardware configurations
- Enhanced GPU module to work better with Apple Silicon's unified memory architecture
- Improved memory management in IOKit interfaces with autoreleasepools
- Fixed problematic GPU tests that caused SIGSEGV crashes during test execution
- Centralized all FFI bindings in `src/utils/bindings.rs` for better maintainability
  - Moved Metal framework bindings to central location
  - Moved all statfs and filesystem bindings
  - Added system process bindings
  - Centralized all sysctl functions
- Enhanced memory module with async support and resilient fallbacks
- Improved swap usage tracking with better error handling in test environments
- Refactored IOKit, System, and Memory modules to use centralized bindings
- Enhanced lib.rs with comprehensive documentation for docs.rs compatibility
- Improved error handling for FFI functions with explicit error messages

## [0.0.x] - Previous Iterations

### 0.0.x - Fixed

- Fixed compilation errors in GPU module by resolving trait bound issues with `msg_send` macro
- Resolved issues with Chart struct references by providing a metrics_summary method instead
- Fixed multiple type safety issues in Objective-C interop code
- Removed redundant unwrap_or calls on primitive types
- Simplified GPU metrics collection to safely handle potential Metal framework errors
- Fixed variable mutability warning in CPU frequency module
- Fixed unused imports and variables in CPU module
- Fixed GPU memory management issues by wrapping IOKit calls in autoreleasepools
- Improved cleanup of Objective-C objects to prevent memory corruption
- Fixed type casting issues in IOKit bindings with proper ffi_c_void types
- Resolved duplicate sysctl definitions across multiple modules

### 0.0.x - Added

- Added better error handling for GPU device initialization
- Added fallback values for GPU metrics when hardware access fails
- Added comprehensive test suite for CPU module with mocked IOKit implementation
- Added unit tests for CPU frequency metrics and monitoring
- Added MockIOKit implementation using mockall for testing hardware interactions
- Added test utilities for creating Objective-C test objects
- Added proper SMC (System Management Controller) interface for temperature readings
- Added CPU and GPU temperature sensor access through SMC keys
- Added fan speed monitoring capability through SMC
- Added CoreFoundation-based GPU metrics collection without Metal dependency
- Added GPU utilization tracking through AGPMController's performance capacity stats
- Added GPU memory usage tracking through IORegistry properties
- Added comprehensive Process module implementation with:
  - Process enumeration using sysctl and libproc
  - CPU and memory usage tracking for individual processes
  - Process I/O statistics monitoring
  - Process hierarchy and tree visualization
  - Child process tracking
  - Thread count monitoring
- Added comprehensive CPU metrics module with:
  - Detailed frequency monitoring with min/max/available frequencies
  - Per-core usage tracking
  - Temperature monitoring
  - Physical and logical core detection
  - CPU model detection
  - Standardized CPU metrics interface
- Added comprehensive Network monitoring module with:
  - Network interface enumeration using getifaddrs
  - Per-interface traffic statistics (bytes/packets sent/received)
  - Network error and collision tracking
  - Real-time bandwidth calculations
  - Interface state monitoring (up/down)
  - MAC and IP address information
  - Support for various interface types (Ethernet, WiFi, Loopback, Virtual)
- Added comprehensive mdBook documentation structure in `/docs`
- Added documentation for modules, FFI bindings, and architecture
- Added comprehensive CPU module documentation with usage examples
- Added comprehensive Network module documentation with usage examples

### 0.0.x - Changed

- Refactored GPU module to be more resilient against hardware access failures
- Simplified Objective-C message sending pattern to avoid UnwindSafe trait issues
- Updated CLAUDE.md with improved development guidelines from Cursor rules
- Enhanced CPU module with testable implementation patterns
- Completely reimplemented IOKit interface to properly bridge with macOS APIs
- Improved IOKit interface with proper SMC key temperature reading
- Refactored Temperature module to use the new IOKit SMC implementation
- Enhanced hardware temperature module with comprehensive sensor reading capabilities
- Reimplemented GPU metrics collection to reduce dependency on Metal API
- Improved GPU module error handling with proper fallbacks for different hardware configurations
- Enhanced GPU module to work better with Apple Silicon's unified memory architecture
- Improved memory management in IOKit interfaces with autoreleasepools
- Fixed problematic GPU tests that caused SIGSEGV crashes during test execution
- Centralized all FFI bindings in `src/utils/bindings.rs` for better maintainability
  - Moved Metal framework bindings to central location
  - Moved all statfs and filesystem bindings
  - Added system process bindings
  - Centralized all sysctl functions
- Enhanced memory module with async support and resilient fallbacks
- Improved swap usage tracking with better error handling in test environments
- Refactored IOKit, System, and Memory modules to use centralized bindings
- Enhanced lib.rs with comprehensive documentation for docs.rs compatibility
- Improved error handling for FFI functions with explicit error messages

## 0.2.0-alpha1 Pre-Release Checklist

The following items need attention before the 0.2.0-alpha1 release:

### Core Functionality

- [ ] Fix remaining Clippy warnings from `cargo clippy --all-targets --all-features --workspace`
- [ ] Complete documentation for all new features and modules
- [ ] Ensure comprehensive test coverage for new functionality
- [ ] Verify all dependencies are up-to-date (especially libc and tokio)

### Memory Safety

- [ ] Review all unsafe blocks for proper documentation and safety guarantees
- [ ] Ensure consistent memory management in Objective-C and CoreFoundation interfaces
- [ ] Verify proper cleanup of resources in drop() implementations

### Code Quality

- [ ] Remove any remaining debug macros (println!, panic!) from production code
- [ ] Convert TODO comments to GitHub issues where appropriate
- [ ] Apply DRY principles to repetitive code sections
- [ ] Ensure code adheres to Rust 2021 idioms
