# GPU Metrics

The `GPU` module in `darwin-metrics` provides comprehensive monitoring of GPU resources on macOS systems, including integrated GPUs on Apple Silicon and discrete GPUs on Intel-based Macs.

## Features

The GPU module offers the following metrics:

- **GPU Utilization**: Current GPU usage as a percentage (0-100%)
- **Memory Usage**: Used, free, and total GPU memory
- **Temperature**: Current GPU temperature in degrees Celsius (where available)
- **Device Information**: GPU model name and device information
- **Fallback Mechanisms**: Works across different Mac models with graceful degradation

## Implementation Details

The GPU metrics are obtained through a combination of macOS frameworks:

1. **IOKit**: Primary source of GPU statistics including utilization, memory usage, and thermal information
2. **Metal**: Used as a fallback for GPU name retrieval
3. **SMC (System Management Controller)**: Used for accessing temperature data

The module handles Apple Silicon systems (with unified memory architecture) differently than Intel-based Macs with discrete GPUs.

## Usage Examples

### Basic GPU Information

```rust
use darwin_metrics::hardware::gpu::GPU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GPU monitoring
    let gpu = GPU::new()?;
    
    // Get basic GPU information
    let name = gpu.name()?;
    println!("GPU: {}", name);
    
    // Get current utilization
    let utilization = gpu.utilization()?;
    println!("GPU Utilization: {:.1}%", utilization);
    
    // Get memory information
    let memory = gpu.memory_info()?;
    println!("Memory: {}/{} MB used", 
        memory.used / 1024 / 1024,
        memory.total / 1024 / 1024
    );
    
    // Get temperature (if available)
    match gpu.temperature() {
        Ok(temp) => println!("Temperature: {:.1}°C", temp),
        Err(_) => println!("Temperature: Not available"),
    }
    
    Ok(())
}
```

### Complete GPU Metrics

For a more comprehensive approach, you can get all metrics at once:

```rust
use darwin_metrics::hardware::gpu::GPU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = GPU::new()?;
    
    // Get all metrics in one call
    let metrics = gpu.metrics()?;
    
    println!("GPU: {}", metrics.name);
    println!("Utilization: {:.1}%", metrics.utilization);
    
    if let Some(temp) = metrics.temperature {
        println!("Temperature: {:.1}°C", temp);
    }
    
    println!("Memory: {}/{} MB used ({:.1}%)", 
        metrics.memory.used / 1024 / 1024,
        metrics.memory.total / 1024 / 1024,
        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
    );
    
    Ok(())
}
```

## Handling Different Mac Models

The GPU module handles different Mac models with varying hardware support:

- **Apple Silicon (M1/M2/M3)**: Correctly identifies unified memory architecture
- **Intel Macs with discrete GPUs**: Reports accurate GPU model and memory information
- **Older Mac models**: Provides fallbacks for missing metrics

## Error Handling

The module implements robust error handling for varied environments:

- Graceful degradation when specific metrics are unavailable
- Comprehensive error types with context for debugging
- Safe memory management with autorelease pools to prevent leaks and crashes

## Memory Management

The GPU module uses the following techniques to ensure memory safety:

1. **Autorelease Pools**: All IOKit and Objective-C calls are wrapped in autorelease pools to properly manage temporary objects
2. **Reference Counting**: Proper retain/release patterns for Core Foundation objects
3. **Safe FFI**: Careful validation of foreign function interface calls
4. **Resource Cleanup**: Explicit cleanup of IOKit resources after use

These techniques prevent memory leaks and address previous issues with segmentation faults that occurred during testing and teardown phases. The implementation follows Apple's guidelines for memory management in mixed Rust/Objective-C code.

## Complete Examples

The repository includes several GPU monitoring examples that demonstrate different approaches:

1. `examples/gpu_static.rs`: Provides basic GPU information with minimal IOKit calls
   - One-time retrieval of GPU model, utilization, and memory
   - Suitable for simple system information tools

2. `examples/gpu_monitor_simple.rs`: A simulation-based GPU monitor
   - Uses a simple approach that doesn't rely heavily on IOKit
   - Good for testing and development

3. `examples/gpu_monitor_safe.rs`: A system load-based GPU metrics estimator
   - Uses proper autoreleasepool management to prevent memory leaks
   - Demonstrates safe Objective-C interoperability

4. `examples/gpu_monitor.rs`: A full-featured GPU monitor
   - Real-time GPU monitoring with automatic updates
   - Visual representation of utilization and memory usage
   - Robust error handling
   - Proper memory management with autoreleasepool

These examples show progressively more complex implementations, from basic static information to real-time monitoring with proper memory management.
