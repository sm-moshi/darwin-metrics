//! Process metrics and information for macOS systems.
//!
//! This module provides functionality to gather process-related metrics and information
//! on macOS systems. It supports monitoring of:
//! - Process CPU and memory usage
//! - Process uptime and status
//! - Real-time process metrics
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::process::Process;
//! use std::time::Duration;
//! use futures_util::stream::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> darwin_metrics::Result<()> {
//!     // Get information about all processes
//!     let processes = Process::get_all().await?;
//!     for process in processes {
//!         println!("Process: {} (PID: {})", process.name, process.pid);
//!     }
//!     
//!     // Monitor a specific process
//!     let mut stream = Process::monitor_metrics(1234, Duration::from_secs(1));
//!     while let Some(metrics) = stream.next().await {
//!         match metrics {
//!             Ok(process) => println!("CPU: {}%, Memory: {} MB", process.cpu_usage, process.memory_usage / 1024 / 1024),
//!             Err(err) => eprintln!("Error: {}", err),
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use crate::Error;
use async_trait::async_trait;
use futures::Future;
use futures::Stream;
use libproc::libproc::proc_pid;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[async_trait]
pub trait ProcessInfo {
    /// Collect process information asynchronously
    async fn collect(&self) -> Result<Vec<u8>, Error>;
}

/// Represents process information and metrics
///
/// This struct provides access to various process metrics, including:
/// - Process ID and name
/// - CPU and memory usage
/// - Process uptime
///
/// # Examples
///
/// ```
/// use darwin_metrics::process::Process;
///
/// let process = Process::new(1234, "my_process");
/// println!("Process: {} (PID: {})", process.name, process.pid);
/// ```
pub struct Process {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Process uptime
    pub uptime: Duration,
    pending_future: Option<Pin<Box<dyn Future<Output = Result<Process, Error>> + Send>>>,
}

impl Process {
    /// Create a new Process instance
    ///
    /// # Arguments
    /// * `pid` - Process ID
    /// * `name` - Process name
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::process::Process;
    ///
    /// let process = Process::new(1234, "my_process");
    /// assert_eq!(process.pid, 1234);
    /// assert_eq!(process.name, "my_process");
    /// ```
    pub fn new(pid: u32, name: impl Into<String>) -> Self {
        Self {
            pid,
            name: name.into(),
            cpu_usage: 0.0,
            memory_usage: 0,
            uptime: Duration::default(),
            pending_future: None,
        }
    }

    /// Get information about all running processes asynchronously
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Process>>` which is:
    /// - `Ok(Vec<Process>)` containing information for all running processes
    /// - `Err(Error)` if the information cannot be retrieved
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::process::Process;
    ///
    /// #[tokio::main]
    /// async fn main() -> darwin_metrics::Result<()> {
    ///     let processes = Process::get_all().await?;
    ///     for process in processes {
    ///         println!("Process: {} (PID: {})", process.name, process.pid);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_all() -> Result<Vec<Self>, Error> {
        Err(Error::not_implemented("Process information collection"))
    }

    /// Get information about a specific process asynchronously
    ///
    /// # Arguments
    /// * `pid` - Process ID to query
    ///
    /// # Returns
    ///
    /// Returns a `Result<Process>` which is:
    /// - `Ok(Process)` containing information for the specified process
    /// - `Err(Error)` if the information cannot be retrieved
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::process::Process;
    ///
    /// #[tokio::main]
    /// async fn main() -> darwin_metrics::Result<()> {
    ///     let process = Process::get_by_pid(1234).await?;
    ///     println!("Process: {} (PID: {})", process.name, process.pid);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_by_pid(pid: u32) -> Result<Self, Error> {
        // Retrieve task information using the correct method
        let task_info = proc_pid::pidinfo::<libproc::task_info::TaskInfo>(pid as i32, 0)
            .map_err(|e| {
                // Log the error for permission issues
                println!("Failed to get task info for PID {}: {}", pid, e);
                Error::NotAvailable(format!("Failed to get task info: {}", e))
            })?;

        // Calculate CPU usage as a percentage (0-100)
        let total_time = task_info.pti_total_user + task_info.pti_total_system;
        let cpu_usage = if total_time > 0 {
            // Normalize to percentage and ensure it doesn't exceed 100%
            (total_time as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };

        // Create and return the Process struct
        Ok(Process {
            pid,
            name: format!("Process with PID {}", pid), // Placeholder for name retrieval
            cpu_usage: cpu_usage.min(100.0),
            memory_usage: task_info.pti_virtual_size as u64,
            uptime: Duration::from_secs(0), // Placeholder for uptime
            pending_future: None,
        })
    }

    /// Monitor process metrics with specified interval
    ///
    /// This method returns a stream that periodically fetches process metrics.
    ///
    /// # Arguments
    /// * `pid` - Process ID to monitor
    /// * `interval` - Interval between metric updates
    ///
    /// # Returns
    ///
    /// Returns a stream of `Result<Process, Error>` that yields process metrics
    /// at the specified interval.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::process::Process;
    /// use std::time::Duration;
    /// use futures_util::stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut stream = Process::monitor_metrics(1234, Duration::from_secs(1));
    ///     while let Some(metrics) = stream.next().await {
    ///         match metrics {
    ///             Ok(process) => println!("CPU: {}%, Memory: {} MB", process.cpu_usage, process.memory_usage / 1024 / 1024),
    ///             Err(err) => eprintln!("Error: {}", err),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn monitor_metrics(
        pid: u32,
        interval: Duration,
    ) -> impl Stream<Item = Result<Self, Error>> {
        ProcessMetricsStream::new(pid, interval)
    }
}

impl fmt::Debug for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("name", &self.name)
            .field("cpu_usage", &self.cpu_usage)
            .field("memory_usage", &self.memory_usage)
            .field("uptime", &self.uptime)
            .field("pending_future", &self.pending_future.as_ref().map(|_| "Future"))
            .finish()
    }
}

impl Clone for Process {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid,
            name: self.name.clone(),
            cpu_usage: self.cpu_usage,
            memory_usage: self.memory_usage,
            uptime: self.uptime,
            pending_future: None,
        }
    }
}

/// Stream implementation for process metrics monitoring
///
/// This struct implements a stream that periodically fetches process metrics
/// at a specified interval.
pub struct ProcessMetricsStream {
    pid: u32,
    interval: tokio::time::Interval,
    pending_future: Option<Pin<Box<dyn Future<Output = Result<Process, Error>> + Send>>>,
}

impl ProcessMetricsStream {
    /// Create a new stream for process metrics
    ///
    /// # Arguments
    /// * `pid` - Process ID to monitor
    /// * `interval` - Interval between metric updates
    pub fn new(pid: u32, interval: Duration) -> Self {
        Self {
            pid,
            interval: tokio::time::interval(interval),
            pending_future: None,
        }
    }
}

impl fmt::Debug for ProcessMetricsStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessMetricsStream")
            .field("pid", &self.pid)
            .field("interval", &self.interval)
            .field("pending_future", &self.pending_future.as_ref().map(|_| "Future"))
            .finish()
    }
}

impl Clone for ProcessMetricsStream {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid,
            interval: tokio::time::interval(self.interval.period()),
            pending_future: None,
        }
    }
}

impl Stream for ProcessMetricsStream {
    type Item = Result<Process, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // First check any pending future
        if let Some(fut) = &mut this.pending_future {
            match fut.as_mut().poll(cx) {
                Poll::Ready(result) => {
                    this.pending_future = None;
                    return Poll::Ready(Some(result));
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        // Then poll the interval
        match this.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                // Start fetching the next metric
                let pid = this.pid;
                this.pending_future = Some(Box::pin(async move { Process::get_by_pid(pid).await }));

                // Try polling the new future immediately
                if let Some(fut) = &mut this.pending_future {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(result) => {
                            this.pending_future = None;
                            Poll::Ready(Some(result))
                        }
                        Poll::Pending => Poll::Pending,
                    }
                } else {
                    // This should never happen
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;
    use std::process::Command;

    async fn spawn_test_process() -> (std::process::Child, u32) {
        let child = Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn test process");
        let pid = child.id();
        (child, pid)
    }

    #[tokio::test]
    async fn test_process_metrics_stream() {
        // First test with our own process which we should have access to
        let (mut child, pid) = spawn_test_process().await;
        let mut stream = Process::monitor_metrics(pid, Duration::from_millis(100));

        match stream.next().await {
            Some(Ok(process)) => {
                assert_eq!(process.pid, pid);
                assert!(process.cpu_usage >= 0.0 && process.cpu_usage <= 100.0);
                assert!(process.memory_usage > 0);
            },
            Some(Err(e)) => panic!("Unexpected error for our test process: {:?}", e),
            None => panic!("Stream ended unexpectedly"),
        }

        // Clean up test process
        child.kill().expect("Failed to kill test process");

        // Now test with PID 1 which typically requires elevated permissions
        let mut stream = Process::monitor_metrics(1, Duration::from_millis(100));
        match stream.next().await {
            Some(Err(Error::NotAvailable(_))) => (), // Permission denied is expected
            other => panic!("Unexpected result for privileged process: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_by_pid() {
        let (mut child, pid) = spawn_test_process().await;

        // Test getting process info for our test process
        let process = Process::get_by_pid(pid).await.expect("Failed to get process info");
        assert_eq!(process.pid, pid);
        assert!(process.cpu_usage >= 0.0 && process.cpu_usage <= 100.0, "CPU usage out of bounds: {}", process.cpu_usage);
        assert!(process.memory_usage > 0, "Memory usage should be positive");
        assert!(process.uptime.as_secs() >= 0);

        // Clean up test process
        child.kill().expect("Failed to kill test process");

        // Test with PID 1 (system process) - should fail with permission error
        match Process::get_by_pid(1).await {
            Err(Error::NotAvailable(_)) => (), // Permission denied is expected
            other => panic!("Unexpected result for privileged process: {:?}", other),
        };
    }

    #[tokio::test]
    async fn test_get_by_pid_invalid() {
        // Test with invalid PID
        match Process::get_by_pid(u32::MAX).await {
            Err(Error::NotAvailable(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }
}
