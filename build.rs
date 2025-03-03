use std::path::{PathBuf, Path};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;

fn wait_for_file(path: &Path, timeout_secs: u64) -> Result<(), String> {
    println!("Waiting for file to be generated: {}", path.display());
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    
    while !path.exists() {
        if start.elapsed() > timeout {
            return Err(format!("Timeout waiting for file: {}", path.display()));
        }
        println!("File not found, waiting... elapsed: {:?}", start.elapsed());
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("File found: {}", path.display());
    Ok(())
}

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/swift/");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    println!("Starting build script...");
    
    // Set up workspace directory
    let workspace_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Failed to get CARGO_MANIFEST_DIR");
    println!("Workspace directory: {}", workspace_dir);
    
    let bridge_dir = Path::new(&workspace_dir).join("generated");
    println!("Bridge directory: {}", bridge_dir.display());
    
    // Create bridge directory if it doesn't exist
    fs::create_dir_all(&bridge_dir).expect("Failed to create bridge directory");
    
    // Set environment variables for Swift toolchain on macOS
    let swift_toolchain = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain";
    let sdkroot = "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk";
    
    println!("Setting environment variables:");
    println!("SWIFT_TOOLCHAIN={}", swift_toolchain);
    println!("SDKROOT={}", sdkroot);
    println!("DYLD_LIBRARY_PATH={}", bridge_dir.display());
    println!("SWIFT_BRIDGE_OUT_DIR={}", bridge_dir.display());
    
    // Set environment variables in an unsafe block since it's a system-wide operation
    unsafe {
        std::env::set_var("SWIFT_TOOLCHAIN", swift_toolchain);
        std::env::set_var("SDKROOT", sdkroot);
        std::env::set_var("DYLD_LIBRARY_PATH", bridge_dir.display().to_string());
        std::env::set_var("SWIFT_BRIDGE_OUT_DIR", bridge_dir.display().to_string());
    }
    
    // Tell cargo to look for frameworks
    println!("cargo:rustc-link-search=framework=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/Library/Frameworks");
    
    // Parse Rust bridge module and generate Swift bindings
    println!("Parsing Rust bridge module...");
    let bridges = swift_bridge_build::parse_bridges(vec!["src/swift/bindings.rs"]);
    println!("Writing bridge files...");
    bridges.write_all_concatenated(&bridge_dir, "darwin_metrics_swift_bridge");
    println!("Finished writing bridge files");
    
    // Wait for bridge files to be generated
    let bridge_subdir = bridge_dir.join("darwin_metrics_swift_bridge");
    let header_path = bridge_subdir.join("darwin_metrics_swift_bridge.h");
    let swift_path = bridge_subdir.join("darwin_metrics_swift_bridge.swift");
    
    println!("Waiting for bridge files to be generated...");
    if let Err(e) = wait_for_file(&header_path, 10) {
        panic!("{}", e);
    }
    if let Err(e) = wait_for_file(&swift_path, 10) {
        panic!("{}", e);
    }
    
    // Create swift directory if it doesn't exist
    let swift_dir = Path::new(&workspace_dir).join("src").join("swift");
    fs::create_dir_all(&swift_dir).expect("Failed to create swift directory");
    
    // Create bridge subdirectory in swift directory
    let swift_bridge_dir = swift_dir.join("darwin_metrics_swift_bridge");
    fs::create_dir_all(&swift_bridge_dir).expect("Failed to create bridge directory");
    
    // Generate module map in the bridge subdirectory
    println!("Generating module map...");
    let module_map = format!(
        r#"module darwin_metrics_swift_bridge {{
    header "../darwin_metrics_swift_bridge.h"
    export *
}}"#
    );
    fs::write(swift_bridge_dir.join("module.modulemap"), module_map)
        .expect("Failed to write module map");
    
    // Copy files and ensure they exist
    fs::copy(&header_path, swift_dir.join("darwin_metrics_swift_bridge.h"))
        .expect("Failed to copy bridge header");
    fs::copy(&swift_path, swift_dir.join("darwin_metrics_swift_bridge.swift"))
        .expect("Failed to copy bridge swift file");
    
    // Verify files were copied successfully
    let swift_header = swift_dir.join("darwin_metrics_swift_bridge.h");
    let swift_source = swift_dir.join("darwin_metrics_swift_bridge.swift");
    let swift_modulemap = swift_bridge_dir.join("module.modulemap");
    
    if let Err(e) = wait_for_file(&swift_header, 5) {
        panic!("Failed to verify copied header file: {}", e);
    }
    if let Err(e) = wait_for_file(&swift_source, 5) {
        panic!("Failed to verify copied swift file: {}", e);
    }
    if let Err(e) = wait_for_file(&swift_modulemap, 5) {
        panic!("Failed to verify copied module map: {}", e);
    }
    
    println!("Successfully copied bridge files");
    
    // Link against system frameworks
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=XCTest");
    
    // Add Swift module search paths
    println!("cargo:rustc-link-search={}", swift_dir.display());
    println!("cargo:rustc-link-search={}", bridge_dir.display());
    
    println!("Build script completed successfully");
}