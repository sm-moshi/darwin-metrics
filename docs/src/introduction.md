# darwin-metrics

`darwin-metrics` is a Rust library that provides native access to macOS system metrics through low-level system APIs. This crate offers efficient, safe, and async-capable interfaces for monitoring system resources on macOS.

## Overview

The purpose of `darwin-metrics` is to give Rust developers easy access to macOS system information without having to deal with the complexities of FFI, Objective-C bridging, or the intricacies of Apple's frameworks. It serves as a Rust-native wrapper around various macOS APIs like IOKit, Foundation, Metal, and more.

## Features

- **CPU Monitoring**: Usage, frequency, temperature, and core information
- **Memory Analysis**: RAM usage, swap space, memory pressure, and page state tracking
- **GPU Information**: Model detection, utilization metrics, VRAM usage
- **Process Monitoring**: Resource usage stats, process enumeration, and hierarchies
- **Storage Metrics**: Disk space utilization and I/O performance
- **Power Management**: Battery status and power consumption data
- **Thermal Monitoring**: System-wide temperature sensors

## Design Philosophy

The library follows several important design principles:

1. **Safety First**: All unsafe FFI code is properly encapsulated, providing a safe Rust API to users
2. **Performance**: Minimizes overhead by using efficient access patterns and sharing resources
3. **Modularity**: Each system component is contained in its own module with clear APIs
4. **Error Handling**: Comprehensive error handling with specific error types
5. **Async Support**: Provides async interfaces where appropriate for non-blocking operations

## Platform Compatibility

This library is specifically designed for macOS systems. It has been tested on:

- macOS Monterey (12.x)
- macOS Ventura (13.x)
- macOS Sonoma (14.x)

> **Note**: Because `darwin-metrics` uses platform-specific APIs, it will not work on non-macOS systems.
