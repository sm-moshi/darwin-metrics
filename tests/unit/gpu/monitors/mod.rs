mod characteristics;
mod memory;
mod temperature;
mod utilization;

use darwin_metrics::error::Result;
use tokio::test;

use crate::common::builders::gpu::create_test_gpu;

#[test]
async fn test_gpu_metrics_collection() -> Result<()> {
    let gpu = create_test_gpu().await;
    let metrics = gpu.get_metric().await?;

    assert!(metrics.value.utilization >= 0.0 && metrics.value.utilization <= 100.0);
    assert!(metrics.value.memory.total > 0);
    assert!(!metrics.value.name.is_empty());

    Ok(())
}

#[test]
async fn test_gpu_memory_monitoring() -> Result<()> {
    let gpu = create_test_gpu().await;
    let memory = gpu.get_memory().await?;

    assert!(memory.total > 0);
    assert!(memory.used <= memory.total);
    assert_eq!(memory.total, memory.used + memory.free);

    Ok(())
}

#[test]
async fn test_gpu_temperature_monitoring() -> Result<()> {
    let gpu = create_test_gpu().await;
    let temp = gpu.get_temperature().await?;

    if let Some(temp) = temp {
        assert!(temp >= 0.0 && temp <= 150.0); // Reasonable temperature range
    }

    Ok(())
}
