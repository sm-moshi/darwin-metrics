# Thermal Monitoring

The Thermal module provides comprehensive temperature monitoring capabilities for macOS systems, allowing you to track CPU, GPU, and other temperature sensors, monitor fan speeds, and detect thermal throttling.

## Features

- **Multiple Temperature Sensors**: Access CPU, GPU, heatsink, ambient, and battery temperature readings
- **Fan Information**: Monitor fan speeds, min/max values, and utilization percentages
- **Thermal Throttling Detection**: Determine if the system is experiencing thermal throttling
- **Power Monitoring**: Track CPU power consumption (when available)
- **Asynchronous API**: Both synchronous and async interfaces are provided
- **Customizable Polling**: Configure polling intervals and thresholds

## Basic Usage

```rust
use darwin_metrics::hardware::Temperature;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Temperature instance
    let mut temperature = Temperature::new();
    
    // Get CPU temperature
    let cpu_temp = temperature.cpu_temperature()?;
    println!("CPU Temperature: {:.1}°C", cpu_temp);
    
    // Monitor temperatures over time
    for _ in 0..5 {
        // Get comprehensive thermal metrics
        let metrics = temperature.get_thermal_metrics()?;
        
        println!("CPU: {:.1}°C", metrics.cpu_temperature.unwrap_or(0.0));
        
        if let Some(gpu_temp) = metrics.gpu_temperature {
            println!("GPU: {:.1}°C", gpu_temp);
        }
        
        // Check if the system is throttling
        if metrics.is_throttling {
            println!("ALERT: System is thermal throttling!");
        }
        
        // Display fan information
        for (i, fan) in metrics.fans.iter().enumerate() {
            println!(
                "Fan {}: {} RPM ({:.1}%)",
                i, fan.speed_rpm, fan.percentage
            );
        }
        
        println!("---");
        thread::sleep(Duration::from_secs(2));
    }
    
    Ok(())
}
```

## Custom Configuration

You can customize the temperature monitoring behavior:

```rust
use darwin_metrics::hardware::{Temperature, TemperatureConfig};

fn main() {
    // Create a custom configuration
    let config = TemperatureConfig {
        poll_interval_ms: 5000,        // Poll every 5 seconds
        throttling_threshold: 90.0,    // Higher throttling threshold
        auto_refresh: true,            // Automatically refresh data
    };
    
    // Initialize temperature module with custom config
    let temperature = Temperature::with_config(config);
    
    // ... use temperature instance
}
```

## Sensor Locations

The `SensorLocation` enum represents different temperature sensor locations:

```rust
pub enum SensorLocation {
    Cpu,                // CPU temperature sensor
    Gpu,                // GPU temperature sensor
    Memory,             // System memory temperature sensor
    Storage,            // Storage/SSD temperature sensor
    Battery,            // Battery temperature sensor
    Heatsink,           // Heatsink temperature sensor
    Ambient,            // Ambient (inside case) temperature sensor
    Other(String),      // Other temperature sensor with a custom name
}
```

## Fan Information

The `Fan` struct provides detailed fan information:

```rust
pub struct Fan {
    pub name: String,      // Fan identifier (e.g., "CPU Fan", "System Fan")
    pub speed_rpm: u32,    // Current fan speed in RPM
    pub min_speed: u32,    // Minimum fan speed in RPM
    pub max_speed: u32,    // Maximum fan speed in RPM
    pub percentage: f64,   // Current fan utilization as a percentage (0-100%)
}
```

## Asynchronous API

For applications using async/await, the module provides async versions of all methods:

```rust
use darwin_metrics::hardware::Temperature;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut temperature = Temperature::new();
    
    // Get CPU temperature asynchronously
    let cpu_temp = temperature.cpu_temperature_async().await?;
    println!("CPU Temperature: {:.1}°C", cpu_temp);
    
    // Get comprehensive thermal metrics asynchronously
    let metrics = temperature.get_thermal_metrics_async().await?;
    
    // Check if the system is throttling asynchronously
    let is_throttling = temperature.is_throttling_async().await?;
    if is_throttling {
        println!("System is thermal throttling!");
    }
    
    Ok(())
}
```

## Finding Available Sensors

Systems may have different temperature sensors available. You can discover the available sensors:

```rust
use darwin_metrics::hardware::Temperature;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut temperature = Temperature::new();
    
    // List all available temperature sensors
    let sensors = temperature.list_sensors()?;
    
    println!("Available temperature sensors:");
    for (name, location) in sensors {
        println!("- {} ({:?})", name, location);
    }
    
    Ok(())
}
```

## Thermal Metrics

The `ThermalMetrics` struct provides a comprehensive snapshot of the system's thermal state:

```rust
pub struct ThermalMetrics {
    pub cpu_temperature: Option<f64>,      // CPU temperature in degrees Celsius
    pub gpu_temperature: Option<f64>,      // GPU temperature in degrees Celsius
    pub heatsink_temperature: Option<f64>, // Heatsink temperature in degrees Celsius
    pub ambient_temperature: Option<f64>,  // Ambient temperature in degrees Celsius
    pub battery_temperature: Option<f64>,  // Battery temperature in degrees Celsius
    pub is_throttling: bool,               // Whether the system is thermal throttling
    pub cpu_power: Option<f64>,            // CPU power consumption in watts
    pub fans: Vec<Fan>,                    // Information about all fans
}
```

## Implementation Details

The thermal module uses macOS's System Management Controller (SMC) to access temperature sensors and fan information. The internal implementation:

1. Communicates with the SMC via IOKit
2. Reads temperature sensor data from various system components
3. Retrieves fan speeds and calculates utilization percentages
4. Monitors CPU power consumption when available
5. Detects thermal throttling via SMC indicators or temperature-based heuristics

## Performance Considerations

- Temperature readings are cached based on the configured polling interval
- Auto-refresh can be disabled for performance-critical applications
- Async methods offload blocking I/O operations to a separate thread pool
- Fan information is updated alongside temperature data for efficiency

## Common Issues

- **Missing Sensors**: Not all Macs have the same temperature sensors; always check if values are present
- **Throttling Detection**: The primary method uses built-in SMC indicators, with temperature thresholds as a fallback
- **Battery Temperature**: Only available on MacBooks with a battery
- **Fan Information**: Systems with passive cooling may not have fan information available