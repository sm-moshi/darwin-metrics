/// # Core Metrics Module
///
/// This module provides the core metrics collection and processing functionality for the darwin-metrics library.
/// It defines the fundamental types and traits for collecting and managing system metrics on macOS.
///
/// ## Features
///
/// * Generic metric types for collecting and storing measurements
/// * Time series data structures for tracking metric history
/// * Hardware-specific metric collection interfaces
/// * Metric aggregation and processing utilities
///
/// ## Modules
///
/// * `hardware` - Hardware-specific metric collection and monitoring interfaces
///
/// ## Example
///
/// ```rust
/// use darwin_metrics::core::metrics::{Metric, MetricSeries};
///
/// // Create a new metric series for tracking CPU usage
/// let mut cpu_metrics = MetricSeries::new("cpu_usage");
///
/// // Add a measurement
/// cpu_metrics.add(Metric::new(75.5));
/// ```
use std::time::SystemTime;

/// # Metrics Module
///
/// The metrics module provides core functionality for collecting and processing
/// system metrics on macOS. It includes interfaces for hardware monitoring,
/// metric collection, and data processing.
///
/// ## Features
///
/// * Hardware monitoring interfaces for CPU, GPU, memory, and storage
/// * Metric collection traits for various system components
/// * Data processing utilities for system metrics
///
/// ## Example
///
/// ```rust
/// use darwin_metrics::core::metrics::hardware::HardwareMonitor;
/// use darwin_metrics::System;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let system = System::new()?;
///     let metrics = system.metrics().await?;
///     println!("CPU Usage: {}%", metrics.cpu_usage);
///     Ok(())
/// }
/// ```

/// Hardware monitoring and metric collection interfaces
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
        Self {
            value,
            timestamp: SystemTime::now(),
        }
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

// NOTE: Hardware monitoring traits like HardwareMonitor, TemperatureMonitor, and UtilizationMonitor
// should be imported directly from darwin_metrics::traits module
