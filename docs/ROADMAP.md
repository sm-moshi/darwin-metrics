# 🗺 darwin-metrics Roadmap

🎯 Phase 1: Codebase Refactoring & Cleanup (0.1.0) - Current Phase

Goal: Improve structure, maintainability, and performance before implementing new features.

🔄 Code Refactoring & Cleanup
• Process Monitoring
• Improve overall module structure and dependency organization
• Ensure async improvements apply only where necessary to avoid unnecessary overhead
• Enhance process I/O tracking and resource usage monitoring
• GPU Metrics
• Refactor Metal API usage for better VRAM tracking
• Improve multi-GPU support and error handling
• Network Monitoring
• Implement bandwidth tracking and real-time network metrics
• Implement real-time connection state monitoring
• Add interface categorization and address tracking
• Temperature Monitoring
• Add fan speed and thermal zone monitoring
• Detect and report thermal throttling events
• General Code Cleanup
• Apply Rust idiomatic principles from .windsurfrules
• Enforce modular structure and remove redundant logic
• Improve error handling and ensure meaningful error propagation
• Ensure existing tests pass after refactoring

⸻

🚀 Phase 2: Enhanced System Metrics (0.2.0)

Goal: Expand monitoring capabilities with additional system metrics.

🔹 New Features
• GPU Performance Metrics
• Track GPU utilization and memory consumption
• Advanced Process Monitoring
• Implement parent-child process hierarchy tracking
• Add per-process thread monitoring
• Disk & Storage Monitoring
• Track I/O performance and volume usage
• Network Enhancements
• Implement packet loss and error tracking
• Add WiFi signal strength monitoring
• Add test coverage for new system metrics and async behavior

⸻

📈 Phase 3: Optimization & Advanced Features (0.3.0)

Goal: Optimize for performance and introduce advanced tracking.
• Hardware Monitoring
• Implement fan control and power management insights
• Performance Optimizations
• Reduce memory footprint and CPU overhead
• Improve async handling for real-time metrics
• Event-based Monitoring
• Implement hooks for real-time system events
• Introduce performance and stress testing for real-time monitoring

⸻

⚡ Phase 4: Final Optimizations & Production Release (1.0.0)

Goal: Prepare for stable, production-ready release.
• Full Documentation & Examples
• Comprehensive Test Coverage
• Performance Benchmarking
• Ensure API consistency before 1.0.0
• Define a deprecation plan for unused/experimental features
• Release on Crates.io

⸻
