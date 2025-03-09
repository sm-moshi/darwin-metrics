# Roadmap

This document outlines the planned development roadmap for the `darwin-metrics` library. It provides a high-level overview of our goals and priorities for future releases.

## Current Status

The library is currently in Phase 1 of development, focusing on code refactoring and cleanup to establish a solid foundation for future feature development.

## Development Phases

### Phase 1: Codebase Refactoring & Cleanup (0.1.0) - In Progress

**Goal:** Improve structure, maintainability, and performance before implementing new features.

**Key Tasks:**

- ✅ Process Monitoring: Implement comprehensive process monitoring capabilities
- 🔄 CPU Metrics: Refactor and enhance CPU monitoring functionality
- 🔄 GPU Metrics: Implement and refine GPU metrics collection
- ⏳ Network Monitoring: Implement bandwidth tracking and network state monitoring
- ⏳ Temperature Monitoring: Enhance thermal sensor data collection
- 🔄 General Code Cleanup: Refactor and improve code quality
  - ✅ Centralized FFI bindings in src/utils/bindings.rs
  - ✅ Improved error handling and propagation
  - ✅ Enhanced code organization and documentation

### Phase 2: Enhanced System Metrics (0.2.0) - Planned

**Goal:** Expand monitoring capabilities with additional system metrics.

**Key Tasks:**

- Enhance CPU and GPU monitoring with async processing
- Implement detailed CPU and GPU frequency tracking
- Add advanced process monitoring features
- Implement comprehensive disk and storage monitoring
- Enhance network monitoring capabilities
- Expand test coverage

### Phase 3: Optimization & Advanced Features (0.3.0) - Planned

**Goal:** Optimize for performance and introduce advanced tracking.

**Key Tasks:**

- Optimize CPU and memory metrics for lower overhead
- Implement fan control features for supported devices
- Improve power management insights
- Reduce memory footprint and improve CPU efficiency
- Implement event-driven monitoring
- Enhance testing with performance benchmarks

### Phase 4: Final Optimizations & Production Release (1.0.0) - Planned

**Goal:** Prepare for stable, production-ready release.

**Key Tasks:**

- Complete API documentation and examples
- Ensure comprehensive test coverage
- Conduct final performance tests and optimizations
- Review API consistency
- Prepare for crates.io release

## Feature Priorities

These are our current feature priorities, ranked from highest to lowest:

1. **System Stability & Performance**: Ensuring the library has minimal impact on the host system
2. **Core Metrics Support**: Comprehensive coverage of CPU, memory, and process metrics
3. **GPU Metrics**: Support for detailed GPU usage and monitoring
4. **Network Metrics**: Monitoring of network interfaces and traffic
5. **Advanced Features**: Power management, fan control, and detailed temperature monitoring

## Release Schedule

While we don't have exact dates, our general timeline is:

- **0.1.0**: Q2 2024 (Alpha release with core functionality)
- **0.2.0**: Q3 2024 (Beta release with enhanced features)
- **0.3.0**: Q4 2024 (Release candidate with optimizations)
- **1.0.0**: Q1 2025 (Stable release with complete documentation)

## Contributing

We welcome contributions that align with our roadmap. If you're interested in helping, please check out our [Contributing Guide](./contributing.md) for details on how to get involved.

---

This roadmap is subject to change based on community feedback and project priorities. For the most up-to-date information, please check the repository's issue tracker.
