use std::path::PathBuf;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/swift/");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    // Use the workspace directory as the root
    let workspace_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let bridge_dir = workspace_dir.join("generated");

    // Create the bridge directory if it doesn't exist
    std::fs::create_dir_all(&bridge_dir).unwrap();

    // Set environment variables for Swift toolchain
    if cfg!(target_os = "macos") {
        // Ensure we use the system's Swift toolchain
        println!("cargo:rustc-env=SWIFT_TOOLCHAIN=/Applications/Xcode-beta.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain");
        println!("cargo:rustc-env=SDKROOT=/Applications/Xcode-beta.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk");
    }

    // Parse the Rust bridge module and generate Swift bindings
    let bridges = vec!["src/lib.rs"];
    swift_bridge_build::parse_bridges(bridges);

    // Link against system frameworks
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");

    // Add framework search paths
    println!("cargo:rustc-link-search=framework=/System/Library/Frameworks");
    println!("cargo:rustc-link-search=framework=/Library/Frameworks");
}