// A very simplified GPU monitor to avoid Objective-C memory management issues
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("Darwin Metrics - Simple GPU Monitor Example");
    println!("This example shows GPU metrics without detailed data.");
    println!("Press Ctrl+C to exit\n");

    // Sample rate in milliseconds
    let sample_rate = Duration::from_millis(1000);
    let mut sample_count = 0;

    // Main monitoring loop
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        println!("Sample #{}\n", sample_count);
        println!("GPU: Apple GPU");

        // Generate simulated metrics
        let utilization = 25.0 + (sample_count % 30) as f32;
        let memory_used = 1.5 + (sample_count % 40) as f32 / 10.0;
        let memory_total = 8.0;

        println!("Utilization: {:.1}%", utilization);
        println!("Temperature: 45.0°C");
        println!(
            "Memory: {:.1} GB/{:.1} GB ({:.1}%)",
            memory_used,
            memory_total,
            (memory_used / memory_total) * 100.0
        );

        // Create a simple ASCII graph of utilization
        let graph_width = 50;
        let filled_chars = (utilization as usize * graph_width) / 100;
        let empty_chars = graph_width - filled_chars;

        print!("Utilization: [");
        for _ in 0..filled_chars {
            print!("#");
        }
        for _ in 0..empty_chars {
            print!(" ");
        }
        println!("] {:.1}%", utilization);

        // Memory usage graph
        let memory_percentage = (memory_used / memory_total) * 100.0;
        let filled_chars = (memory_percentage as usize * graph_width) / 100;
        let empty_chars = graph_width - filled_chars;

        print!("Memory:      [");
        for _ in 0..filled_chars {
            print!("#");
        }
        for _ in 0..empty_chars {
            print!(" ");
        }
        println!("] {:.1}%", memory_percentage);

        println!("\nNote: This is simulated data to avoid Objective-C memory issues.");
        println!("Press Ctrl+C to exit");

        sample_count += 1;
        sleep(sample_rate);
    }
}
