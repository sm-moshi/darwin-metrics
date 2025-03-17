use async_trait::async_trait;
use metal::Device as MTLDevice;
use std::time::SystemTime;

use crate::{
    core::{
        metrics::{hardware::HardwareMonitor, Metric},
        types::{ByteSize, Percentage},
    },
    error::Result,
    hardware::gpu::{
        monitors::{
            characteristics::GpuCharacteristicsMonitor, memory::GpuMemoryMonitor, temperature::GpuTemperatureMonitor,
            utilization::GpuUtilizationMonitor,
        },
        types::{GpuCharacteristics, GpuMemory, GpuMetrics, GpuUtilization},
    },
};

/// GPU monitoring functionality
///
/// This struct provides access to GPU information and metrics on macOS systems. It supports both discrete and
/// integrated GPUs, including Apple Silicon GPUs.
///
/// # Examples
///
/// ```no_run
/// use darwin_metrics::hardware::gpu::Gpu;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpu = Gpu::new().await?;
///     let metrics = gpu.get_metric().await?;
///     
///     println!("GPU: {}", metrics.value.name);
///     println!("Utilization: {:.1}%", metrics.value.utilization);
///     println!("Memory used: {} bytes", metrics.value.memory.used);
///     
///     Ok(())
/// }
/// ```
pub struct GpuMonitors {
    pub characteristics: GpuCharacteristicsMonitor,
    pub memory: GpuMemoryMonitor,
    pub temperature: GpuTemperatureMonitor,
    pub utilization: GpuUtilizationMonitor,
}

/// Represents a GPU device
pub struct Gpu {
    metal_device: Option<MTLDevice>,
    monitors: GpuMonitors,
}

impl Gpu {
    /// Creates a new GPU instance
    pub fn new() -> Result<Self> {
        let metal_device = MTLDevice::system_default();

        // Create the monitors with None for now
        let monitors = GpuMonitors {
            characteristics: GpuCharacteristicsMonitor::new(None),
            memory: GpuMemoryMonitor::new(None),
            temperature: GpuTemperatureMonitor::new(None),
            utilization: GpuUtilizationMonitor::new(None, 0),
        };

        Ok(Self { metal_device, monitors })
    }

    /// Gets the Metal device if available
    pub fn get_metal_device(&self) -> Option<&MTLDevice> {
        self.metal_device.as_ref()
    }

    /// Gets current GPU utilization (0-100)
    pub async fn get_utilization(&self) -> Result<GpuUtilization> {
        self.monitors.utilization.get_utilization().await
    }

    /// Gets current GPU temperature in Celsius
    pub async fn get_temperature(&self) -> Result<f64> {
        let temp = self.monitors.temperature.get_temperature().await?;
        Ok(temp.into())
    }

    /// Gets current GPU memory information
    pub async fn get_memory(&self) -> Result<GpuMemory> {
        self.monitors.memory.get_memory_info().await
    }

    /// Gets GPU characteristics
    pub async fn get_characteristics(&self) -> Result<GpuCharacteristics> {
        self.monitors.characteristics.get_characteristics().await
    }

    /// Gets the GPU name
    pub async fn name(&self) -> Result<String> {
        if let Some(device) = &self.metal_device {
            Ok(device.name().to_string())
        } else {
            Ok("Unknown GPU".to_string())
        }
    }

    /// Gets the hardware type
    pub async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    /// Gets the device ID
    pub async fn device_id(&self) -> Result<String> {
        if let Some(device) = &self.metal_device {
            Ok(format!("metal-{}", device.registry_id()))
        } else {
            Ok("unknown-gpu".to_string())
        }
    }

    /// Gets the current GPU metrics
    pub async fn get_metric(&self) -> Result<Metric<GpuMetrics>> {
        let utilization = self.get_utilization().await?;
        let memory = self.get_memory().await?;
        let temp = self.get_temperature().await?;

        let metrics = GpuMetrics {
            timestamp: SystemTime::now(),
            utilization: Percentage::from_f64(utilization.value),
            memory_used: ByteSize::from_bytes(memory.used),
            memory_total: ByteSize::from_bytes(memory.total),
            temperature: temp,
            power_usage: None,
        };

        Ok(Metric::new(metrics))
    }
}

#[async_trait]
impl HardwareMonitor for Gpu {
    type MetricType = GpuMetrics;

    async fn name(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("gpu0".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<GpuMetrics>> {
        let utilization = self.get_utilization().await?;
        let memory = self.get_memory().await?;
        let temp = self.get_temperature().await?;

        let metrics = GpuMetrics {
            timestamp: SystemTime::now(),
            utilization: Percentage::from_f64(utilization.value),
            memory_used: ByteSize::from_bytes(memory.used),
            memory_total: ByteSize::from_bytes(memory.total),
            temperature: temp,
            power_usage: None,
        };

        Ok(Metric::new(metrics))
    }
}

// MTLDevice is already managed by the metal crate, so we don't need to manually drop it
unsafe impl Send for Gpu {}
unsafe impl Sync for Gpu {}
