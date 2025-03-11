use std::collections::HashSet;
use std::ffi::c_char;

use crate::error::Error;
use crate::utils::bindings::{
    address_family, extern_proc, extract_proc_name, get_network_stats_native, getloadavg,
    if_data64, if_flags, is_system_process, kinfo_proc, proc_info, proc_state, reachability_flags,
    smc_key_from_chars, sysctl_constants, timeval, MTLCreateSystemDefaultDevice, MTLDeviceRef,
    Statfs, SMC_KEY_AMBIENT_TEMP, SMC_KEY_BATTERY_TEMP, SMC_KEY_CPU_POWER, SMC_KEY_CPU_TEMP,
    SMC_KEY_CPU_THROTTLE, SMC_KEY_FAN_NUM, SMC_KEY_GPU_TEMP,
};

#[test]
fn test_extract_proc_name() {
    // Create a kinfo_proc structure with a process name
    let mut proc_info = kinfo_proc {
        kp_proc: proc_info { p_flag: 0, p_pid: 123, p_ppid: 1, p_stat: 0 },
        kp_eproc: extern_proc { p_starttime: timeval { tv_sec: 0, tv_usec: 0 }, p_comm: [0; 16] },
    };

    // Set a process name
    let test_name = b"test_process\0";
    for (i, &byte) in test_name.iter().enumerate() {
        if i < proc_info.kp_eproc.p_comm.len() {
            proc_info.kp_eproc.p_comm[i] = byte;
        }
    }

    // Extract the name
    let extracted_name = extract_proc_name(&proc_info);
    assert_eq!(extracted_name, "test_process");
}

#[test]
fn test_extract_proc_name_no_null_terminator() {
    // Create a kinfo_proc structure with a process name that fills the entire buffer
    let mut proc_info = kinfo_proc {
        kp_proc: proc_info { p_flag: 0, p_pid: 123, p_ppid: 1, p_stat: 0 },
        kp_eproc: extern_proc { p_starttime: timeval { tv_sec: 0, tv_usec: 0 }, p_comm: [0; 16] },
    };

    // Set a process name that fills the entire buffer (no null terminator)
    let test_name = b"sixteencharname"; // Exactly 16 characters
    for (i, &byte) in test_name.iter().enumerate() {
        if i < proc_info.kp_eproc.p_comm.len() {
            proc_info.kp_eproc.p_comm[i] = byte;
        }
    }

    // Extract the name
    let extracted_name = extract_proc_name(&proc_info);
    assert_eq!(extracted_name, "sixteencharname");
}

#[test]
fn test_extract_proc_name_empty() {
    // Create a kinfo_proc structure with an empty process name
    let proc_info = kinfo_proc {
        kp_proc: proc_info { p_flag: 0, p_pid: 123, p_ppid: 1, p_stat: 0 },
        kp_eproc: extern_proc { p_starttime: timeval { tv_sec: 0, tv_usec: 0 }, p_comm: [0; 16] },
    };

    // Extract the name (should be empty)
    let extracted_name = extract_proc_name(&proc_info);
    assert_eq!(extracted_name, "");
}

#[test]
fn test_extract_proc_name_non_ascii() {
    // Create a kinfo_proc structure with a non-ASCII process name
    let mut proc_info = kinfo_proc {
        kp_proc: proc_info { p_flag: 0, p_pid: 123, p_ppid: 1, p_stat: 0 },
        kp_eproc: extern_proc { p_starttime: timeval { tv_sec: 0, tv_usec: 0 }, p_comm: [0; 16] },
    };

    // Set a process name with non-ASCII characters Using UTF-8 bytes for "café" (with an accent)
    let test_name = b"caf\xC3\xA9\0";
    for (i, &byte) in test_name.iter().enumerate() {
        if i < proc_info.kp_eproc.p_comm.len() {
            proc_info.kp_eproc.p_comm[i] = byte;
        }
    }

    // Extract the name
    let extracted_name = extract_proc_name(&proc_info);
    assert_eq!(extracted_name, "café");
}

#[test]
fn test_is_system_process() {
    // Test system processes
    assert!(is_system_process(1, "launchd"));
    assert!(is_system_process(999, "random_system_process"));
    assert!(is_system_process(1234, "com.apple.service"));
    assert!(is_system_process(5000, "kernel_task"));
    assert!(is_system_process(5000, "WindowServer"));

    // Test non-system processes
    assert!(!is_system_process(1000, "user_app"));
    assert!(!is_system_process(1234, "firefox"));
    assert!(!is_system_process(5000, "chrome"));
}

#[test]
fn test_is_system_process_edge_cases() {
    // Test edge cases
    assert!(is_system_process(0, "init")); // PID 0
    assert!(is_system_process(1, "non_system_name")); // System PID but non-system name
    assert!(!is_system_process(1000, "")); // Empty name
    assert!(is_system_process(999, "")); // System PID with empty name

    // Test other system process names
    assert!(is_system_process(5000, "systemstats"));
    assert!(is_system_process(5000, "logd"));
    assert!(is_system_process(5000, "syslogd"));
}

#[test]
fn test_smc_key_from_chars() {
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
    let result = smc_key_from_chars(key);

    // Calculate the expected value: ('T' << 24) | ('C' << 16) | ('0' << 8) | 'P'
    let expected = (b'T' as u32) << 24 | (b'C' as u32) << 16 | (b'0' as u32) << 8 | (b'P' as u32);

    assert_eq!(result, expected);
}

#[test]
fn test_smc_key_from_chars_predefined_keys() {
    // Test predefined SMC keys from the bindings
    let cpu_temp_key = smc_key_from_chars(SMC_KEY_CPU_TEMP);
    let gpu_temp_key = smc_key_from_chars(SMC_KEY_GPU_TEMP);
    let fan_num_key = smc_key_from_chars(SMC_KEY_FAN_NUM);
    let ambient_temp_key = smc_key_from_chars(SMC_KEY_AMBIENT_TEMP);
    let battery_temp_key = smc_key_from_chars(SMC_KEY_BATTERY_TEMP);
    let cpu_power_key = smc_key_from_chars(SMC_KEY_CPU_POWER);
    let cpu_throttle_key = smc_key_from_chars(SMC_KEY_CPU_THROTTLE);

    // Verify each key is unique
    let keys = [
        cpu_temp_key,
        gpu_temp_key,
        fan_num_key,
        ambient_temp_key,
        battery_temp_key,
        cpu_power_key,
        cpu_throttle_key,
    ];

    // Check that all keys are unique
    let unique_keys: HashSet<_> = keys.iter().collect();
    assert_eq!(keys.len(), unique_keys.len());
}

#[test]
fn test_smc_key_from_chars_zero() {
    let key = [0 as c_char, 0 as c_char, 0 as c_char, 0 as c_char];
    let result = smc_key_from_chars(key);
    assert_eq!(result, 0);
}

#[test]
fn test_address_family_constants() {
    // Test that address family constants have the expected values
    assert_eq!(address_family::AF_UNSPEC, 0);
    assert_eq!(address_family::AF_INET, 2);
    assert_eq!(address_family::AF_INET6, 30);
    assert_eq!(address_family::AF_LINK, 18);
}

#[test]
fn test_if_flags_constants() {
    // Test that interface flags constants have the expected values
    assert_eq!(if_flags::IFF_UP, 0x1);
    assert_eq!(if_flags::IFF_BROADCAST, 0x2);
    assert_eq!(if_flags::IFF_DEBUG, 0x4);
    assert_eq!(if_flags::IFF_LOOPBACK, 0x8);
    assert_eq!(if_flags::IFF_POINTOPOINT, 0x10);
    assert_eq!(if_flags::IFF_RUNNING, 0x40);
    assert_eq!(if_flags::IFF_NOARP, 0x80);
    assert_eq!(if_flags::IFF_PROMISC, 0x100);
    assert_eq!(if_flags::IFF_ALLMULTI, 0x200);
    assert_eq!(if_flags::IFF_MULTICAST, 0x8000);
    assert_eq!(if_flags::IFF_WIRELESS, 0x20);
}

#[test]
fn test_sysctl_constants() {
    // Test sysctl constants
    assert_eq!(sysctl_constants::CTL_KERN, 1);
    assert_eq!(sysctl_constants::CTL_HW, 6);
    assert_eq!(sysctl_constants::CTL_VM, 2);

    assert_eq!(sysctl_constants::KERN_PROC, 14);
    assert_eq!(sysctl_constants::KERN_PROC_ALL, 0);
    assert_eq!(sysctl_constants::KERN_PROC_PID, 1);
    assert_eq!(sysctl_constants::KERN_PROC_PGRP, 2);
    assert_eq!(sysctl_constants::KERN_PROC_TTY, 3);
    assert_eq!(sysctl_constants::KERN_PROC_UID, 4);
    assert_eq!(sysctl_constants::KERN_PROC_RUID, 5);

    assert_eq!(sysctl_constants::HW_MACHINE, 1);
    assert_eq!(sysctl_constants::HW_MEMSIZE, 24);

    assert_eq!(sysctl_constants::VM_SWAPUSAGE, 5);
}

#[test]
fn test_proc_state_constants() {
    // Test process state constants
    assert_eq!(proc_state::SIDL, 1);
    assert_eq!(proc_state::SRUN, 2);
    assert_eq!(proc_state::SSLEEP, 3);
    assert_eq!(proc_state::SSTOP, 4);
    assert_eq!(proc_state::SZOMB, 5);
}

#[test]
fn test_reachability_flags_constants() {
    // Test network reachability flags
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsTransientConnection, 1 << 0);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsReachable, 1 << 1);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsConnectionRequired, 1 << 2);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsConnectionOnTraffic, 1 << 3);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsInterventionRequired, 1 << 4);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsConnectionOnDemand, 1 << 5);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsIsLocalAddress, 1 << 16);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsIsDirect, 1 << 17);
    assert_eq!(reachability_flags::kSCNetworkReachabilityFlagsIsWWAN, 1 << 18);
}

#[test]
fn test_statfs_struct_size() {
    // Test that the Statfs struct has the expected size
    let statfs_size = std::mem::size_of::<Statfs>();

    // The exact size may vary by platform, but we can at least verify it's not zero and is large enough to hold all the
    // fields
    assert!(statfs_size > 0);

    // Statfs should be large enough to hold at least the basic fields plus the arrays
    let min_expected_size = std::mem::size_of::<u32>() +    // f_bsize
        std::mem::size_of::<i32>() +    // f_iosize
        std::mem::size_of::<u64>() * 5 + // f_blocks, f_bfree, f_bavail, f_files, f_ffree
        std::mem::size_of::<[i32; 2]>() + // f_fsid
        std::mem::size_of::<u32>() * 4 + // f_owner, f_type, f_flags, f_fssubtype
        std::mem::size_of::<[c_char; 16]>() + // f_fstypename
        std::mem::size_of::<[c_char; 1024]>() * 2 + // f_mntonname, f_mntfromname
        std::mem::size_of::<[u32; 8]>(); // f_reserved

    assert!(statfs_size >= min_expected_size);
}

#[test]
fn test_statfs_debug_clone() {
    // Test that Statfs implements Debug and Clone
    let statfs = Statfs {
        f_bsize: 4096,
        f_iosize: 1024,
        f_blocks: 1000000,
        f_bfree: 500000,
        f_bavail: 400000,
        f_files: 10000,
        f_ffree: 5000,
        f_fsid: [1, 2],
        f_owner: 0,
        f_type: 1,
        f_flags: 0,
        f_fssubtype: 0,
        f_fstypename: [0; 16],
        f_mntonname: [0; 1024],
        f_mntfromname: [0; 1024],
        f_reserved: [0; 8],
    };

    // Test Debug implementation
    let debug_str = format!("{:?}", statfs);
    assert!(debug_str.contains("Statfs"));

    // Test Copy implementation (using direct assignment instead of clone)
    let statfs_clone = statfs;
    assert_eq!(statfs.f_bsize, statfs_clone.f_bsize);
    assert_eq!(statfs.f_blocks, statfs_clone.f_blocks);
    assert_eq!(statfs.f_bfree, statfs_clone.f_bfree);
}

#[test]
fn test_metal_device_ref_type() {
    // Test that MTLDeviceRef is defined as a pointer type
    let null_device: MTLDeviceRef = std::ptr::null_mut();
    assert!(null_device.is_null());

    // We can't easily test MTLCreateSystemDefaultDevice without mocking, but we can at least verify the function is
    // linked correctly by calling it and checking if it returns a non-null pointer on systems with Metal support or
    // null on systems without Metal.
    //
    // Note: This is a real system call, but it's safe to make and doesn't modify anything.
    let device = unsafe { MTLCreateSystemDefaultDevice() };

    // We don't assert anything about the result since it depends on the system, but we've at least exercised the
    // function binding
    println!("MTLCreateSystemDefaultDevice returned: {:?}", device);
}

#[test]
fn test_getloadavg_binding() {
    // Test the getloadavg binding
    let mut loads = [0.0, 0.0, 0.0];

    // Call getloadavg to get the system load averages
    let result = unsafe { getloadavg(loads.as_mut_ptr(), loads.len() as i32) };

    // The function should return the number of samples retrieved (3) or -1 on error
    assert!(result == 3 || result == -1);

    if result == 3 {
        // If successful, the load averages should be non-negative
        assert!(loads[0] >= 0.0);
        assert!(loads[1] >= 0.0);
        assert!(loads[2] >= 0.0);

        // The 5-minute and 15-minute averages should be available
        println!("Load averages: 1min={}, 5min={}, 15min={}", loads[0], loads[1], loads[2]);
    }
}

// Mock for sysctlbyname to test get_network_stats_native This is a more advanced test that requires mocking system
// calls We'll use conditional compilation to only include it when running tests
#[cfg(test)]
mod mock_tests {
    use super::*;

    // Test get_network_stats_native with a mock implementation This test is more complex and would require mocking the
    // sysctlbyname function For now, we'll just test the error case with an invalid interface name
    #[test]
    fn test_get_network_stats_native_invalid_interface() {
        // Test with an interface name that contains a null byte, which should fail
        let result = get_network_stats_native("invalid\0interface");
        assert!(result.is_err());

        if let Err(Error::Network(msg)) = result {
            assert!(msg.contains("Failed to create sysctlbyname key"));
        } else {
            panic!("Expected Error::Network");
        }
    }

    #[test]
    fn test_get_network_stats_native_empty_interface() {
        // Test with an empty interface name
        let result = get_network_stats_native("");

        // This should either return an error or succeed with zeroed stats depending on the system
        if let Ok(stats) = result {
            // If it succeeds, the stats should be valid
            println!("Empty interface stats: rx={}, tx={}", stats.ifi_ibytes, stats.ifi_obytes);
        } else if let Err(Error::Network(msg)) = result {
            // If it fails, it should be a network error
            println!("Empty interface error: {}", msg);
        } else {
            panic!("Unexpected error type");
        }
    }

    // Test if_data64 struct initialization
    #[test]
    fn test_if_data64_initialization() {
        // Create a zeroed if_data64 struct
        let data: if_data64 = unsafe { std::mem::zeroed() };

        // Verify that all numeric fields are zero
        assert_eq!(data.ifi_ibytes, 0);
        assert_eq!(data.ifi_obytes, 0);
        assert_eq!(data.ifi_ipackets, 0);
        assert_eq!(data.ifi_opackets, 0);
        assert_eq!(data.ifi_baudrate, 0);

        // Test that we can create and manipulate the struct
        let mut data = if_data64 {
            ifi_type: 1,
            ifi_physical: 2,
            ifi_addrlen: 6,
            ifi_hdrlen: 14,
            ifi_recvquota: 0,
            ifi_xmitquota: 0,
            ifi_unused1: 0,
            ifi_mtu: 1500,
            ifi_metric: 0,
            ifi_baudrate: 1000000000,
            ifi_ipackets: 1000,
            ifi_ierrors: 0,
            ifi_opackets: 500,
            ifi_oerrors: 0,
            ifi_collisions: 0,
            ifi_ibytes: 1000000,
            ifi_obytes: 500000,
            ifi_imcasts: 0,
            ifi_omcasts: 0,
            ifi_iqdrops: 0,
            ifi_noproto: 0,
            ifi_recvtiming: 0,
            ifi_xmittiming: 0,
            ifi_lastchange: timeval { tv_sec: 0, tv_usec: 0 },
        };

        // Modify some fields
        data.ifi_ibytes += 1000;
        data.ifi_obytes += 500;

        // Verify the modifications
        assert_eq!(data.ifi_ibytes, 1001000);
        assert_eq!(data.ifi_obytes, 500500);
    }
}
