# darwin-metrics - a macOS System Metrics Crate - Development Roadmap

---

## **Project Setup** *(Completed - 0.1.0)*

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [x] Write a build script (`build.rs`) to compile and link macOS frameworks
- [x] Implement a minimal working example with IOKit bindings

---

## **Core Features**

### **Thread Safety & Resource Management** *(0.1.0)*

- [x] Implement global thread-safe resource management
  - [x] Add atomic updates for shared state
  - [x] Implement thread-safe caching layer
  - [x] Add resource pooling for heavy operations
  - [x] Handle concurrent access patterns
  - [x] Implement proper cleanup mechanisms
  - [x] Add memory leak detection
  - [x] Monitor resource usage patterns

### **Battery Metrics** *(0.1.0 Core Features)*

- [x] Define Battery struct and interface
- [x] Implement basic battery info functions
- [x] Add bounds checking for percentage values
- [x] Implement macOS-specific battery info retrieval
- [x] Add battery health and cycle count information
- [x] Add power source detection (AC/Battery)
- [x] Add temperature monitoring
- [x] Add comprehensive test coverage for edge cases
  - [x] Test battery state transitions
  - [x] Test power source transition scenarios
  - [x] Test temperature range edge cases
  - [x] Test error handling scenarios

#### Additional Battery Features *(Post 0.1.0)*

- [ ] Battery Serial Number Information
  - [ ] Add BatterySerialNumber field to Battery struct
  - [ ] Implement serial number retrieval using IOKit
  - [ ] Add tests for serial number handling
- [ ] Battery Manufacture Date
  - [ ] Add ManufactureDate field with proper timestamp parsing
  - [ ] Implement date retrieval and formatting
  - [ ] Add tests for date parsing edge cases
- [ ] Power Adapter Information
  - [ ] Create PowerAdapter struct
  - [ ] Add adapter serial number support
  - [ ] Add adapter power rating information
  - [ ] Add adapter name and model details
  - [ ] Implement comprehensive adapter info tests
- [ ] Battery Calibration Status
  - [ ] Add calibration state tracking
  - [ ] Implement calibration needs detection
  - [ ] Add last calibration date tracking
  - [ ] Add tests for calibration state transitions

### **IOKit Integration** *(0.1.0)*

- [x] Define IOKit trait and implementation
- [x] Implement mock IOKit for testing
- [x] Add proper error handling for IOKit operations
- [x] Implement safe FFI boundaries for IOKit calls
- [x] Add null pointer safety checks
- [x] Fix remaining type conflicts and import issues
  - [x] Strengthen type safety around AnyObject casts
  - [x] Add proper error handling for type conversions
  - [x] Resolve unsafe block type conflicts
- [x] Add comprehensive IOKit error types
  - [x] Implement custom error types for IOKit operations
  - [x] Add detailed error messages and context
  - [x] Implement error conversion traits

### **CPU Metrics** *(0.1.0 Core Features)*

- [x] Define CPU struct and interface
- [x] Set up thread-safe CPU info data structure
- [x] Get CPU usage percentage per core
- [x] Get CPU model name and frequency
- [x] Fetch total CPU load (user, system, idle)
- [x] Implement CPU temperature monitoring
- [x] Normalize CPU usage calculation between 0-100%
- [x] Add comprehensive test coverage
  - [x] Test CPU initialization
  - [x] Test usage calculations
  - [x] Test error handling
  - [x] Test field visibility and access

### **Memory Metrics** *(0.1.0 Core Features)*

- [x] Define Memory struct and interface
- [x] Set up memory info data structure
- [x] Memory Pressure Detection
  - [x] Add memory pressure thresholds
  - [x] Implement pressure level callbacks
- [x] RAM Usage Monitoring
  - [x] Track active vs inactive memory
  - [x] Monitor compressed memory usage
  - [x] Track memory page states
- [x] Swap Management
  - [x] Monitor swap in/out rates
  - [x] Track swap file usage
  - [x] Monitor swap pressure

### **Temperature Metrics** *(0.1.0 Core Features)*

- [x] Define Temperature struct and interface
- [x] Implement temperature unit conversion (F/C)
- [ ] Hardware Monitoring
  - [ ] Track fan RPM speeds
  - [ ] Monitor thermal zones
  - [ ] Track thermal throttling
  - [ ] Monitor power impact
  - [ ] Implement thermal warnings

### **Network Metrics** *(0.1.0)*

- [x] Define Network struct and interface
- [x] Implement network interface enumeration
- [ ] Network Monitoring
  - [ ] Monitor throughput (upload/download)
  - [ ] Track interface states
  - [ ] Collect interface statistics

### **Process Monitoring** *(0.1.0)*

- [x] Define Process struct and interface
- [x] Implement process info collection trait
- [x] Add async process monitoring
- [x] Implement process metrics stream
  - [x] Fix interval state management
  - [x] Implement proper async stream polling
  - [x] Add thread-safe cloning support
  - [x] Add comprehensive doc tests
- [x] Implement Process::get_by_pid() with proper error handling
- [x] CPU usage tracking
  - [x] Normalize CPU usage calculation between 0-100%
  - [x] Handle permission errors gracefully
- [ ] Process Information Collection
  - [ ] Add process name resolution
  - [ ] Implement process state tracking
  - [ ] Add process hierarchy mapping
- [ ] Process Statistics
  - [ ] Memory usage monitoring
  - [ ] Resource limit tracking
  - [ ] I/O statistics collection
  - [ ] Thread count monitoring

### **GPU Metrics via Metal API!!** *(0.1.0 Core Features)*

- [ ] Define GPU struct and interface
- [ ] Implement basic GPU info retrieval
- [ ] Add memory usage tracking
- [ ] Add comprehensive test coverage
  - [ ] Test GPU initialization
  - [ ] Test metrics collection
  - [ ] Test error handling
  - [ ] Test mock implementations

### **Module Separation Plan** *(Post 0.1.0)*

### **Module Architecture**
- [ ] Create dedicated frequency monitoring module
- [ ] Implement separate power consumption module
- [ ] Maintain temperature-specific functionality

### **API Design**
- [ ] Implement unified top-level API
- [ ] Add clear, descriptive function names
- [ ] Include comprehensive documentation

### **User Experience**
- [ ] Add usage examples
- [ ] Implement builder pattern for configuration
- [ ] Include sensible defaults

---

## **Quality Assurance**

### **Testing Strategy** *(0.1.0)*

- [x] Test Suite Implementation
  - [x] Unit tests for all metric types
  - [x] Integration tests for IOKit interactions
  - [x] Concurrent access tests
  - [x] Error handling tests
  - [x] Resource cleanup tests
  - [x] Memory leak detection
  - [x] FFI layer fuzzing tests

### **Documentation** *(0.1.0)*

- [x] Set up basic module documentation
- [x] Write comprehensive API documentation for Battery module
- [x] Write comprehensive API documentation for IOKit module
- [x] Add proper examples in docstrings
- [x] Add usage examples for each metric type
- [x] Document error handling and safety considerations
- [x] Write documentation for remaining modules
- [ ] Create example applications

---

## **Distribution**

### **Packaging** *(0.1.0)*

- [x] Configure crate features
- [x] Set up release profile optimizations
- [x] Architecture Support
  - [x] apple-silicon (default)
- [x] Compatibility
  - [~] Ensure macOS 15+ support
  - [~] Add version compatibility matrix
- [ ] Publish to `crates.io`

### **Future Enhancements** *(Post 0.1.0)*

- [ ] Add metric history tracking
- [ ] Support power management profiles
- [ ] Add network interface monitoring
- [ ] Implement advanced thermal management
- [ ] Add power efficiency analytics

---

## **Code Quality & Safety**

### **Completed** *(0.1.0)*

- [x] Implement proper error propagation
- [x] Implement safe FFI boundaries
- [x] Add null pointer safety checks

### **In Progress** *(0.1.0)*

- [~] Implement thread-safe resource management
- [~] Add proper cleanup for system resources
- [~] Fix type conflicts and import issues

### **Pending** *(0.1.0)*

- [ ] Add memory leak detection tests
- [ ] Implement fuzzing tests for FFI layer
- [ ] Add objc2 runtime safety checks
- [ ] Implement comprehensive objc2 error handling
