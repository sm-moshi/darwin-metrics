import Foundation
import darwin_metrics_swift_bridge
import os

// MARK: - FFI Interface

/// A wrapper to safely manage FFI memory and provide proper cleanup
private final class FFIWrapper {
    private var pointer: UnsafeMutablePointer<TestFFI>?

    init() {
        pointer = UnsafeMutablePointer<TestFFI>.allocate(capacity: 1)
        pointer?.initialize(to: TestFFI())
    }

    deinit {
        if let ptr = pointer {
            ptr.deinitialize(count: 1)
            ptr.deallocate()
        }
    }

    func getPointer() -> UnsafeMutablePointer<TestFFI>? {
        return pointer
    }
}

// Thread-safe storage for FFI wrappers to prevent memory leaks
private let ffiStorage = NSMapTable<NSString, FFIWrapper>.strongToStrongObjects()
private let storageLock = os_unfair_lock_t.allocate(capacity: 1)

// MARK: - Public Interface

/// Creates and returns a test FFI structure
/// - Returns: A pointer to the FFI structure, or nil if allocation fails
/// - Note: The memory is managed internally and will be cleaned up when no longer needed
@_cdecl("getTest")
public func getTest() -> UnsafeMutablePointer<TestFFI>? {
    os_unfair_lock_lock(storageLock)
    defer { os_unfair_lock_unlock(storageLock) }

    let wrapper = FFIWrapper()
    guard let pointer = wrapper.getPointer() else {
        os_log("Failed to allocate FFI structure", type: .error)
        return nil
    }

    // Initialize the structure
    pointer.pointee.value = 42
    pointer.pointee.other_value = 3.14159

    // Store the wrapper to keep it alive
    let key = NSString(string: UUID().uuidString)
    ffiStorage.setObject(wrapper, forKey: key)

    return pointer
}

@available(macOS 13.0, *)
public final class TestFFIProvider {
    private let logger = Logger(subsystem: "com.darwin.metrics", category: "TestFFI")

    // Thread-safe state management using actor
    private actor State {
        var value: Int32 = 42
        var otherValue: Double = 3.14159

        func getData() -> TestFFI {
            TestFFI(value: value, other_value: otherValue)
        }
    }

    private let state = State()

    public init() {}

    public func getTest() async -> TestFFIResult {
        do {
            let data = try await state.getData()
            return TestFFIResult(success: true, data: data)
        } catch {
            logger.error("Failed to get test data: \(error.localizedDescription)")
            // Return a default TestFFI with success = false
            return TestFFIResult(success: false, data: TestFFI(value: 0, other_value: 0.0))
        }
    }
}

// Bridge implementation
@_cdecl("swift_getTest")
public func swift_getTest(provider: UnsafePointer<TestFFIProvider>) -> TestFFIResult {
    guard let provider = provider.pointee as? TestFFIProvider else {
        return TestFFIResult(success: false, data: TestFFI(value: 0, other_value: 0.0))
    }

    return await provider.getTest()
}
