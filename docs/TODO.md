# TODO List for macOS System Metrics Crate

## **Project Setup**

- [x] Initialize Rust crate with `cargo init --lib`
- [ ] Set up `swift-bridge` for Rust-Swift interoperability
- [ ] Create a Swift module for macOS API bindings
- [ ] Write a build script (`build.rs`) to compile Swift code
- [ ] Implement a minimal working example

## **Core Features**

### **CPU Metrics**

- [ ] Get CPU usage percentage per core
- [ ] Get CPU model name and frequency
- [ ] Fetch total CPU load (user, system, idle)

### **Memory Metrics**

- [ ] Get total and used RAM
- [ ] Fetch swap usage
- [ ] Calculate memory pressure levels

### **GPU Metrics**

- [ ] Get active GPU model
- [ ] Fetch GPU usage percentage
- [ ] Monitor VRAM consumption

### **Disk Metrics**

- [ ] Get total and used disk space
- [ ] Fetch read/write speeds
- [ ] Monitor disk I/O activity

### **Energy Consumption**

- [ ] Fetch battery percentage
- [ ] Check if the device is charging
- [ ] Get estimated time remaining on battery

### **Fan Speed & Temperatures**

- [ ] Fetch fan RPM speeds
- [ ] Monitor CPU and GPU temperatures
- [ ] Fetch SSD and other hardware temperatures

### **Processes & System Information**

- [ ] List all running processes with CPU and memory usage
- [ ] Get system uptime
- [ ] Fetch kernel version and macOS version
- [ ] Fetch hostname and device model

## **Testing & Benchmarking**

- [ ] Write unit tests for Rust wrapper functions
- [ ] Implement integration tests for Rust-Swift communication
- [ ] Benchmark system call performance

## **Documentation**

- [ ] Write Rust API documentation
- [ ] Document Swift functions and mappings
- [ ] Create example Rust applications using the library

## **Packaging & Distribution**

- [ ] Ensure compatibility with Apple Silicon & Intel Macs (macOS 15+)
- [ ] Implement dynamic linking for Swift libraries
- [ ] Publish to `crates.io`
