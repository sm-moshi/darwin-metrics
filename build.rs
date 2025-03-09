use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Add framework search paths - order matters!
    println!(
        "cargo:rustc-link-search=framework=/Applications/Xcode.app/Contents/Developer/Platforms/\
         MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks"
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

    Ok(())
}
