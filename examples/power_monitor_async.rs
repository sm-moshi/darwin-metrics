use darwin_metrics::power::{self, PowerConsumption, PowerState};
use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn display_power_info(info: &PowerConsumption) {
    println!("\nSystem Power Information (Async):");
    println!("-------------------------------");
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
        },
        PowerState::Charging => {
            if let Some(battery) = info.battery_percentage {
                println!("Battery level: {:.1}% (charging)", battery);
            }
        },
        PowerState::AC => {
            println!("Running on external power");
        },
        PowerState::Unknown => {
            println!("Power state unknown");
        },
    }

    if let Some(impact) = info.power_impact {
        println!("\nPower impact score: {:.2}", impact);
    }
}

#[tokio::main]
async fn main() {
    println!("Async power monitoring example (10 second duration)");
    println!("--------------------------------------------------");

    // Get the start time
    let start_time = Instant::now();

    // Run for 10 seconds
    while start_time.elapsed().as_secs() < 10 {
        match power::get_power_consumption_async().await {
            Ok(info) => {
                display_power_info(&info).await;
            },
            Err(e) => {
                eprintln!("Error getting power information: {:?}", e);
            },
        }

        // Sleep for 1 second between measurements
        sleep(Duration::from_secs(1)).await;
        println!("\n--- Time elapsed: {} seconds ---", start_time.elapsed().as_secs());
    }

    println!("\nAsync monitoring complete.");
}
