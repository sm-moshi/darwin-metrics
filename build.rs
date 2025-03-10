use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // No special cfg flags needed anymore

    // Get the target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| String::from("unknown"));

    // Check if non-macOS platform
    let is_non_macos = target_os != "macos";

    // Check if we're running llvm-cov
    let is_coverage = std::env::args().any(|arg| arg.contains("llvm-cov"));

    if is_non_macos {
        println!(
            "cargo:warning=Building for non-macOS platform. This library only works on macOS."
        );
        return Ok(());
    }

    // Enable skip-ffi-crashes feature if we're running coverage
    if is_coverage {
        println!("cargo:warning=Building with coverage instrumentation - enabling skip-ffi-crashes feature");
        println!("cargo:rustc-cfg=feature=\"skip-ffi-crashes\"");
    }

    #[cfg(target_os = "macos")]
    {
        // Add framework search paths - order matters!
        println!(
            "cargo:rustc-link-search=framework=/Applications/Xcode.app/Contents/Developer/\
             Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks"
        );
        println!("cargo:rustc-link-search=framework=/System/Library/Frameworks");
        println!("cargo:rustc-link-search=framework=/Library/Frameworks");
        println!("cargo:rustc-link-search=native=/usr/lib");

        // Link required frameworks
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
        println!("cargo:rustc-link-lib=framework=IOSurface");
    }

    Ok(())
}
