use std::time::SystemTime;

// Keep the hardware module for backward compatibility but mark it as deprecated
#[deprecated(
    since = "0.2.0",
    note = "Hardware traits have been moved to the traits module. Use `darwin_metrics::traits` instead."
)]
pub mod hardware;

/// A single metric measurement with a timestamp
#[derive(Debug, Clone)]
pub struct Metric<T> {
    /// The value of the metric
    pub value: T,
    /// The timestamp when the metric was recorded
    pub timestamp: SystemTime,
}

impl<T> Metric<T> {
    /// Creates a new metric with the current time as timestamp
    pub fn new(value: T) -> Self {
        Self { value, timestamp: SystemTime::now() }
    }

    /// Creates a new metric with a specific timestamp
    pub fn with_timestamp(value: T, timestamp: SystemTime) -> Self {
        Self { value, timestamp }
    }
}

/// A series of metrics over time
#[derive(Debug, Clone)]
pub struct MetricSeries<T> {
    /// The collection of metrics in this series
    pub metrics: Vec<Metric<T>>,
}

impl<T> Default for MetricSeries<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MetricSeries<T> {
    /// Creates a new empty metric series
    pub fn new() -> Self {
        Self { metrics: Vec::new() }
    }

    /// Adds a metric to the series
    pub fn add(&mut self, metric: Metric<T>) {
        self.metrics.push(metric);
    }

    /// Returns the most recent metric in the series, if any
    pub fn latest(&self) -> Option<&Metric<T>> {
        self.metrics.last()
    }
}

// Re-export hardware monitoring traits through the prelude only
// Deprecated - use traits module directly instead
#[deprecated(
    since = "0.2.0",
    note = "Use `darwin_metrics::traits` instead."
)]
pub(crate) use hardware::{HardwareMonitor, TemperatureMonitor, UtilizationMonitor};
