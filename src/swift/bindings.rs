use swift_bridge::*;

// Swift bridge module
#[swift_bridge::bridge(swift_module_name = "darwin_metrics_swift_bridge")]
mod ffi {
    // Test FFI structure
    #[swift_bridge(swift_repr = "struct")]
    pub struct TestFFI {
        // Integer value
        pub value: i32,
        // Float value
        pub other_value: f64,
    }

    #[swift_bridge(swift_repr = "struct")]
    pub struct TestFFIResult {
        pub success: bool,
        pub data: TestFFI,
    }

    // Swift interface
    extern "Swift" {
        type TestFFIProvider;

        // Get test function
        #[swift_bridge(swift_name = "getTest")]
        fn get_test(provider: &TestFFIProvider) -> TestFFIResult;
    }
}

pub use ffi::*; 