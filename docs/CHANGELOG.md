# Changelog

All notable changes to the darwin-metrics project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Enhanced GPU hardware detection with specific Apple Silicon chip identification
- Added comprehensive GPU characteristics struct with core count, clock speed, and raytracing capability
- Added example program to demonstrate improved GPU hardware detection
- Improved memory reporting for different GPU architectures
- Added tests for GPU hardware detection

## [0.2.0-dev] - 2025-03-10

### Added

- Created base for 0.2.x development branch
- Enhanced async support throughout codebase
- Improved Metal API integration for more reliable GPU monitoring

### Changed

- Refactored GPU module for better stability on Apple Silicon
- Improved error handling in IOKit bindings
- Enhanced memory management in Objective-C interfaces

## [0.1.0] - 2025-03-09

### Added

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
- Created example program demonstrating network monitoring capabilities
- Completed Temperature module implementation with:
  - Fan speed tracking with RPM values and utilization percentage
  - Multiple thermal zone monitoring (CPU, GPU, heatsink, ambient, battery)
  - Thermal throttling detection via SMC keys
  - Efficient temperature polling with configurable intervals
  - Comprehensive thermal metrics collection API
  - Fan control and monitoring capabilities
- Completed Memory module implementation with:
  - System memory tracking with detailed page state information
  - Memory pressure monitoring with configurable thresholds and callbacks
  - Swap usage tracking with pressure indicators
  - Asynchronous memory metrics collection capabilities
  - Resilient fallbacks for test environments
  - Comprehensive tests for all memory metrics
  - Example programs for both synchronous and async memory monitoring
- First public release with core functionality for CPU, GPU, memory, network, and thermal monitoring
- GitHub Actions CI/CD pipeline for automated testing and releases

## [0.0.x] - Previous Iterations

### Fixed

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

### Added

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

### Changed

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

### TODO

- GPU Implementation Issues:

  - Improve GPU hardware detection for various Mac models
  - Create more comprehensive examples to validate the GPU implementation
  - Add support for multiple GPU configurations
  - Add utilization tracking for specialized GPU components (like Media Engines)

- API Implementation Issues:

  - Complete real implementation of host_statistics64 for CPU core usage monitoring and VM memory statistics
  - Implement proper frequency detection through sysctlbyname("hw.cpufrequency") calls
  - Verify AppleACPICPU service compatibility with Objective-C method calls (`numberOfCores`, `numberOfProcessorCores`, etc.)

- System Integration Issues:
  - Add support for individual core temperature sensors through additional SMC keys
  - Fix system module API calls to properly detect M1/M2/M3 chip variants
  - Add support for additional SMC temperature sensors (battery, SSD, etc.)
  - Extend fan speed monitoring with RPM conversion for different fan types
- GPU Advanced Metrics (Future):

  - Implement detailed GPU utilization metrics using Metal Performance Shaders
  - Add support for multiple GPU configurations
  - Improve GPU memory tracking accuracy with Metal API for dedicated GPUs
  - Implement Neural Engine usage monitoring for Apple Silicon

- Environment Compatibility:
  - Ensure compatibility with Fish shell environment and aliases:
    - Use `rg` instead of `grep` (ripgrep)
    - Use `fd` instead of `find`
    - Use `bat` instead of `cat`
    - Use `eza --tree` instead of `tree`
    - Note: Some tools are installed in `~/.cargo/bin/` and others in `/opt/homebrew/bin/`
