# TODO List for macOS System Metrics Crate

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [x] Set up `swift-bridge` for Rust-Swift interoperability
- [x] Configure library crate types (staticlib, cdylib, rlib)
- [x] Set up basic error handling and result types
- [ ] Create a Swift module for macOS API bindings
- [ ] Write a build script (`build.rs`) to compile Swift code
- [ ] Implement a minimal working example

## **Core Features**

### **Battery Metrics** (In Progress)

- [x] Define Battery struct and interface
- [x] Implement basic battery info functions
- [ ] Implement macOS-specific battery info retrieval
- [ ] Add battery health and cycle count information
- [ ] Add power source detection (AC/Battery)

### **CPU Metrics** (In Progress)

- [x] Define CPU struct and interface
- [ ] Get CPU usage percentage per core
- [ ] Get CPU model name and frequency
- [ ] Fetch total CPU load (user, system, idle)
- [ ] Implement CPU temperature monitoring

### **Memory Metrics** (In Progress)

- [x] Define Memory struct and interface
- [x] Set up memory info data structure
- [ ] Implement macOS memory pressure level detection
- [ ] Get total and used RAM
- [ ] Fetch swap usage
- [ ] Calculate memory pressure levels

### **GPU Metrics** (In Progress)

- [ ] Implement usage of **Metal API** for this!
  - [x] Define GPU struct and interface
  - [x] Set up GPU info data structure
  - [ ] Get active GPU model
  - [ ] Fetch GPU usage percentage
  - [ ] Monitor VRAM consumption
  - [ ] Implement multi-GPU support

### **Disk Metrics** (In Progress)

- [x] Define Disk struct and interface
- [x] Set up disk info data structure
- [ ] Get total and used disk space
- [ ] Fetch read/write speeds
- [ ] Monitor disk I/O activity
- [ ] Add support for multiple volumes

### **Temperature Metrics via SMC** (In Progress)

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
- [ ] Add error context and chaining
- [ ] Implement recovery strategies

## **Testing & Benchmarking**

- [x] Set up test infrastructure
- [x] Add basic unit tests for Battery struct
- [ ] Write unit tests for all metric types
- [ ] Implement integration tests for Rust-Swift communication
- [ ] Add benchmarking suite
- [ ] Test on both Intel and Apple Silicon

## **Documentation**

- [x] Set up basic module documentation
- [ ] Write comprehensive API documentation
- [ ] Add usage examples for each metric type
- [ ] Document Swift-Rust FFI interface
- [ ] Create example applications
- [ ] Add performance considerations documentation

## **Packaging & Distribution**

- [x] Configure crate features
- [x] Set up release profile optimizations
- [ ] Ensure compatibility with Apple Silicon & Intel Macs (macOS 15+)
- [ ] Implement dynamic linking for Swift libraries
- [ ] Add version compatibility matrix
- [ ] Publish to `crates.io`

## **Future Enhancements**

- [ ] Add metric history tracking
- [ ] Implement metric alerting system
- [ ] Add system event notifications
- [ ] Support for power management profiles
- [ ] Add network interface monitoring
