# 🗺️ Roadmap for darwin-metrics - a crate for native macOS system metrics

---

## 🎯 **Phase 1: Foundation** *(Weeks 1-2)*

*(Current Phase)*

### Core Setup

- [x] Set up Rust library structure
- [x] Implement direct IOKit bindings
- [x] Implement build automation (`build.rs`)
- [x] Create a minimal working example
  - [x] Battery status implementation
  - [x] CPU usage monitoring

---

## 🔄 **Phase 2: Core System Metrics** *(Weeks 3-5)*

*(In Progress)*

### System Monitoring

- [~] Implement CPU metrics
  - [x] Usage monitoring
  - [x] Frequency tracking
  - [x] Model information
- [~] Implement memory metrics
  - [x] Total memory tracking
  - [ ] Used memory monitoring
  - [ ] Swap usage tracking
- [ ] Implement disk metrics
  - [ ] Storage monitoring
  - [ ] I/O statistics
- [~] Testing
  - [x] Unit tests for core functionality
  - [ ] Integration tests for IOKit bindings

---

## 🎮 **Phase 3: Advanced Hardware Monitoring** *(Weeks 6-8)*

### Hardware Features

- [ ] GPU Monitoring
  - [ ] Usage tracking via IOKit
  - [ ] VRAM monitoring
  - [ ] Active GPU detection
- [ ] Temperature & Cooling
  - [ ] Fan speed monitoring via SMC
  - [ ] Temperature tracking
- [ ] Performance
  - [ ] Optimize IOKit calls
  - [ ] Implement data caching
- [ ] Error Handling
  - [ ] Robust error recovery
  - [ ] Graceful degradation

---

## 📊 **Phase 4: Process Monitoring & System Info** *(Weeks 9-11)*

### System Integration

- [ ] Process Monitoring
  - [ ] Process listing via libproc
  - [ ] Per-process CPU usage
  - [ ] Per-process memory usage
- [ ] System Information
  - [ ] Kernel version tracking
  - [ ] System uptime monitoring
  - [ ] macOS version detection
- [ ] Logging & Testing
  - [ ] Structured logging system
  - [ ] Comprehensive integration tests
  - [ ] Performance benchmarks

---

## ⚡ **Phase 5: Optimization & Release** *(Weeks 12-14)*

### Final Steps

- [ ] Performance Optimization
  - [ ] Minimize system call overhead
  - [ ] Optimize memory usage
  - [ ] Reduce CPU impact
- [ ] API Finalization
  - [ ] Stabilize public API
  - [ ] Ensure backward compatibility
  - [ ] Implement proper error types
- [ ] Documentation & Release
  - [ ] Complete API documentation
  - [ ] Write usage examples
  - [ ] Document IOKit integration
  - [ ] Publish to `crates.io`
  - [ ] Create GitHub release

---

*Note: Timeline estimates are approximate and may be adjusted based on development progress and priorities.*
