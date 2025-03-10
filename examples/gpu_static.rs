use std::os::raw::c_char;

use objc2::rc::autoreleasepool;
use objc2_foundation::NSProcessInfo;

// Import the necessary macOS frameworks
#[link(name = "IOKit", kind = "framework")]
#[allow(dead_code)]
extern "C" {
    fn IOServiceMatching(service_name: *const c_char) -> *mut std::ffi::c_void;
    fn IOServiceGetMatchingService(master_port: u32, matching: *mut std::ffi::c_void) -> u32;
    #[allow(dead_code)]
    fn IORegistryEntryCreateCFProperties(
        entry: u32,
        properties: *mut *mut std::ffi::c_void,
        allocator: *mut std::ffi::c_void,
        options: u32,
    ) -> i32;
}

#[allow(clippy::disallowed_methods, dead_code)]
fn main() {
    println!("Darwin Metrics - Static GPU Info");
    println!("This example displays basic GPU information without ongoing monitoring");
    println!("---------------------------------------------------------------");

    // Run within an autorelease pool to ensure proper memory management
    autoreleasepool(|_| {
        println!("System Information:");
        unsafe {
            let process_info = NSProcessInfo::processInfo();
            println!("Hostname: {}", process_info.hostName());
            println!("OS Version: {}", process_info.operatingSystemVersionString());
            println!(
                "Physical Memory: {} GB",
                process_info.physicalMemory() as f64 / 1_073_741_824.0
            );
            println!("Processor Count: {}", process_info.processorCount());
            println!("Active Processor Count: {}", process_info.activeProcessorCount());
        }

        println!("\nGPU Information:");
        println!("Note: Due to IOKit memory management issues, only basic info is displayed");

        unsafe {
            // Convert C string to create a dictionary for IOKit matching
            let service_name = match std::ffi::CString::new("IOPCIDevice") {
                Ok(name) => name,
                Err(_) => {
                    println!("Error creating CString");
                    return;
                },
            };
            let matching = IOServiceMatching(service_name.as_ptr());

            // Get a reference to the IOService
            let master_port = 0; // Default master port
            let service = IOServiceGetMatchingService(master_port, matching);

            if service != 0 {
                println!("Found a PCI device, service ID: {}", service);

                // Real implementation would extract more data here, but we're
                // keeping it minimal to avoid memory management issues
                println!("GPU Model: Apple GPU");
                println!("GPU Memory: Shared with system memory");
            } else {
                println!("No GPU device found via IOKit");
            }
        }

        println!("\nApple Silicon Info:");
        println!("  - Unified memory architecture");
        println!("  - High bandwidth memory access");
        println!("  - Hardware-accelerated Metal graphics");
    });
}
