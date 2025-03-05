use crate::Error;
use async_trait::async_trait;
use futures::Future;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[async_trait]
pub trait ProcessInfo {
    async fn collect(&self) -> Result<Vec<u8>, Error>;
}

/// Process information
#[derive(Debug, Clone)]
pub struct Process {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Process uptime
    pub uptime: Duration,
}

impl Process {
    /// Create a new Process instance
    pub fn new(pid: u32, name: impl Into<String>) -> Self {
        Self {
            pid,
            name: name.into(),
            cpu_usage: 0.0,
            memory_usage: 0,
            uptime: Duration::default(),
        }
    }

    /// Get information about all running processes asynchronously
    pub async fn get_all() -> Result<Vec<Self>, Error> {
        Err(Error::not_implemented("Process information collection"))
    }

    /// Get information about a specific process asynchronously
    pub async fn get_by_pid(_pid: u32) -> Result<Self, Error> {
        Err(Error::not_implemented("Process information collection"))
    }

    /// Monitor process metrics with specified interval
    pub fn monitor_metrics(
        pid: u32,
        interval: Duration,
    ) -> impl Stream<Item = Result<Self, Error>> {
        ProcessMetricsStream::new(pid, interval)
    }
}

/// Stream implementation for process metrics monitoring
pub struct ProcessMetricsStream {
    pid: u32,
    interval: tokio::time::Interval,
    pending_future: Option<Pin<Box<dyn Future<Output = Result<Process, Error>> + Send>>>,
}

impl ProcessMetricsStream {
    /// Create a new stream for process metrics
    pub fn new(pid: u32, interval: Duration) -> Self {
        Self {
            pid,
            interval: tokio::time::interval(interval),
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
    use futures::StreamExt;

    #[tokio::test]
    async fn test_process_metrics_stream() {
        let mut stream = Process::monitor_metrics(1, Duration::from_millis(100));

        // Should return NotImplemented error as the actual implementation isn't ready
        match stream.next().await {
            Some(Err(Error::NotImplemented(_))) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }
}
