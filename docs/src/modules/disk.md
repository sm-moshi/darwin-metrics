# Disk Monitoring

The `disk` module in `darwin-metrics` provides comprehensive monitoring of storage volumes and disk I/O performance on macOS systems.

## Features

The disk monitoring module offers the following capabilities:

- **Volume Information**: Details about mounted filesystems including capacity, usage, and mount points
- **Disk I/O Performance**: Read/write operations per second, throughput, and latency metrics
- **Disk Type Detection**: Identification of SSD, HDD, Fusion drives, and other storage types
- **Per-Volume Metrics**: Track individual volumes, partitions, and mount points
- **Path-Based Lookups**: Find which volume contains a specific file or directory

## Usage Examples

### Basic Volume Information

```rust
use darwin_metrics::disk::Disk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get information about all mounted volumes
    let volumes = Disk::get_all()?;
    
    for volume in volumes {
        println!("Volume: {}", volume.name);
        println!("  Mount point: {}", volume.mount_point);
        println!("  Filesystem: {}", volume.fs_type);
        println!("  Capacity: {}", volume.total_display());
        println!("  Used: {} ({}%)", 
            volume.used_display(), 
            volume.usage_percentage() as u32);
        println!("  Available: {}", volume.available_display());
        
        if volume.is_nearly_full() {
            println!("  WARNING: Volume is nearly full!");
        }
        
        println!();
    }
    
    // Get information about the root filesystem
    let root = Disk::get_info()?;
    println!("Root filesystem: {}", root.summary());
    
    // Get information about a specific path
    let home = Disk::get_for_path("/Users")?;
    println!("Home directory volume: {}", home.summary());
    
    Ok(())
}
```

### Disk Performance Monitoring

```rust
use darwin_metrics::disk::DiskMonitor;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the disk monitor
    let mut monitor = DiskMonitor::new();
    
    // Sample disk performance every second
    for _ in 0..10 {
        match monitor.get_performance() {
            Ok(performance) => {
                for (device, perf) in performance {
                    println!("Device: {}", device);
                    println!("  Read: {:.1} ops/s, {:.2} MB/s", 
                        perf.reads_per_second,
                        perf.bytes_read_per_second as f64 / (1024.0 * 1024.0));
                    println!("  Write: {:.1} ops/s, {:.2} MB/s", 
                        perf.writes_per_second,
                        perf.bytes_written_per_second as f64 / (1024.0 * 1024.0));
                    println!("  Latency: {:.2} ms read, {:.2} ms write", 
                        perf.read_latency_ms,
                        perf.write_latency_ms);
                    println!("  Utilization: {:.1}%", perf.utilization);
                    println!();
                }
            },
            Err(e) => println!("Error: {}", e),
        }
        
        // Update internal statistics 
        monitor.update()?;
        
        // Wait for next sample
        sleep(Duration::from_secs(1));
    }
    
    Ok(())
}
```

## Implementation Details

The disk module uses several macOS APIs to gather comprehensive storage metrics:

1. **statfs**: For basic volume information like capacity and usage
2. **IOKit**: For disk performance metrics and device type detection
3. **DiskArbitration framework**: For additional volume metadata
4. **FSEvents**: For monitoring filesystem changes (future enhancement)

## Performance Considerations

Disk I/O monitoring is designed to be lightweight, with a minimal performance impact on the system:

- Incremental updates track deltas between samples rather than absolute values
- Smart caching of filesystem metadata to reduce syscalls
- Configurable sampling rate to balance detail with overhead

## Advanced Features

### Disk Type Detection

The module can detect various storage types:

```rust
use darwin_metrics::disk::{Disk, DiskType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let volumes = Disk::get_all()?;
    
    for volume in volumes {
        let type_description = match volume.disk_type {
            DiskType::SSD => "Solid State Drive",
            DiskType::HDD => "Hard Disk Drive",
            DiskType::Fusion => "Fusion Drive",
            DiskType::External => "External Drive",
            DiskType::Network => "Network Volume",
            DiskType::RAM => "RAM Disk",
            DiskType::Virtual => "Virtual Disk",
            DiskType::Unknown => "Unknown Type",
        };
        
        println!("{}: {}", volume.name, type_description);
    }
    
    Ok(())
}
```

## Complete Example

For a full-featured example of disk monitoring, see the `examples/disk_monitor.rs` file in the repository, which demonstrates:

- Real-time volume information display
- I/O performance tracking
- Visual representation of disk usage and I/O rates
- Handling of error conditions