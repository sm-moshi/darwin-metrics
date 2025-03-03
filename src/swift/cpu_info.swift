import Foundation
import IOKit.ps
import IOKit.pwr_mgt
import darwin_metrics_swift_bridge  // Import the generated Swift bridge module

// MARK: - CPU Information Gathering

@_cdecl("getCPUInfo")
public func getCPUInfo() -> UnsafeMutablePointer<CPUInfoFFI> {
    // Create result structure
    let result = UnsafeMutablePointer<CPUInfoFFI>.allocate(capacity: 1)
    result.initialize(to: CPUInfoFFI())

    // Get basic CPU info
    let basicInfo = getCPUBasicInfo()
    result.pointee.physical_cores = basicInfo.physicalCores
    result.pointee.logical_cores = basicInfo.logicalCores
    result.pointee.performance_cores = basicInfo.performanceCores
    result.pointee.efficiency_cores = basicInfo.efficiencyCores
    result.pointee.model_name = basicInfo.modelName
    result.pointee.brand_string = basicInfo.brandString

    // Get CPU frequency
    if let freq = getCurrentCPUFrequency() {
        result.pointee.frequency_mhz = freq
    }

    // Get CPU load information
    let loadInfo = getCPULoadInfo()
    result.pointee.total_load_user = loadInfo.userLoad
    result.pointee.total_load_system = loadInfo.systemLoad
    result.pointee.total_load_idle = loadInfo.idleLoad

    // Get temperature and power info
    let smc = SMC.shared

    // Get CPU package temperature
    if let temp = smc.getValue("TC0P") {
        result.pointee.package_temperature = temp
    }

    // Get power consumption
    if let currentPower = smc.getValue("PC0C") {
        result.pointee.current_watts = currentPower
    }

    if let maxPower = smc.getValue("PC0M") {
        result.pointee.max_watts = maxPower
    }

    // Package power includes all CPU components
    if let packagePower = smc.getValue("PC0T") {
        result.pointee.package_watts = packagePower
    }

    return result
}

// MARK: - Helper Functions

private struct CPUBasicInfo {
    var physicalCores: Int32
    var logicalCores: Int32
    var performanceCores: Int32
    var efficiencyCores: Int32
    var modelName: String
    var brandString: String
}

private func getCPUBasicInfo() -> CPUBasicInfo {
    var size = 0
    var result = CPUBasicInfo(
        physicalCores: 0,
        logicalCores: 0,
        performanceCores: 0,
        efficiencyCores: 0,
        modelName: "",
        brandString: ""
    )

    // Get brand string
    sysctlbyname("machdep.cpu.brand_string", nil, &size, nil, 0)
    var buffer = [CChar](repeating: 0, count: size)
    sysctlbyname("machdep.cpu.brand_string", &buffer, &size, nil, 0)
    result.brandString = String(cString: buffer)

    // Get model name
    if let modelName = getMacModelIdentifier() {
        result.modelName = modelName
    }

    // Get core counts
    var count: Int32 = 0

    // Total physical cores
    var length = MemoryLayout<Int32>.size
    sysctlbyname("hw.physicalcpu", &count, &length, nil, 0)
    result.physicalCores = count

    // Total logical cores
    sysctlbyname("hw.logicalcpu", &count, &length, nil, 0)
    result.logicalCores = count

    // Performance cores (perflevel0)
    sysctlbyname("hw.perflevel0.logicalcpu", &count, &length, nil, 0)
    result.performanceCores = count

    // Efficiency cores (perflevel1)
    sysctlbyname("hw.perflevel1.logicalcpu", &count, &length, nil, 0)
    result.efficiencyCores = count

    return result
}

private func getCurrentCPUFrequency() -> Double? {
    var size = MemoryLayout<Int32>.size
    var speed: Int32 = 0
    let result = sysctlbyname("hw.cpufrequency", &speed, &size, nil, 0)
    if result == 0 {
        return Double(speed) / 1_000_000.0  // Convert Hz to MHz
    }
    return nil
}

private struct CPULoadInfo {
    var userLoad: Double
    var systemLoad: Double
    var idleLoad: Double
}

private func getCPULoadInfo() -> CPULoadInfo {
    var cpuLoad = host_cpu_load_info()
    var count = mach_msg_type_number_t(
        MemoryLayout<host_cpu_load_info>.size / MemoryLayout<integer_t>.size)
    let result = withUnsafeMutablePointer(to: &cpuLoad) {
        $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) {
            host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, $0, &count)
        }
    }

    if result == KERN_SUCCESS {
        let total = Double(
            cpuLoad.cpu_ticks.0 + cpuLoad.cpu_ticks.1 + cpuLoad.cpu_ticks.2 + cpuLoad.cpu_ticks.3)
        return CPULoadInfo(
            userLoad: Double(cpuLoad.cpu_ticks.0) / total * 100.0,
            systemLoad: Double(cpuLoad.cpu_ticks.1) / total * 100.0,
            idleLoad: Double(cpuLoad.cpu_ticks.2) / total * 100.0
        )
    }

    return CPULoadInfo(userLoad: 0, systemLoad: 0, idleLoad: 0)
}

private func getCPUCoreInfo(performanceCoreCount: Int32) -> [CPUCoreInfoFFI] {
    var result: [CPUCoreInfoFFI] = []
    let smc = SMC.shared

    // Get per-core information
    for i in 0..<ProcessInfo.processInfo.processorCount {
        var coreInfo = CPUCoreInfoFFI()

        // Determine if this is a performance core
        coreInfo.is_performance_core = Int32(i) < performanceCoreCount

        // Get core temperature from SMC
        let tempKey = String(format: "TC%dC", i)
        if let temp = smc.getValue(tempKey) {
            coreInfo.temperature = temp
        }

        // Get core load
        if let loadInfo = getPerCoreLoadInfo(coreIndex: i) {
            coreInfo.user_load = loadInfo.user
            coreInfo.system_load = loadInfo.system
            coreInfo.idle_load = loadInfo.idle
        }

        result.append(coreInfo)
    }

    return result
}

private func getCPUTempAndPower() -> (temperature: Double, powerInfo: CPUPowerInfoFFI) {
    let smc = SMC.shared
    var powerInfo = CPUPowerInfoFFI()
    var packageTemp: Double = 0

    // Get CPU package temperature
    if let temp = smc.getValue("TC0P") {
        packageTemp = temp
    }

    // Get power consumption
    if let currentPower = smc.getValue("PC0C") {
        powerInfo.current_watts = currentPower
    }

    if let maxPower = smc.getValue("PC0M") {
        powerInfo.max_watts = maxPower
    }

    // Package power includes all CPU components
    if let packagePower = smc.getValue("PC0T") {
        powerInfo.package_watts = packagePower
    }

    return (packageTemp, powerInfo)
}

private func getMacModelIdentifier() -> String? {
    var size = 0
    sysctlbyname("hw.model", nil, &size, nil, 0)
    var buffer = [CChar](repeating: 0, count: size)
    let result = sysctlbyname("hw.model", &buffer, &size, nil, 0)
    if result == 0 {
        return String(cString: buffer)
    }
    return nil
}

private func getPerCoreLoadInfo(coreIndex: Int) -> (user: Double, system: Double, idle: Double)? {
    // Implementation requires processor_info API
    // This is a placeholder - actual implementation would use processor_info
    // to get per-core CPU load
    return (user: 0, system: 0, idle: 0)
}
