# TODO List for macOS System Metrics Crate

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [x] Write a build script (`build.rs`) to compile and link frameworks
- [x] Implement a minimal working example

## **Core Features**

### **Battery Metrics** (In Progress)

- [x] Define Battery struct and interface
- [x] Implement basic battery info functions
- [x] Add bounds checking for percentage values
- [x] Implement proper mock expectations in tests
- [~] Implement macOS-specific battery info retrieval (partial)
- [~] Add battery health and cycle count information (partial)
- [~] Add power source detection (AC/Battery) (partial)
- [~] Add temperature monitoring (partial)
- [ ] Implement thread-safe battery info caching
- [ ] Add comprehensive test coverage for edge cases

Additional features that could be added in the future:

- [ ] Add battery serial number information
- [ ] Add battery manufacture date
- [ ] Add detailed power adapter information
- [ ] Implement battery calibration status

### **IOKit Integration** (In Progress)

- [x] Define IOKit trait and implementation
- [x] Implement mock IOKit for testing
- [x] Add proper error handling for IOKit operations
- [x] Implement safe FFI boundaries for IOKit calls
- [x] Add null pointer safety checks
- [ ] Add thread-safe resource management
- [ ] Fix remaining type conflicts and import issues
- [ ] Add comprehensive test coverage for error paths

## **objc2 Migration** (High Priority)

### **Core Migration**

- [x] Add objc2 and related framework dependencies
- [x] Configure objc2 features in Cargo.toml
- [x] Remove io-kit-sys dependency
- [x] Remove core-foundation dependency
- [x] Create safe abstractions using objc2 runtime

### **IOKit Refactoring**

- [x] Rewrite IOKit interface using objc2 macros and types
- [x] Implement IOService using objc2's class definition system
- [x] Convert existing FFI calls to use objc2's message sending
- [x] Add proper memory management using objc2's retain/release system
- [x] Implement autorelease pool management
- [x] Add thread safety using objc2's MainThreadMarker

### **Framework Integration**

- [x] Set up objc2-foundation integration
- [x] Set up objc2-core-foundation integration
- [x] Set up objc2-core-graphics integration
- [x] Implement block2 support for callbacks
- [x] Add proper exception handling

### **Testing & Safety**

- [x] Add tests for objc2 class implementations
- [x] Verify memory management in tests
- [x] Add thread safety tests
- [x] Implement proper error propagation with objc2
- [x] Add comprehensive documentation for objc2 usage

### **Performance Optimization**

- [x] Implement zero-cost abstractions using objc2
- [x] Optimize message sending with static selectors
- [x] Add compile-time selector verification
- [x] Implement efficient autorelease eliding
- [x] Profile and optimize critical paths

### **CPU Metrics** (In Progress)

- [x] Define CPU struct and interface
- [x] Set up thread-safe CPU info data structure
- [x] Get CPU usage percentage per core
- [x] Get CPU model name and frequency
- [x] Fetch total CPU load (user, system, idle)
- [x] Implement CPU temperature monitoring

### **Memory Metrics** (In Progress)

- [x] Define Memory struct and interface
- [x] Set up memory info data structure
- [ ] Implement mock data for testing
- [ ] Implement macOS memory pressure level detection
- [ ] Get total and used RAM
- [ ] Fetch swap usage
- [ ] Calculate memory pressure levels

### **GPU Metrics** (In Progress)

- [~] Define GPU struct and interface
- [~] Set up GPU info data structure
- [~] Add basic GPU name retrieval
- [~] Implement basic Metal framework integration
- [~] Define temperature and power monitoring interfaces
- [~] Implement GPU temperature monitoring
- [~] Implement power usage monitoring
- [~] Get active GPU model details
- [~] Fetch GPU usage percentage
- [~] Monitor VRAM consumption
- [~] Implement multi-GPU support

### **Disk Metrics** (In Progress)

- [ ] Define Disk struct and interface
- [ ] Set up disk info data structure
- [ ] Add basic disk space calculations
- [ ] Implement byte formatting utilities
- [ ] Get total and used disk space
- [ ] Fetch read/write speeds
- [ ] Monitor disk I/O activity
- [ ] Add support for multiple volumes

### **Temperature Metrics** (In Progress)

- [x] Define Temperature struct and interface
- [x] Implement temperature unit conversion (F/C)
- [ ] Fetch fan RPM speeds
- [ ] Monitor CPU and GPU temperatures
- [ ] Fetch SSD and other hardware temperatures

### **Network Metrics**

- [ ] Define Network struct and interface
- [ ] Implement network interface enumeration
- [ ] Monitor network throughput (upload/download)
- [ ] Track network interface states
- [ ] Collect interface statistics
- [ ] Monitor network connection states
- [ ] Implement Wi-Fi specific metrics
- [ ] Add support for multiple network interfaces
- [ ] Implement bandwidth monitoring

## **Feature Enhancements**

### **Async Support**

- [x] Add tokio dependency with full features
- [ ] Implement async versions of metric collection
- [ ] Add background monitoring capabilities
- [ ] Implement metric caching system

### **Error Handling**

- [x] Implement custom Error type
- [x] Add detailed error messages
- [x] Add error context and chaining
- [x] Implement recovery strategies
- [x] Add comprehensive error variants for all subsystems

## **Testing & Benchmarking**

- [x] Set up test infrastructure
- [x] Add basic unit tests for Battery struct
- [x] Ensure proper mock setup in tests
- [ ] Write unit tests for all metric types
- [ ] Add coverage tests for error cases
- [ ] Add tests for utility functions
- [ ] Add benchmarking suite
- [ ] Test on both Intel and Apple Silicon

## **Documentation**

- [x] Set up basic module documentation
- [x] Write comprehensive API documentation for Battery module
- [x] Write comprehensive API documentation for IOKit module
- [x] Add proper examples in docstrings
- [x] Add usage examples for each metric type
- [x] Document error handling and safety considerations
- [ ] Write comprehensive API documentation for remaining modules
- [ ] Create example applications
- [ ] Add performance considerations documentation

## **Packaging & Distribution**

- [x] Configure crate features
- [x] Set up release profile optimizations
- [ ] Ensure compatibility with Apple Silicon & Intel Macs (macOS 15+)
- [ ] Add version compatibility matrix
- [ ] Publish to `crates.io`

## **Future Enhancements**

- [ ] Add metric history tracking
- [ ] Support for power management profiles
- [ ] Add network interface monitoring

## **Code Quality & Safety**

- [x] Implement proper error propagation
- [x] Implement safe FFI boundaries
- [x] Add null pointer safety checks
- [~] Implement thread-safe resource management (partial)
- [~] Add proper cleanup for system resources (partial)
- [~] Fix type conflicts and import issues (partial)
- [ ] Add memory leak detection tests
- [ ] Implement fuzzing tests for FFI layer
- [ ] Add objc2 runtime safety checks
- [ ] Implement comprehensive objc2 error handling
