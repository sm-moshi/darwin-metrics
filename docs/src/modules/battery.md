# Battery Module

The Battery module provides functionality to monitor and manage battery information on macOS
systems. It interfaces with the system's IOKit framework to retrieve real-time battery statistics
and status information.

## Features

- Real-time battery status monitoring
- Power source detection (Battery/AC/Unknown)
- Battery health assessment
- Temperature monitoring
- Cycle count tracking
- Charging status
- Remaining time estimation

## Core Types

### PowerSource

An enumeration representing the current power source:

```rust
pub enum PowerSource {
    Battery,  // Running on battery power
    AC,      // Connected to AC power
    Unknown  // Power source cannot be determined
}
```

### Battery

The main struct containing battery information:

```rust
pub struct Battery {
    pub is_present: bool,         // Whether a battery is installed
    pub is_charging: bool,        // Current charging status
    pub percentage: f64,          // Current charge percentage (0-100)
    pub time_remaining: Duration, // Estimated time remaining
    pub power_source: PowerSource, // Current power source
    pub cycle_count: u32,         // Battery cycle count
    pub health_percentage: f64,   // Battery health percentage
    pub temperature: f64,         // Battery temperature in Celsius
}
```

## Main Functions

### Creation and Updates

- `Battery::new()` - Creates a new Battery instance with current system information
- `Battery::update()` - Updates all battery metrics with latest system values
- `Battery::get_info()` - Retrieves current battery information

### Status Checks

- `is_critical()` - Checks if battery level is critically low (<10%)
- `is_low()` - Checks if battery level is low (<20%)
- `is_health_poor()` - Checks if battery health is poor (<80%)
- `has_high_cycle_count()` - Checks if battery has high cycle count (>1000)
- `is_temperature_critical()` - Checks if battery temperature is outside safe range (-10°C to 40°C)

### Display Functions

- `time_remaining_display()` - Formats remaining time as a human-readable string
- `power_source_display()` - Returns a string representation of the current power source

## Example Usage

```rust
use darwin_metrics::Battery;

fn main() -> Result<()> {
    // Create a new Battery instance
    let mut battery = Battery::new()?;
    
    // Print battery status
    println!("Battery Status:");
    println!("Charge: {}%", battery.percentage);
    println!("Power Source: {}", battery.power_source_display());
    println!("Time Remaining: {}", battery.time_remaining_display());
    
    // Check battery health
    if battery.is_health_poor() {
        println!("Warning: Battery health is poor!");
    }
    
    // Update battery information
    battery.update()?;
    
    Ok(())
}
```

## Implementation Details

The module uses macOS's IOKit framework to interact with the system's power management. It
specifically interfaces with the "AppleSmartBattery" service to retrieve battery information.

Key system properties monitored:

- BatteryInstalled
- IsCharging
- CurrentCapacity
- MaxCapacity
- DesignCapacity
- CycleCount
- Temperature
- TimeRemaining
- ExternalConnected

## Error Handling

The module uses the crate's error handling system, returning `Result<T>` types for operations that
might fail. Common error cases include:

- Battery service not found
- Failed to retrieve battery properties
- System API errors

## Performance Considerations

- Battery information is retrieved on-demand
- Updates are not automatic - call `update()` to refresh values
- Temperature values are converted from raw values (1/100°C) to Celsius
- Time remaining is converted from minutes to Duration

## Thread Safety

The Battery struct implements `Send` and `Sync`, making it safe to use across thread boundaries. The
underlying IOKit interactions are handled in a thread-safe manner.
