use darwin_metrics::gpu::Gpu;

/// Creates a test GPU instance
pub fn create_test_gpu() -> Gpu {
    Gpu::new().expect("Failed to create test GPU")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_builder() {
        let gpu = create_test_gpu();
        assert!(gpu.name().is_ok(), "Should be able to get GPU name");
        assert!(gpu.metrics().is_ok(), "Should be able to get GPU metrics");
    }
} 