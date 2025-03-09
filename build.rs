use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Tell Rust that docsrs is a valid configuration flag
    println!("cargo:rustc-check-cfg=cfg(docsrs)");

    // Get the target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| String::from("unknown"));

    // Check if we're building for docs.rs or a non-macOS platform
    let is_docs_rs = env::var("DOCS_RS").is_ok();
    let is_non_macos = target_os != "macos";

    if is_docs_rs || is_non_macos {
        println!(
            "cargo:warning=Building for documentation or non-macOS platform. Skipping \
             macOS-specific linking."
        );

        // Set a custom cfg flag for our stubs module
        println!("cargo:rustc-cfg=use_stubs");
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
