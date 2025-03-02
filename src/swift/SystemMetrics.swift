import DarwinMetrics
//!
import Foundation
import IOKit

// MARK: - FFI Data Structures

struct BatteryInfoFFI {
    let isPresent: Bool
    let isCharging: Bool
    let percentage: Double
    let timeRemaining: Int32
}

struct CPUInfoFFI {
    let cores: Int32
    let frequencyMHz: Double
}

struct MemoryInfoFFI {
    let totalGB: Double
    let usedGB: Double
    let freeGB: Double
}

// MARK: - Battery Information

func getBatteryInfo() -> BatteryInfo {
    let service = IOServiceGetMatchingService(
        kIOMainPortDefault, IOServiceMatching("AppleSmartBattery"))
    defer { IOObjectRelease(service) }

    guard service != 0 else {
        return BatteryInfo.new(isPresent: false, isCharging: false, percentage: 0, timeRemaining: 0)
    }

    var propertyList: Unmanaged<CFMutableDictionary>?
    let result = IORegistryEntryCreateCFProperties(service, &propertyList, kCFAllocatorDefault, 0)

    guard result == KERN_SUCCESS,
        let properties = propertyList?.takeRetainedValue() as? [String: Any]
    else {
        return BatteryInfo.new(isPresent: false, isCharging: false, percentage: 0, timeRemaining: 0)
    }

    let isCharging = properties["IsCharging"] as? Bool ?? false
    let currentCapacity = properties["CurrentCapacity"] as? Int ?? 0
    let maxCapacity = properties["MaxCapacity"] as? Int ?? 1
    let timeRemaining = properties["TimeRemaining"] as? Int ?? 0

    let percentage = Double(currentCapacity) / Double(maxCapacity) * 100.0

    return BatteryInfo.new(
        isPresent: true,
        isCharging: isCharging,
        percentage: percentage,
        timeRemaining: Int32(timeRemaining)
    )
}

// MARK: - CPU Information

func getCPUInfo() -> CPUInfo {
    var size = UInt32(0)
    var cores: Int32 = 0
    var mib = [CTL_HW, HW_NCPU]

    var sizeOfCores = MemoryLayout<Int32>.size
    sysctl(&mib, 2, &cores, &sizeOfCores, nil, 0)

    var cpuFreq: UInt64 = 0
    var sizeOfFreq = MemoryLayout<UInt64>.size
    sysctlbyname("hw.cpufrequency", &cpuFreq, &sizeOfFreq, nil, 0)

    return CPUInfo.new(
        cores: cores,
        frequencyMhz: Double(cpuFreq) / 1_000_000.0
    )
}

// MARK: - Memory Information

func getMemoryInfo() -> MemoryInfo {
    var stats = vm_statistics64()
    var size = mach_msg_type_number_t(
        MemoryLayout<vm_statistics64_data_t>.size / MemoryLayout<integer_t>.size)

    let result = withUnsafeMutablePointer(to: &stats) { pointer in
        pointer.withMemoryRebound(to: integer_t.self, capacity: Int(size)) { pointer in
            host_statistics64(mach_host_self(), HOST_VM_INFO64, pointer, &size)
        }
    }

    guard result == KERN_SUCCESS else {
        return MemoryInfo.new(totalGb: 0, usedGb: 0, freeGb: 0)
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

    return MemoryInfo.new(
        totalGb: total / bytesInGB,
        usedGb: used / bytesInGB,
        freeGb: free / bytesInGB
    )
}
