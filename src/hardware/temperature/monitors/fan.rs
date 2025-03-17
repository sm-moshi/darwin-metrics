use super::FanMonitoring;
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::hardware::temperature::constants::*;
use std::sync::Arc;

/// Monitor for fan speed and control
pub struct FanMonitor {
    io_kit: Arc<Box<dyn IOKit>>,
    index: usize,
}

impl FanMonitor {
    /// Create a new fan monitor for the specified fan index
    pub fn new(io_kit: Arc<Box<dyn IOKit>>, index: usize) -> Self {
        Self { io_kit, index }
    }
}

#[async_trait::async_trait]
impl FanMonitoring for FanMonitor {
    async fn speed_rpm(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.speed_rpm).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    async fn min_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.min_speed).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    async fn max_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.max_speed).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    async fn percentage(&self) -> Result<f64> {
        let speed = self.speed_rpm().await? as f64;
        let min = self.min_speed().await? as f64;
        let max = self.max_speed().await? as f64;

        if max == min {
            return Ok(0.0);
        }

        let percentage = ((speed - min) / (max - min)) * 100.0;
        Ok(percentage.clamp(MIN_FAN_SPEED_PERCENTAGE, MAX_FAN_SPEED_PERCENTAGE))
    }

    async fn fan_name(&self) -> Result<String> {
        Ok(format!("Fan {}", self.index))
    }
}
