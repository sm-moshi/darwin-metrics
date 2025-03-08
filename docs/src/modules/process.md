# Process Monitoring

The process module provides comprehensive monitoring of processes running on your macOS system. It offers ways to enumerate running processes, track their resource usage, obtain detailed information about specific processes, and even visualize process hierarchies.

## Basic Usage

```rust,no_run,ignore
// Basic process monitoring example
use darwin_metrics::process::Process;
use std::time::Duration;

// Get all running processes
let processes = Process::get_all().unwrap();
println!("Total processes: {}", processes.len());

// Get information about a specific process
let pid = std::process::id(); // Our own process ID
let process = Process::get_by_pid(pid).unwrap();
println!("Process: {} (PID: {})", process.name, process.pid);
println!("CPU Usage: {}%", process.cpu_usage);
println!("Memory Usage: {} bytes", process.memory_usage);
println!("Thread Count: {}", process.thread_count);
```

## Key Features

### Process Information

The `Process` struct provides access to critical metrics:

- **Basic Info**: Process ID (PID), name, parent PID
- **Resource Usage**: CPU usage percentage, memory consumption
- **Performance Metrics**: Thread count, uptime, I/O statistics
- **Process State**: Running, suspended, etc.

### Process Enumeration

There are multiple ways to retrieve process information:

```rust,no_run,ignore
// Process enumeration examples
use darwin_metrics::process::Process;

// Get all processes
let all_processes = Process::get_all().unwrap();

// Get process by specific PID
let process = Process::get_by_pid(1234).unwrap();

// Get child processes of a specific process
let children = Process::get_child_processes(1234).unwrap();

// Get parent process
let parent_pid = Process::get_parent_pid(1234).unwrap();
```

### Process Tree

You can visualize the entire process hierarchy:

```rust,no_run,ignore
// Process tree visualization
use darwin_metrics::process::Process;

// Get process tree (process and depth pairs)
let process_tree = Process::get_process_tree().unwrap();

// Print process tree
for (process, depth) in process_tree {
    let indent = "  ".repeat(depth);
    println!("{}{} ({})", indent, process.name, process.pid);
}
```

### Real-time Monitoring

For continuous monitoring of process metrics:

```rust,no_run,ignore
// Real-time process monitoring with async
use darwin_metrics::process::Process;
use futures::stream::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let pid = std::process::id();
    let interval = Duration::from_secs(1);
    
    // Create a stream of process metrics
    let mut stream = Process::monitor_metrics(pid, interval);
    
    // Process the metrics as they arrive
    while let Some(result) = stream.next().await {
        if let Ok(process) = result {
            println!("CPU: {}%, Memory: {}", process.cpu_usage, process.memory_usage);
        }
    }
}
```

## Implementation Details

The process module uses a hybrid approach for efficiency:

1. **Bulk Retrieval**: Uses `sysctl` for efficient retrieval of all processes at once
2. **Detailed Information**: Uses `libproc` for gathering detailed metrics about specific processes
3. **Fallback Mechanism**: If one approach fails, the module automatically falls back to alternatives

This design ensures both performance and reliability across different macOS versions and permission scenarios.

## I/O Statistics

For each process, you can retrieve detailed I/O statistics:

```rust,no_run,ignore
// Process I/O statistics example
use darwin_metrics::process::Process;

// Get current process ID
let pid = std::process::id();

// Get process with I/O stats
let process = Process::get_by_pid(pid).unwrap();

// Access I/O information
println!("Read bytes: {}", process.io_stats.read_bytes);
println!("Write bytes: {}", process.io_stats.write_bytes);
println!("Read operations: {}", process.io_stats.read_count);
println!("Write operations: {}", process.io_stats.write_count);
```

## System Processes

The module provides utilities to identify system processes:

```rust,no_run,ignore
// System process identification
use darwin_metrics::process::Process;

// Get current process ID
let pid = std::process::id();

// Check if this is a system process
let process = Process::get_by_pid(pid).unwrap();
if process.is_system_process() {
    println!("{} is a system process", process.name);
}
```

## Performance Considerations

- The first call to `get_all()` might be slower as it initializes internal caches
- Subsequent calls will be faster due to optimized data structures
- Using `monitor_metrics()` is more efficient than repeatedly calling `get_by_pid()`
- The module cleans up its internal history tracking to prevent memory leaks
