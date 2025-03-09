use std::error::Error;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Stay within OUT_DIR for any file operations
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    println!("cargo:warning=OUT_DIR is: {}", out_path.display());
    
    // Ensure we're not creating any files in the source directory
    // If needed, only write files to out_path.join("some_file")

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
