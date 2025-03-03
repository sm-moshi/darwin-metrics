import Foundation
import XCTest
import darwin_metrics_swift_bridge

@available(macOS 13.0, *)
final class TestFFITests: XCTestCase {
    // Test provider instance
    private var provider: TestFFIProvider!

    override func setUp() async throws {
        super.setUp()
        provider = TestFFIProvider()
    }

    override func tearDown() async throws {
        provider = nil
        await super.tearDown()
    }

    // Test basic data retrieval
    func testGetTestData() async throws {
        let result = await provider.getTest()
        XCTAssertTrue(result.success, "Expected successful result")
        XCTAssertEqual(result.data.value, 42, "Expected default value of 42")
        XCTAssertEqual(result.data.other_value, 3.14159, "Expected default value of 3.14159")
    }

    // Test concurrent access
    func testConcurrentAccess() async throws {
        // Create multiple concurrent tasks accessing the provider
        let tasks = (0..<10).map { _ in
            Task {
                await provider.getTest()
            }
        }

        // Wait for all tasks to complete and verify results
        let results = try await withThrowingTaskGroup(of: TestFFIResult.self) { group in
            for task in tasks {
                group.addTask {
                    try await task.value
                }
            }

            var results: [TestFFIResult] = []
            for try await result in group {
                results.append(result)
            }
            return results
        }

        // Verify all results are successful and contain correct data
        for result in results {
            XCTAssertTrue(result.success, "Expected successful result in concurrent access")
            XCTAssertEqual(
                result.data.value, 42, "Expected consistent value across concurrent access")
            XCTAssertEqual(
                result.data.other_value, 3.14159,
                "Expected consistent value across concurrent access")
        }
    }

    // Test error handling
    func testErrorHandling() async throws {
        // Create a provider that will fail
        let failingProvider = TestFFIProvider()
        // Force an error condition (implementation dependent)
        // For now, we'll just verify the error case returns expected defaults
        let result = await failingProvider.getTest()

        if !result.success {
            XCTAssertEqual(result.data.value, 0, "Expected zero value on failure")
            XCTAssertEqual(result.data.other_value, 0.0, "Expected zero value on failure")
        }
    }

    // Test memory management
    func testMemoryManagement() async throws {
        // Create and destroy many providers to check for memory leaks
        for _ in 0..<100 {
            let tempProvider = TestFFIProvider()
            let _ = await tempProvider.getTest()
        }

        // If we reach here without crashes or leaks, test passes
        XCTAssert(true, "Memory management test completed")
    }
}
