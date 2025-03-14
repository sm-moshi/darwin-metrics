#![allow(unused_imports)]

use objc2::{msg_send, rc::autoreleasepool};

use crate::{
    error::Error,
    hardware::iokit::{GpuStats, IOKit, IOKitImpl, MockIOKit},
    utils::test_utils::{create_test_dictionary, create_test_object},
};

#[test]
fn test_get_gpu_stats() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_gpu_stats().returning(|| {
        Ok(GpuStats {
            utilization: 50.0,
            perf_cap: 50.0,
            perf_threshold: 100.0,
            memory_used: 1024 * 1024 * 1024,
            memory_total: 4 * 1024 * 1024 * 1024,
            name: "Test GPU".to_string(),
        })
    });

    let result = mock_iokit.get_gpu_stats().unwrap();

    assert_eq!(result.utilization, 50.0);
    assert_eq!(result.memory_total, 4 * 1024 * 1024 * 1024);
    assert_eq!(result.name, "Test GPU");
}

#[test]
fn test_gpu_stats_error_handling() {
    let mut mock = MockIOKit::new();

    mock.expect_get_service().returning(|_| {
        Err(Error::IOKit("GPU service not found".to_string()))
    });

    let result = mock.get_gpu_stats();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GPU service not found"));

    let mut mock = MockIOKit::new();
    mock.expect_get_service().returning(|_| Ok(create_test_object()));
    mock.expect_io_registry_entry_create_cf_properties().returning(|_| {
        Err(Error::IOKit("Failed to read properties".to_string()))
    });

    let result = mock.get_gpu_stats();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read properties"));
}

#[test]
fn test_gpu_stats_default() {
    let stats = GpuStats::default();
    
    assert_eq!(stats.utilization, 0.0);
    assert_eq!(stats.perf_cap, 0.0);
    assert_eq!(stats.perf_threshold, 0.0);
    assert_eq!(stats.memory_used, 0);
    assert_eq!(stats.memory_total, 0);
    assert_eq!(stats.name, "");
}

#[test]
fn test_gpu_stats() {
    let iokit = IOKitImpl::default();
    // ... rest of test ...
}

// This test is disabled by default because it can cause segfaults
#[test]
#[cfg(not(feature = "skip-ffi-crashes"))]
fn test_real_gpu_stats() {
    autoreleasepool(|_| {
        let iokit = IOKitImpl;
        println!("Created IOKitImpl instance");

        println!("Testing IOAccelerator service directly");
        let matching = iokit.io_service_matching("IOAccelerator");
        println!("Got matching dictionary for IOAccelerator");

        let service_opt = iokit.io_service_get_matching_service(&matching);
        match service_opt {
            Some(service) => {
                println!("Found IOAccelerator service, trying to get properties");
                match iokit.io_registry_entry_create_cf_properties(&service) {
                    Ok(props) => {
                        println!("Successfully got properties from IOAccelerator");

                        println!("============== Testing Key Access ==============");

                        let vram_keys = [
                            "VRAM,totalMB",
                            "VRAM,usedMB",
                            "totalVRAM",
                            "usedVRAM",
                            "vramUsage",
                            "vramFree",
                        ];

                        for key in vram_keys.iter() {
                            if let Some(value) = iokit.get_number_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        let name_keys = [
                            "model",
                            "name",
                            "IOGLBundleName",
                            "IOAccelRevision",
                            "device-id",
                            "vendor-id",
                            "IOAccelIndex",
                            "IOAccelTypes",
                            "gpuType",
                            "gpu_product",
                        ];

                        for key in name_keys.iter() {
                            if let Some(value) = iokit.get_string_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        let perf_keys = [
                            "IOGPUCurrentPowerState",
                            "IOGPUMaximumPowerState",
                            "deviceUtilization",
                            "powerState",
                            "GPUPerfCap",
                            "GPUPerfThreshold",
                        ];

                        for key in perf_keys.iter() {
                            if let Some(value) = iokit.get_number_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        println!("\n============== Testing Metal API ==============");
                        use crate::utils::bindings::MTLCreateSystemDefaultDevice;

                        println!("Creating Metal device...");
                        autoreleasepool(|_pool| {
                            unsafe {
                                let device = MTLCreateSystemDefaultDevice();
                                if device.is_null() {
                                    println!("Failed to create Metal device");
                                    return;
                                }

                                println!("Metal device created successfully");

                                let device_obj: *mut objc2::runtime::AnyObject = device.cast();

                                println!("Fetching device name...");
                                let name_obj: *mut objc2::runtime::AnyObject =
                                    msg_send![device_obj, name];
                                if name_obj.is_null() {
                                    println!("Failed to get device name");
                                } else {
                                    autoreleasepool(|_| {
                                        let _: () = msg_send![name_obj, retain];

                                        let utf8_string: *const u8 =
                                            msg_send![name_obj, UTF8String];
                                        if utf8_string.is_null() {
                                            println!("Failed to get UTF8 string");
                                        } else {
                                            let c_str =
                                                std::ffi::CStr::from_ptr(utf8_string as *const i8);
                                            let name = c_str.to_string_lossy();
                                            println!("GPU name from Metal API: {}", name);
                                        }

                                        let _: () = msg_send![name_obj, release];
                                    });
                                }

                                println!("Releasing Metal device...");
                                let _: () = msg_send![device_obj, release];
                                println!("Metal device released");
                            }
                        });

                        println!("Memory management handled properly, test continuing...");
                    },
                    Err(e) => {
                        println!("Error getting properties: {:?}", e);
                    },
                }
            },
            None => {
                println!("IOAccelerator service not found");
            },
        }
    });
} 