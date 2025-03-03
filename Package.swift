// swift-tools-version:6.0
import PackageDescription

let package = Package(
    name: "DarwinMetrics",
    platforms: [
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "DarwinMetrics",
            targets: ["DarwinMetrics"]
        )
    ],
    dependencies: [],
    targets: [
        .target(
            name: "DarwinMetrics",
            dependencies: [],
            path: "src/swift",
            exclude: ["bindings.rs", "tests"],
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath("."),
                .unsafeFlags([
                    "-fmodules", "-fmodule-map-file=darwin_metrics_swift_bridge/module.modulemap",
                ]),
            ],
            swiftSettings: [
                .enableUpcomingFeature("StrictConcurrency"),
                .enableExperimentalFeature("DataRaceSafety"),
            ],
            linkerSettings: [
                .linkedFramework("IOKit"),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("Foundation"),
            ]
        ),
        .testTarget(
            name: "DarwinMetricsTests",
            dependencies: ["DarwinMetrics"],
            path: "src/swift/tests",
            sources: ["."],
            swiftSettings: [
                .enableUpcomingFeature("StrictConcurrency"),
                .enableExperimentalFeature("DataRaceSafety"),
            ],
            linkerSettings: [
                .linkedFramework("XCTest")
            ]
        ),
    ]
)
