use darwin_metrics::power::{self, PowerConsumption, PowerState};
use std::{thread, time::Duration};

fn display_power_info(info: &PowerConsumption) {
    println!("\nSystem Power Information:");
    println!("------------------------");
    println!("Package power: {:.2} W", info.package);
    println!("CPU cores power: {:.2} W", info.cores);

    if let Some(gpu) = info.gpu {
        println!("GPU power: {:.2} W", gpu);
    }

    if let Some(memory) = info.dram {
        println!("Memory power: {:.2} W", memory);
    }

    if let Some(neural) = info.neural_engine {
        println!("Neural Engine: {:.2} W", neural);
    }

    // Power state information
    println!("\nPower State: {:?}", info.power_state);
    match info.power_state {
        PowerState::Battery => {
            if let Some(battery) = info.battery_percentage {
                println!("Battery level: {:.1}%", battery);
            }
            println!("Running on battery power");
        },
        PowerState::Charging => {
            if let Some(battery) = info.battery_percentage {
                println!("Battery level: {:.1}% (charging)", battery);
            }
            println!("Charging from external power");
        },
        PowerState::AC => {
            println!("Running on external power");
        },
        PowerState::Unknown => {
            println!("Power state unknown");
        },
    }

    if let Some(impact) = info.power_impact {
        println!("Power impact score: {:.2}", impact);
        let rating = if impact < 5.0 {
            "Low power usage"
        } else if impact < 15.0 {
            "Moderate power usage"
        } else {
            "High power usage"
        };
        println!("Power rating: {}", rating);
    }
}

fn main() {
    println!("Power monitoring example (10 second duration)");
    println!("---------------------------------------------");

    // Get the start time
    let start_time = std::time::Instant::now();

    // Run for 10 seconds
    while start_time.elapsed().as_secs() < 10 {
        match power::get_power_consumption() {
            Ok(info) => {
                display_power_info(&info);
            },
            Err(e) => {
                eprintln!("Error getting power information: {:?}", e);
            },
        }

        // Sleep for 1 second between measurements
        thread::sleep(Duration::from_secs(1));
        println!("\n--- Time elapsed: {} seconds ---", start_time.elapsed().as_secs());
    }

    println!("\nMonitoring complete.");
}
