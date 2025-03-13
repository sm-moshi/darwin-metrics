use std::{
    collections::HashMap,
    // fmt,
    time::{Duration, Instant, SystemTime},
};

use libproc::{pid_rusage, proc_pid, task_info};

use std::ffi::c_void;
use libc::{proc_pidinfo, PROC_PIDTASKINFO};
use libproc::pid_rusage::RUsageInfoV4;

use crate::{
    error::{Error, Result},
    utils::bindings::{
        extract_proc_name, is_system_process, kinfo_proc, sysctl, proc_info,
        sysctl_constants::{CTL_KERN, KERN_PROC, KERN_PROC_ALL},
    },
};

/// Interface for collecting process information
///
/// This trait defines the interface for collecting process-specific information from the operating system.
pub trait ProcessInfo {
    /// Collects process information and returns it as a byte vector
    fn collect(&self) -> Result<Vec<u8>>;
}

/// Process I/O statistics
///
/// This struct holds information about a process's I/O operations, including read and write counts and bytes
/// transferred.
#[derive(Debug, Clone, Default)]
pub struct ProcessIOStats {
    /// Number of bytes read from disk
    pub read_bytes: u64,
    /// Number of bytes written to disk
    pub write_bytes: u64,
    /// Number of read operations performed
    pub read_count: u64,
    /// Number of write operations performed
    pub write_count: u64,
}

use std::sync::Mutex;

use once_cell::sync::Lazy as SyncLazy;

/// Static cache for tracking CPU usage calculations between calls
static CPU_HISTORY: SyncLazy<Mutex<HashMap<u32, (Instant, u64)>>> =
    SyncLazy::new(|| Mutex::new(HashMap::new()));

/// Get CPU history tracking map
#[allow(clippy::disallowed_methods)]
fn get_cpu_history() -> std::sync::MutexGuard<'static, HashMap<u32, (Instant, u64)>> {
    CPU_HISTORY.lock().unwrap()
}

/// Process information and monitoring functionality
///
/// This struct represents a process in the system and provides access to its metrics and status information.
#[derive(Debug, Clone)]
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
    /// Process I/O statistics
    pub io_stats: ProcessIOStats,
    /// Number of threads in the process
    pub thread_count: u32,
    /// Whether the process is suspended
    pub is_suspended: bool,
}

/// Process metrics monitor
///
/// This struct provides a way to periodically check process metrics at regular intervals.
/// It uses a polling approach where you can check for updates using the next_update method.
#[derive(Debug, Clone)]
pub struct ProcessMetricsStream {
    pid: u32,
    update_interval: Duration,
    last_update: Instant,
}

impl Process {
    /// Creates a new Process instance with basic information
    pub fn new(pid: u32, name: &str) -> Self {
        Self {
            pid,
            name: name.to_string(),
            cpu_usage: 0.0,
            memory_usage: 0,
            uptime: Duration::new(0, 0),
            io_stats: ProcessIOStats::default(),
            thread_count: 0,
            is_suspended: false,
        }
    }

    /// Gets information about a specific process by its PID
    pub fn get_by_pid(pid: u32) -> Result<Self> {
        let name = libproc::proc_pid::name(pid as i32).map_err(|e| {
            crate::Error::process_error(Some(pid), format!("Failed to get process name: {}", e))
        })?;

        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0)
            .map_err(|e| {
                crate::Error::process_error(Some(pid), format!("Failed to get process info: {}", e))
            })?;

        // Validate and calculate process start time
        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return Err(crate::Error::process_error(Some(pid), "Invalid process start time"));
        };

        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return Err(crate::Error::process_error(
                    Some(pid),
                    "Process start time is in the future",
                ));
            },
            Err(_) => match now.duration_since(start_time) {
                Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                    return Err(crate::Error::process_error(
                        Some(pid),
                        "Process is unrealistically old",
                    ));
                },
                Ok(_) => (),
                Err(_) => {
                    return Err(crate::Error::process_error(
                        Some(pid),
                        "Failed to calculate process age",
                    ))
                },
            },
        }

        // Calculate memory usage
        let memory_usage = proc_info.ptinfo.pti_resident_size;

        // Calculate CPU usage with history for more accurate rate calculation
        let total_cpu_time = proc_info.ptinfo.pti_total_user + proc_info.ptinfo.pti_total_system;
        let cpu_usage = Self::calculate_cpu_usage(pid, total_cpu_time);

        // Get thread count (convert from i32 to u32)
        let thread_count = proc_info.ptinfo.pti_threadnum as u32;

        // Check if process is suspended Use a heuristic since TaskInfo doesn't have pti_suspend_count in this version
        let is_suspended = false; // We can't easily determine if a process is suspended

        // Get I/O statistics
        let io_stats = Self::get_process_io_stats(pid).unwrap_or_default();

        Ok(Self {
            pid,
            name,
            cpu_usage,
            memory_usage,
            uptime: now.duration_since(start_time).unwrap_or_else(|_| Duration::new(0, 0)),
            io_stats,
            thread_count,
            is_suspended,
        })
    }

    /// Gets information about all running processes
    pub fn get_all() -> Result<Vec<Self>> {
        // Try to use sysctl first for bulk retrieval
        match Self::get_all_via_sysctl() {
            Ok(processes) => Ok(processes),
            Err(_) => {
                // Fall back to libproc if sysctl fails
                Self::get_all_via_libproc()
            },
        }
    }

    /// Get all processes using the sysctl API for efficient bulk retrieval
    fn get_all_via_sysctl() -> Result<Vec<Self>> {
        use std::{mem, os::raw::c_void, ptr};

        unsafe {
            // First call to get the size of the buffer needed
            let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0, 0, 0];
            let mut size: usize = 0;

            if sysctl(mib.as_mut_ptr(), 3, ptr::null_mut(), &mut size, ptr::null(), 0) < 0 {
                return Err(crate::Error::process_error(
                    Some(0u32), // Use 0 as PID when not available
                    "Failed to get process list size",
                ));
            }

            // Calculate number of processes
            let count = size / mem::size_of::<kinfo_proc>();

            // Allocate buffer
            let mut processes = Vec::<kinfo_proc>::with_capacity(count);
            let processes_ptr = processes.as_mut_ptr() as *mut c_void;

            // Second call to actually get the data
            if sysctl(mib.as_mut_ptr(), 3, processes_ptr, &mut size, ptr::null(), 0) < 0 {
                return Err(crate::Error::process_error(
                    Some(0u32), // Use 0 as PID when not available
                    "Failed to get process information",
                ));
            }

            // Set the length of the vector
            let count = size / mem::size_of::<kinfo_proc>();
            processes.set_len(count);

            // Convert to our Process struct format
            let mut result = Vec::with_capacity(count);

            for proc_info in processes {
                let pid = proc_info.kp_proc.p_pid;
                if pid <= 0 {
                    continue;
                }

                // Use the helper function from bindings
                let name = extract_proc_name(&proc_info);

                // Create a basic process with the information we have
                let process = Self::new(pid as u32, &name);

                result.push(process);
            }

            // Populate more detailed information for each process
            for process in &mut result {
                // Try to get additional information via libproc, but don't fail if we can't
                if let Ok(detailed) = Self::get_by_pid(process.pid) {
                    process.cpu_usage = detailed.cpu_usage;
                    process.memory_usage = detailed.memory_usage;
                    process.thread_count = detailed.thread_count;
                    process.is_suspended = detailed.is_suspended;
                }
            }

            Ok(result)
        }
    }

    /// Fallback method using libproc (the original implementation)
    fn get_all_via_libproc() -> Result<Vec<Self>> {
        // Use the listpids function for simplicity, handling deprecation warning
        #[allow(deprecated)]
        let pids = proc_pid::listpids(proc_pid::ProcType::ProcAllPIDS).map_err(|e| {
            crate::Error::process_error(Some(0u32), format!("Failed to list process IDs: {}", e))
        })?;

        let mut processes = Vec::with_capacity(pids.len());
        for pid in pids {
            if pid == 0 {
                continue;
            }

            match Self::get_by_pid(pid) {
                Ok(process) => processes.push(process),
                Err(_) => {}, // Skip processes we can't access
            }
        }

        Ok(processes)
    }

    /// Gets the parent process ID for a given PID
    pub fn get_parent_pid(pid: u32) -> Result<Option<u32>> {
        // Special case for PID 0 and 1
        if pid == 0 || pid == 1 {
            return Ok(None);
        }

        let proc_info =
            proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).map_err(|e| {
                crate::Error::process_error(Some(pid), format!("Failed to get process info: {}", e))
            })?;

        let ppid = proc_info.pbsd.pbi_ppid;

        // If parent PID is 0 or invalid, return None
        if ppid == 0 {
            return Ok(None);
        }

        Ok(Some(ppid as u32))
    }

    /// Gets the process start time
    pub fn get_process_start_time(pid: u32) -> Result<SystemTime> {
        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0)
            .map_err(|e| {
                crate::Error::process_error(Some(pid), format!("Failed to get process info: {}", e))
            })?;

        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return Err(crate::Error::process_error(Some(pid), "Invalid process start time"));
        };

        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return Err(crate::Error::process_error(
                    Some(pid),
                    "Process start time is in the future",
                ));
            },
            Err(_) => match now.duration_since(start_time) {
                Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                    return Err(crate::Error::process_error(
                        Some(pid),
                        "Process is unrealistically old",
                    ));
                },
                Ok(_) => (),
                Err(_) => {
                    return Err(crate::Error::process_error(
                        Some(pid),
                        "Failed to calculate process age",
                    ))
                },
            },
        }

        Ok(start_time)
    }

    /// Gets all child processes of a given PID
    pub fn get_child_processes(pid: u32) -> Result<Vec<Self>> {
        let all_processes = Self::get_all()?;

        let mut children = Vec::new();
        for process in all_processes {
            if let Ok(Some(parent_pid)) = Self::get_parent_pid(process.pid) {
                if parent_pid == pid {
                    children.push(process);
                }
            }
        }

        Ok(children)
    }

    /// Determines if this is a system process
    pub fn is_system_process(&self) -> bool {
        // Use the helper from bindings
        is_system_process(self.pid, &self.name)
    }

    /// Calculate CPU usage percentage based on total CPU time
    fn calculate_cpu_usage(pid: u32, total_cpu_time: u64) -> f64 {
        let mut history = get_cpu_history();
        let now = Instant::now();

        // Get previous measurement if available
        let cpu_usage = if let Some(&(prev_time, prev_cpu_time)) = history.get(&pid) {
            let time_delta = now.duration_since(prev_time).as_secs_f64();

            // Only calculate if we have a meaningful time difference
            if time_delta >= 0.1 {
                // Calculate CPU usage as percentage
                let cpu_time_delta = total_cpu_time.saturating_sub(prev_cpu_time) as f64;
                let usage = (cpu_time_delta / time_delta / 1_000_000.0) * 100.0;

                // Cap at 100% per logical CPU (though could be higher for multi-threaded processes)
                usage.min(800.0) // 800% cap assuming 8 cores max utilization
            } else {
                // Time delta too small, just return previous value or 0
                0.0
            }
        } else {
            // First measurement, can't calculate rate yet
            0.0
        };

        // Update history
        history.insert(pid, (now, total_cpu_time));

        // Clean up old history entries This is a simple approach - in a production system, you might want a more
        // sophisticated cleanup
        if history.len() > 1000 {
            // Get all PIDs we're tracking
            let pids: Vec<u32> = history.keys().copied().collect();

            // Check if each process still exists
            for pid in pids {
                if proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).is_err() {
                    history.remove(&pid);
                }
            }
        }

        cpu_usage
    }

    fn get_process_io_stats(pid: u32) -> Result<ProcessIOStats> {
        use pid_rusage::RUsageInfoV4;

        let rusage = pid_rusage::pidrusage::<RUsageInfoV4>(pid as i32).map_err(|e| {
            crate::Error::process_error(Some(pid), format!("Failed to get process I/O stats: {}", e))
        })?;

        Ok(ProcessIOStats {
            read_bytes: rusage.ri_diskio_bytesread,
            write_bytes: rusage.ri_diskio_byteswritten,
            // Calculate read/write counts based on typical block size (4KB)
            read_count: rusage.ri_diskio_bytesread / 4096,
            write_count: rusage.ri_diskio_byteswritten / 4096,
        })
    }

    fn get_io_stats(&self, rusage: &RUsageInfoV4) -> ProcessIOStats {
        ProcessIOStats {
            read_bytes: rusage.ri_diskio_bytesread,
            write_bytes: rusage.ri_diskio_byteswritten,
            // Calculate read/write counts based on typical block size (4KB)
            read_count: rusage.ri_diskio_bytesread / 4096,
            write_count: rusage.ri_diskio_byteswritten / 4096,
        }
    }

    fn get_process_info(pid: i32) -> Result<proc_info> {
        let mut info: proc_info = unsafe { std::mem::zeroed() };
        let info_size = std::mem::size_of::<proc_info>();

        let result = unsafe {
            proc_pidinfo(
                pid,
                PROC_PIDTASKINFO,
                0,
                &mut info as *mut _ as *mut c_void,
                info_size as i32,
            )
        };

        if result <= 0 {
            let error_message = Self::get_error_message(result);
            return Err(Error::process_error(Some(pid as u32), error_message.unwrap_or_else(|| "Failed to get process info".to_string())));
        }

        Ok(info)
    }

    fn get_error_message(result: i32) -> Option<String> {
        if result <= 0 {
            Some(format!("Process info error: {}", result))
        } else {
            None
        }
    }
}

impl ProcessMetricsStream {
    /// Creates a new ProcessMetricsStream for monitoring a specific process
    ///
    /// # Arguments
    /// * `pid` - The process ID to monitor
    /// * `interval` - The minimum duration between updates
    pub fn new(pid: u32, interval: Duration) -> Self {
        Self {
            pid,
            update_interval: interval,
            last_update: Instant::now(),
        }
    }

    /// Checks if enough time has elapsed and returns new process metrics if available
    ///
    /// Returns None if the update interval hasn't elapsed yet, or Some(Result<Process>)
    /// containing either the updated process metrics or an error if the process couldn't be read.
    pub fn next_update(&mut self) -> Option<Result<Process>> {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.last_update = now;
            Some(Process::get_by_pid(self.pid))
        } else {
            None
        }
    }
}
