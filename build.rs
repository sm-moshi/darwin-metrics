use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Tell Rust that docsrs is a valid configuration flag
    println!("cargo:rustc-check-cfg=cfg(docsrs)");

    // Skip linking macOS-specific libraries when building documentation on docs.rs
    if cfg!(docsrs) {
        println!("cargo:warning=Building for docs.rs, skipping macOS-specific linking");
        return Ok(());
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
