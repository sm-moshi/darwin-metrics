use crate::error::Result;
use crate::utils::{autorelease_pool, objc_safe_exec};
use async_trait::async_trait;
use futures::Future;
use futures::Stream;
use libproc::task_info;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

#[async_trait]
pub trait ProcessInfo {
    async fn collect(&self) -> crate::Result<Vec<u8>>;
}

pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub uptime: Duration,
    pending_future: Option<Pin<Box<dyn Future<Output = crate::Result<Process>> + Send>>>,
}

impl Process {
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

    pub async fn get_all() -> crate::Result<Vec<Self>> {
        Err(crate::Error::not_implemented("Process information collection"))
    }

    pub async fn get_by_pid(pid: u32) -> crate::Result<Self> {
        let name = libproc::proc_pid::name(pid as i32)
            .map_err(|e| crate::Error::NotAvailable(format!("Failed to get process name: {}", e)))?;

        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0)
            .map_err(|e| crate::Error::system_error(format!("Failed to get process info: {}", e)))?;
        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return Err(crate::Error::system_error("Invalid process start time"));
        };
        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return Err(crate::Error::system_error("Invalid process start time"));
            }
            Err(_) => {
                match now.duration_since(start_time) {
                    Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                        return Err(crate::Error::system_error("Invalid process start time"));
                    }
                    Ok(_) => (),
                    Err(_) => return Err(crate::Error::system_error("Invalid process data")),
                }
            }
        }
        let cpu_time = proc_info.ptinfo.pti_total_user + proc_info.ptinfo.pti_total_system;
        let memory_usage = proc_info.ptinfo.pti_resident_size;

        Ok(Process {
            pid,
            name,
            cpu_usage: (cpu_time as f64 / 1_000_000.0).min(100.0),
            memory_usage,
            uptime: SystemTime::now()
                .duration_since(start_time)
                .unwrap_or(Duration::ZERO),
            pending_future: None,
        })
    }

    pub async fn get_process_start_time(pid: u32) -> crate::Result<SystemTime> {
        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0)
            .map_err(|e| crate::Error::system_error(format!("Failed to get process info: {}", e)))?;
        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return Err(crate::Error::system_error("Invalid process start time"));
        };
        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return Err(crate::Error::system_error("Invalid process start time"));
            }
            Err(_) => {
                match now.duration_since(start_time) {
                    Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                        return Err(crate::Error::system_error("Invalid process start time"));
                    }
                    Ok(_) => (),
                    Err(_) => return Err(crate::Error::system_error("Invalid process data")),
                }
            }
        }
        Ok(start_time)
    }

    pub fn monitor_metrics(
        pid: u32,
        interval: Duration,
    ) -> impl Stream<Item = crate::Result<Self>> {
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
            .field(
                "pending_future",
                &self.pending_future.as_ref().map(|_| "Future"),
            )
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

pub struct ProcessMetricsStream {
    pid: u32,
    interval: tokio::time::Interval,
    pending_future: Option<Pin<Box<dyn Future<Output = crate::Result<Process>> + Send>>>,
}

impl ProcessMetricsStream {
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
            .field(
                "pending_future",
                &self.pending_future.as_ref().map(|_| "Future"),
            )
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
    type Item = crate::Result<Process>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(fut) = &mut this.pending_future {
            match fut.as_mut().poll(cx) {
                Poll::Ready(result) => {
                    this.pending_future = None;
                    return Poll::Ready(Some(result));
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        match this.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                let pid = this.pid;
                this.pending_future = Some(Box::pin(async move { Process::get_by_pid(pid).await }));

                if let Some(fut) = &mut this.pending_future {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(result) => {
                            this.pending_future = None;
                            Poll::Ready(Some(result))
                        }
                        Poll::Pending => Poll::Pending,
                    }
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
