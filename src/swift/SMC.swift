import Foundation
import IOKit

/// A class for interacting with the System Management Controller (SMC)
public class SMC {
    /// Shared instance for accessing SMC
    public static let shared = SMC()

    private var connection: io_connect_t = 0

    private init() {
        let matchingDictionary = IOServiceMatching("AppleSMC")
        let service = IOServiceGetMatchingService(kIOMainPortDefault, matchingDictionary)

        if service != 0 {
            let result = IOServiceOpen(service, mach_task_self_, 0, &connection)
            if result != kIOReturnSuccess {
                print("Failed to open SMC connection")
            }
            IOObjectRelease(service)
        }
    }

    deinit {
        if connection != 0 {
            IOServiceClose(connection)
        }
    }

    /// Get a sensor value from SMC
    /// - Parameter key: The SMC key to read (e.g., "TC0P" for CPU temperature)
    /// - Returns: The sensor value as a Double, or nil if the read failed
    public func getValue(_ key: String) -> Double? {
        guard connection != 0 else { return nil }

        var inputStruct = SMCKeyData_t()
        var outputStruct = SMCKeyData_t()
        var outputSize = MemoryLayout<SMCKeyData_t>.size

        inputStruct.key = stringToKey(key)
        inputStruct.data8 = SMC_CMD_READ_KEYINFO

        let result = IOConnectCallStructMethod(
            connection,
            2,
            &inputStruct,
            MemoryLayout<SMCKeyData_t>.size,
            &outputStruct,
            &outputSize)

        if result == kIOReturnSuccess {
            // Read the actual value
            inputStruct.keyInfo.dataSize = outputStruct.keyInfo.dataSize
            inputStruct.data8 = SMC_CMD_READ_BYTES

            outputSize = MemoryLayout<SMCKeyData_t>.size
            let readResult = IOConnectCallStructMethod(
                connection,
                2,
                &inputStruct,
                MemoryLayout<SMCKeyData_t>.size,
                &outputStruct,
                &outputSize)

            if readResult == kIOReturnSuccess {
                // Convert the raw value based on the data type
                return processValue(outputStruct.bytes, size: Int(outputStruct.keyInfo.dataSize))
            }
        }

        return nil
    }

    private func stringToKey(_ key: String) -> UInt32 {
        var ans = UInt32(0)
        let bytes = Array(key.utf8)
        for i in 0..<4 {
            if i < bytes.count {
                ans += UInt32(bytes[i]) << (UInt32(3 - i) * 8)
            }
        }
        return ans
    }

    private func processValue(
        _ bytes: (
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8
        ),
        size: Int
    ) -> Double? {
        // Most SMC values are either SP78 (fixed-point) or FP88 (floating-point)
        let val = Double(Int16(bytes.0) * 256 + Int16(bytes.1))
        return val / 256.0
    }
}

// MARK: - SMC Types and Constants

private let SMC_CMD_READ_BYTES: UInt8 = 5
private let SMC_CMD_READ_KEYINFO: UInt8 = 9

private struct SMCKeyData_t {
    var key: UInt32 = 0
    var vers = SMCVers_t()
    var pLimitData: UInt16 = 0
    var keyInfo = SMCKeyInfoData_t()
    var padding: UInt16 = 0
    var result: UInt8 = 0
    var status: UInt8 = 0
    var data8: UInt8 = 0
    var data32: UInt32 = 0
    var bytes:
        (
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
            UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8
        ) = (
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0
        )
}

private struct SMCVers_t {
    var major: UInt8 = 0
    var minor: UInt8 = 0
    var build: UInt8 = 0
    var reserved: UInt8 = 0
    var release: UInt16 = 0
}

private struct SMCKeyInfoData_t {
    var dataSize: UInt32 = 0
    var dataType: UInt32 = 0
    var dataAttributes: UInt8 = 0
}
