//!
//! Swift module for macOS system metrics
//!
//! This module provides Swift bindings for accessing system metrics
//! using the `IOKit` framework. It includes functions to retrieve
//! battery information, CPU usage, memory statistics, and more.
//!
import Foundation
import IOKit
import os.log

// MARK: - Error Handling

enum SystemMetricsError: Error {
    case serviceNotFound
    case invalidData
    case systemError(String)
}

// MARK: - FFI Data Structures

@objc
class BatteryInfoFFI: NSObject {
    @objc private(set) var isPresent: Bool
    @objc private(set) var isCharging: Bool
    @objc private(set) var percentage: Double
    @objc private(set) var timeRemaining: Int32

    init(isPresent: Bool, isCharging: Bool, percentage: Double, timeRemaining: Int32) {
        self.isPresent = isPresent
        self.isCharging = isCharging
        self.percentage = percentage
        self.timeRemaining = timeRemaining
        super.init()
    }
}

@objc
class CPUInfoFFI: NSObject {
    @objc private(set) var cores: Int32
    @objc private(set) var frequencyMHz: Double

    init(cores: Int32, frequencyMHz: Double) {
        self.cores = cores
        self.frequencyMHz = frequencyMHz
        super.init()
    }
}

@objc
class MemoryInfoFFI: NSObject {
    @objc private(set) var totalGB: Double
    @objc private(set) var usedGB: Double
    @objc private(set) var freeGB: Double

    init(totalGB: Double, usedGB: Double, freeGB: Double) {
        self.totalGB = totalGB
        self.usedGB = usedGB
        self.freeGB = freeGB
        super.init()
    }
}

// MARK: - Battery Information

private let logger = Logger(subsystem: "com.darwin-metrics", category: "BatteryInfo")

@_cdecl("get_battery_info")
func getBatteryInfoFFI() -> UnsafeMutableRawPointer {
    do {
        let info = try getBatteryInfo()
        return Unmanaged.passRetained(info).toOpaque()
    } catch {
        logger.error("Failed to get battery info: \(error.localizedDescription)")
        return Unmanaged.passRetained(
            BatteryInfoFFI(isPresent: false, isCharging: false, percentage: 0, timeRemaining: 0)
        ).toOpaque()
    }
}

func getBatteryInfo() throws -> BatteryInfoFFI {
    let service = IOServiceGetMatchingService(
        kIOMainPortDefault,
        IOServiceMatching("AppleSmartBattery")
    )

    guard service != 0 else {
        logger.error("Battery service not found")
        throw SystemMetricsError.serviceNotFound
    }

    defer { IOObjectRelease(service) }

    var propertyList: Unmanaged<CFMutableDictionary>?
    let result = IORegistryEntryCreateCFProperties(
        service,
        &propertyList,
        kCFAllocatorDefault,
        0
    )

    guard result == KERN_SUCCESS,
        let properties = propertyList?.takeRetainedValue() as? [String: Any]
    else {
        logger.error("Failed to get battery properties")
        throw SystemMetricsError.invalidData
    }

    let isCharging = properties["IsCharging"] as? Bool ?? false
    let currentCapacity = properties["CurrentCapacity"] as? Int ?? 0
    let maxCapacity = properties["MaxCapacity"] as? Int ?? 1
    let timeRemaining = properties["TimeRemaining"] as? Int ?? 0

    let percentage = Double(currentCapacity) / Double(maxCapacity) * 100.0

    return BatteryInfoFFI(
        isPresent: true,
        isCharging: isCharging,
        percentage: percentage,
        timeRemaining: Int32(timeRemaining)
    )
}

// MARK: - CPU Information

@_cdecl("get_cpu_info")
func getCPUInfoFFI() -> UnsafeMutableRawPointer {
    let info = getCPUInfo()
    return Unmanaged.passRetained(info).toOpaque()
}

func getCPUInfo() -> CPUInfoFFI {
    var size = UInt32(0)
    var cores: Int32 = 0
    var mib = [CTL_HW, HW_NCPU]

    var sizeOfCores = MemoryLayout<Int32>.size
    sysctl(&mib, 2, &cores, &sizeOfCores, nil, 0)

    var cpuFreq: UInt64 = 0
    var sizeOfFreq = MemoryLayout<UInt64>.size
    sysctlbyname("hw.cpufrequency", &cpuFreq, &sizeOfFreq, nil, 0)

    return CPUInfoFFI(
        cores: cores,
        frequencyMHz: Double(cpuFreq) / 1_000_000.0
    )
}

// MARK: - Memory Information

@_cdecl("get_memory_info")
func getMemoryInfoFFI() -> UnsafeMutableRawPointer {
    let info = getMemoryInfo()
    return Unmanaged.passRetained(info).toOpaque()
}

func getMemoryInfo() -> MemoryInfoFFI {
    var stats = vm_statistics64()
    var size = mach_msg_type_number_t(
        MemoryLayout<vm_statistics64_data_t>.size / MemoryLayout<integer_t>.size)

    let result = withUnsafeMutablePointer(to: &stats) { pointer in
        pointer.withMemoryRebound(to: integer_t.self, capacity: Int(size)) { pointer in
            host_statistics64(mach_host_self(), HOST_VM_INFO64, pointer, &size)
        }
    }

    guard result == KERN_SUCCESS else {
        return MemoryInfoFFI(
            totalGB: 0,
            usedGB: 0,
            freeGB: 0
        )
    }

    let pageSize = UInt64(vm_kernel_page_size)
    let total = Double(ProcessInfo.processInfo.physicalMemory)

    // Convert page counts to bytes, then to GB
    let used = Double(
        UInt64(stats.active_count + stats.inactive_count + stats.wire_count) * pageSize
    )
    let free = Double(UInt64(stats.free_count) * pageSize)

    // Convert to GB
    let bytesInGB = 1024.0 * 1024.0 * 1024.0

    return MemoryInfoFFI(
        totalGB: total / bytesInGB,
        usedGB: used / bytesInGB,
        freeGB: free / bytesInGB
    )
}
