use std::ffi::c_void;
/// # Process Monitoring
///
/// The process module provides comprehensive monitoring of processes running on macOS systems.
/// It offers ways to enumerate running processes, track their resource usage, obtain detailed
/// information about specific processes, and visualize process hierarchies.
///
/// ## Features
///
/// * **Process Information**: Access to process ID, name, parent PID, CPU usage, memory usage, etc.
/// * **Resource Monitoring**: Track CPU and memory usage, thread count, and process state
/// * **I/O Statistics**: Monitor disk read/write operations and bytes transferred
/// * **Process Relationships**: Examine parent-child relationships and process hierarchies
/// * **System Process Detection**: Identify system vs. user processes
///
/// ## Implementation Details
///
/// The module uses a hybrid approach for efficiency:
///
/// 1. **Bulk Retrieval**: Uses `sysctl` for efficient retrieval of all processes at once
/// 2. **Detailed Information**: Uses `libproc` for gathering detailed metrics about specific processes
/// 3. **Fallback Mechanism**: If one approach fails, the module automatically falls back to alternatives
///
/// ## Examples
///
/// ### Basic Process Information
///
/// ```rust
/// use darwin_metrics::process::Process;
///
/// // Get all running processes
/// let processes = Process::get_all().unwrap();
/// println!("Total processes: {}", processes.len());
///
/// // Get information about a specific process
/// let pid = std::process::id(); // Our own process ID
/// let process = Process::get_by_pid(pid).unwrap();
/// println!("Process: {} (PID: {})", process.name, process.pid);
/// println!("CPU Usage: {}%", process.cpu_usage);
/// println!("Memory Usage: {} bytes", process.memory_usage);
/// println!("Thread Count: {}", process.thread_count);
/// ```
///
/// ### Process Relationships
///
/// ```rust
/// use darwin_metrics::process::Process;
///
/// // Get child processes of a specific process
/// let pid = 1; // launchd process
/// let children = Process::get_child_processes(pid).unwrap();
/// println!("Process {} has {} children", pid, children.len());
///
/// // Get parent process
/// let my_pid = std::process::id();
/// if let Ok(Some(parent_pid)) = Process::get_parent_pid(my_pid) {
///     println!("My parent process has PID: {}", parent_pid);
/// }
/// ```
///
/// ### Process Monitoring
///
/// ```rust
/// use darwin_metrics::process::ProcessMetricsStream;
/// use std::time::Duration;
///
/// // Create a metrics stream for a specific process
/// let pid = std::process::id();
/// let mut stream = ProcessMetricsStream::new(pid, Duration::from_secs(1));
///
/// // Poll for updates
/// loop {
///     if let Some(result) = stream.next_update() {
///         match result {
///             Ok(process) => println!("CPU: {}%, Memory: {}", process.cpu_usage, process.memory_usage),
///             Err(e) => eprintln!("Error: {}", e),
///         }
///     }
///     // Do other work or sleep briefly
///     std::thread::sleep(Duration::from_millis(100));
/// }
/// ```
///
/// ## Performance Considerations
///
/// * The first call to `get_all()` might be slower as it initializes internal caches
/// * Subsequent calls will be faster due to optimized data structures
/// * Using `ProcessMetricsStream` is more efficient than repeatedly calling `get_by_pid()`
/// * The module cleans up its internal history tracking to prevent memory leaks
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime},
};

use async_trait::async_trait;
use libc::{PROC_PIDTASKINFO, proc_pidinfo};
use libproc::pid_rusage::RUsageInfoV4;
use libproc::{pid_rusage, proc_pid, task_info};

use crate::core::metrics::hardware::{
    ProcessIOMonitor, ProcessInfoMonitor, ProcessRelationshipMonitor, ProcessResourceMonitor,
};
use crate::error::{Error, Result};
use crate::utils::bindings::sysctl_constants::{CTL_KERN, KERN_PROC, KERN_PROC_ALL};
use crate::utils::bindings::{extract_proc_name, is_system_process, kinfo_proc, proc_info, sysctl};

/// Error types specific to process operations
#[derive(Debug, Clone)]
pub enum ProcessError {
    /// Error accessing process information
    AccessDenied { pid: Option<u32>, message: String },
    /// Process not found
    NotFound { pid: u32 },
    /// Invalid process data
    InvalidData {
        pid: Option<u32>,
        field: String,
        message: String,
    },
    /// System call error
    SystemCall { pid: Option<u32>, call: String, code: i32 },
    /// General process error
    General { pid: Option<u32>, message: String },
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessDenied { pid, message } => {
                write!(
                    f,
                    "Access denied for process{}: {}",
                    pid.map_or(String::new(), |p| format!(" {}", p)),
                    message
                )
            },
            Self::NotFound { pid } => write!(f, "Process not found: PID {}", pid),
            Self::InvalidData { pid, field, message } => {
                write!(
                    f,
                    "Invalid {} for process{}: {}",
                    field,
                    pid.map_or(String::new(), |p| format!(" {}", p)),
                    message
                )
            },
            Self::SystemCall { pid, call, code } => {
                write!(
                    f,
                    "System call '{}' failed for process{} with code {}",
                    call,
                    pid.map_or(String::new(), |p| format!(" {}", p)),
                    code
                )
            },
            Self::General { pid, message } => {
                write!(
                    f,
                    "Process error{}: {}",
                    pid.map_or(String::new(), |p| format!(" (PID {})", p)),
                    message
                )
            },
        }
    }
}

impl From<ProcessError> for Error {
    fn from(err: ProcessError) -> Self {
        match err {
            ProcessError::AccessDenied { pid, message } => Error::process_error(pid, message),
            ProcessError::NotFound { pid } => Error::process_error(Some(pid), format!("Process not found: {}", pid)),
            ProcessError::InvalidData { pid, field: _, message } => Error::process_error(pid, message),
            ProcessError::SystemCall { pid, call: _, code } => {
                Error::process_error(pid, format!("System call failed with code {}", code))
            },
            ProcessError::General { pid, message } => Error::process_error(pid, message),
        }
    }
}

/// Helper function to create a process error and convert it to the crate's Error type
fn process_error<T>(error: ProcessError) -> Result<T> {
    Err(error.into())
}

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

/// CPU usage history tracker
///
/// This struct manages the history of CPU usage measurements for processes,
/// providing efficient storage and automatic cleanup of stale entries.
#[derive(Debug)]
struct CpuHistoryTracker {
    /// Map of process ID to (timestamp, CPU time) pairs
    history: HashMap<u32, (Instant, u64)>,
    /// Maximum number of processes to track before cleanup
    max_processes: usize,
    /// Maximum age of entries before they're considered stale (in seconds)
    max_age_secs: u64,
    /// Last time the history was cleaned up
    last_cleanup: Instant,
}

impl CpuHistoryTracker {
    /// Create a new CPU history tracker
    fn new(max_processes: usize, max_age_secs: u64) -> Self {
        Self {
            history: HashMap::new(),
            max_processes,
            max_age_secs,
            last_cleanup: Instant::now(),
        }
    }

    /// Get the CPU time for a process, if available
    fn get(&self, pid: u32) -> Option<&(Instant, u64)> {
        self.history.get(&pid)
    }

    /// Update the CPU time for a process
    fn update(&mut self, pid: u32, cpu_time: u64) {
        let now = Instant::now();
        self.history.insert(pid, (now, cpu_time));

        // Check if we need to clean up
        if self.history.len() > self.max_processes || now.duration_since(self.last_cleanup).as_secs() > 60 {
            self.cleanup();
        }
    }

    /// Clean up stale entries
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.last_cleanup = now;

        // Remove entries for processes that no longer exist
        let pids: Vec<u32> = self.history.keys().copied().collect();
        for pid in pids {
            // Check if process still exists
            if proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).is_err() {
                self.history.remove(&pid);
                continue;
            }

            // Check if entry is too old
            if let Some((timestamp, _)) = self.history.get(&pid) {
                if now.duration_since(*timestamp).as_secs() > self.max_age_secs {
                    self.history.remove(&pid);
                }
            }
        }
    }
}

use std::sync::Mutex;

use once_cell::sync::Lazy as SyncLazy;

/// Static CPU history tracker instance
static CPU_HISTORY: SyncLazy<Mutex<CpuHistoryTracker>> = SyncLazy::new(
    || Mutex::new(CpuHistoryTracker::new(1000, 3600)), // Track up to 1000 processes, entries expire after 1 hour
);

/// Get CPU history tracking instance
#[allow(clippy::disallowed_methods)]
fn get_cpu_history() -> std::sync::MutexGuard<'static, CpuHistoryTracker> {
    CPU_HISTORY.lock().unwrap()
}

/// Process information and monitoring functionality
///
/// This struct represents a process in the system and provides access to its metrics and status information.
/// It provides methods for retrieving process details, monitoring resource usage, and examining process relationships.
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::process::Process;
///
/// // Get information about the current process
/// let pid = std::process::id();
/// let process = Process::get_by_pid(pid).unwrap();
///
/// println!("Process: {} (PID: {})", process.name, process.pid);
/// println!("CPU Usage: {}%", process.cpu_usage);
/// println!("Memory Usage: {} bytes", process.memory_usage);
/// println!("Thread Count: {}", process.thread_count);
/// println!("Uptime: {:?}", process.uptime);
/// println!("I/O Stats: {:?}", process.io_stats);
/// ```
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
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::process::ProcessMetricsStream;
/// use std::time::Duration;
///
/// // Create a metrics stream for a specific process
/// let pid = std::process::id();
/// let mut stream = ProcessMetricsStream::new(pid, Duration::from_secs(1));
///
/// // Poll for updates
/// loop {
///     if let Some(result) = stream.next_update() {
///         match result {
///             Ok(process) => println!("CPU: {}%, Memory: {}", process.cpu_usage, process.memory_usage),
///             Err(e) => eprintln!("Error: {}", e),
///         }
///     }
///     // Do other work or sleep briefly
///     std::thread::sleep(Duration::from_millis(100));
/// }
/// ```
///
/// # Performance Considerations
///
/// * Using a `ProcessMetricsStream` is more efficient than repeatedly calling `Process::get_by_pid()`
/// * The stream respects the update interval, preventing excessive system calls
/// * Consider using a longer interval (e.g., 1-5 seconds) for monitoring many processes
#[derive(Debug, Clone)]
pub struct ProcessMetricsStream {
    /// Process ID to monitor
    pid: u32,
    /// Minimum duration between updates
    update_interval: Duration,
    /// Timestamp of the last update
    last_update: Instant,
}

/// Monitor for process information
pub struct ProcessInfoMonitorImpl {
    process: Process,
}

/// Monitor for process resource usage
pub struct ProcessResourceMonitorImpl {
    process: Process,
}

/// Monitor for process I/O operations
pub struct ProcessIOMonitorImpl {
    process: Process,
    last_update: Instant,
    last_read_bytes: u64,
    last_write_bytes: u64,
}

/// Monitor for process relationships
pub struct ProcessRelationshipMonitorImpl {
    process: Process,
}

impl Process {
    /// Creates a new Process instance with basic information
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID
    /// * `name` - The process name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// let process = Process::new(1, "launchd");
    /// assert_eq!(process.pid, 1);
    /// assert_eq!(process.name, "launchd");
    /// ```
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
    ///
    /// This method retrieves detailed information about a process, including its resource usage,
    /// I/O statistics, and status information.
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID to retrieve information for
    ///
    /// # Returns
    ///
    /// A `Result` containing either the `Process` instance or an error if the process couldn't be found
    /// or accessed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// // Get information about the current process
    /// let pid = std::process::id();
    /// match Process::get_by_pid(pid) {
    ///     Ok(process) => println!("Process: {} (PID: {})", process.name, process.pid),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The process doesn't exist
    /// - The caller doesn't have permission to access the process
    /// - The process information is invalid or corrupted
    pub fn get_by_pid(pid: u32) -> Result<Self> {
        let name = libproc::proc_pid::name(pid as i32).map_err(|e| ProcessError::AccessDenied {
            pid: Some(pid),
            message: format!("Failed to get process name: {}", e),
        })?;

        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).map_err(|e| {
            ProcessError::AccessDenied {
                pid: Some(pid),
                message: format!("Failed to get process info: {}", e),
            }
        })?;

        // Validate and calculate process start time
        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return process_error(ProcessError::InvalidData {
                pid: Some(pid),
                field: "start_time".to_string(),
                message: "Invalid process start time".to_string(),
            });
        };

        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return process_error(ProcessError::InvalidData {
                    pid: Some(pid),
                    field: "start_time".to_string(),
                    message: "Process start time is in the future".to_string(),
                });
            },
            Err(_) => match now.duration_since(start_time) {
                Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                    return process_error(ProcessError::InvalidData {
                        pid: Some(pid),
                        field: "start_time".to_string(),
                        message: "Process is unrealistically old".to_string(),
                    });
                },
                Ok(_) => (),
                Err(_) => {
                    return process_error(ProcessError::General {
                        pid: Some(pid),
                        message: "Failed to calculate process age".to_string(),
                    });
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
    ///
    /// This method retrieves a list of all processes running on the system, with basic information
    /// about each process. It uses an efficient approach that first tries to get all processes via
    /// sysctl, and falls back to libproc if that fails.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a vector of `Process` instances or an error if the process list
    /// couldn't be retrieved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// match Process::get_all() {
    ///     Ok(processes) => {
    ///         println!("Total processes: {}", processes.len());
    ///         for process in processes.iter().take(5) {
    ///             println!("- {} (PID: {})", process.name, process.pid);
    ///         }
    ///     },
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// This method can be expensive, especially on systems with many processes. Consider caching
    /// the results if you need to call it frequently.
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
        use std::os::raw::c_void;
        use std::{mem, ptr};

        unsafe {
            // First call to get the size of the buffer needed
            let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0, 0, 0];
            let mut size: usize = 0;

            if sysctl(mib.as_mut_ptr(), 3, ptr::null_mut(), &mut size, ptr::null(), 0) < 0 {
                return process_error(ProcessError::SystemCall {
                    pid: None,
                    call: "sysctl(KERN_PROC_ALL)".to_string(),
                    code: std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
                });
            }

            // Calculate number of processes
            let count = size / mem::size_of::<kinfo_proc>();

            // Allocate buffer
            let mut processes = Vec::<kinfo_proc>::with_capacity(count);
            let processes_ptr = processes.as_mut_ptr() as *mut c_void;

            // Second call to actually get the data
            if sysctl(mib.as_mut_ptr(), 3, processes_ptr, &mut size, ptr::null(), 0) < 0 {
                return process_error(ProcessError::SystemCall {
                    pid: None,
                    call: "sysctl(KERN_PROC_ALL)".to_string(),
                    code: std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
                });
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
        let pids = proc_pid::listpids(proc_pid::ProcType::ProcAllPIDS)
            .map_err(|e| crate::Error::process_error(Some(0u32), format!("Failed to list process IDs: {}", e)))?;

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
    ///
    /// This method retrieves the parent process ID (PPID) for a specified process.
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID to get the parent for
    ///
    /// # Returns
    ///
    /// A `Result` containing either `Some(parent_pid)` or `None` if the process has no parent
    /// (e.g., for PID 0 or 1), or an error if the process information couldn't be retrieved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// let pid = std::process::id();
    /// match Process::get_parent_pid(pid) {
    ///     Ok(Some(parent_pid)) => println!("Parent PID: {}", parent_pid),
    ///     Ok(None) => println!("No parent process (likely PID 1)"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn get_parent_pid(pid: u32) -> Result<Option<u32>> {
        // Special case for PID 0 and 1
        if pid == 0 || pid == 1 {
            return Ok(None);
        }

        let proc_info =
            proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).map_err(|e| ProcessError::AccessDenied {
                pid: Some(pid),
                message: format!("Failed to get process info: {}", e),
            })?;

        let ppid = proc_info.pbsd.pbi_ppid;

        // If parent PID is 0 or invalid, return None
        if ppid == 0 {
            return Ok(None);
        }

        Ok(Some(ppid as u32))
    }

    /// Gets the process start time
    ///
    /// This method retrieves the time when a process was started.
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID to get the start time for
    ///
    /// # Returns
    ///
    /// A `Result` containing either the `SystemTime` when the process started or an error if
    /// the process information couldn't be retrieved or the start time is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let pid = std::process::id();
    /// match Process::get_process_start_time(pid) {
    ///     Ok(start_time) => {
    ///         let since_epoch = start_time.duration_since(UNIX_EPOCH).unwrap();
    ///         println!("Process started at: {} seconds since epoch", since_epoch.as_secs());
    ///     },
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn get_process_start_time(pid: u32) -> Result<SystemTime> {
        let proc_info = libproc::proc_pid::pidinfo::<task_info::TaskAllInfo>(pid as i32, 0).map_err(|e| {
            ProcessError::AccessDenied {
                pid: Some(pid),
                message: format!("Failed to get process info: {}", e),
            }
        })?;

        let start_time = if proc_info.pbsd.pbi_start_tvsec > 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(proc_info.pbsd.pbi_start_tvsec as u64)
        } else {
            return process_error(ProcessError::InvalidData {
                pid: Some(pid),
                field: "start_time".to_string(),
                message: "Invalid process start time".to_string(),
            });
        };

        let now = SystemTime::now();
        match start_time.duration_since(now) {
            Ok(_) => {
                return process_error(ProcessError::InvalidData {
                    pid: Some(pid),
                    field: "start_time".to_string(),
                    message: "Process start time is in the future".to_string(),
                });
            },
            Err(_) => match now.duration_since(start_time) {
                Ok(age) if age > Duration::from_secs(60 * 60 * 24 * 365 * 50) => {
                    return process_error(ProcessError::InvalidData {
                        pid: Some(pid),
                        field: "start_time".to_string(),
                        message: "Process is unrealistically old".to_string(),
                    });
                },
                Ok(_) => (),
                Err(_) => {
                    return process_error(ProcessError::General {
                        pid: Some(pid),
                        message: "Failed to calculate process age".to_string(),
                    });
                },
            },
        }

        Ok(start_time)
    }

    /// Gets all child processes of a given PID
    ///
    /// This method retrieves a list of all processes that have the specified process as their parent.
    ///
    /// # Arguments
    ///
    /// * `pid` - The parent process ID to find children for
    ///
    /// # Returns
    ///
    /// A `Result` containing either a vector of child `Process` instances or an error if the
    /// process list couldn't be retrieved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// // Get children of the init process (launchd, PID 1)
    /// match Process::get_child_processes(1) {
    ///     Ok(children) => {
    ///         println!("Process 1 has {} children", children.len());
    ///         for child in children.iter().take(5) {
    ///             println!("- {} (PID: {})", child.name, child.pid);
    ///         }
    ///     },
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// This method calls `get_all()` internally, which can be expensive on systems with many processes.
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
    ///
    /// This method checks if the process is a system process based on its name and PID.
    ///
    /// # Returns
    ///
    /// `true` if the process is a system process, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::Process;
    ///
    /// let pid = std::process::id();
    /// if let Ok(process) = Process::get_by_pid(pid) {
    ///     if process.is_system_process() {
    ///         println!("This is a system process");
    ///     } else {
    ///         println!("This is a user process");
    ///     }
    /// }
    /// ```
    pub fn is_system_process(&self) -> bool {
        // Use the helper from bindings
        is_system_process(self.pid, &self.name)
    }

    /// Calculate CPU usage percentage based on total CPU time
    fn calculate_cpu_usage(pid: u32, total_cpu_time: u64) -> f64 {
        let mut history = get_cpu_history();
        let now = Instant::now();

        // Get previous measurement if available
        let cpu_usage = if let Some(&(prev_time, prev_cpu_time)) = history.get(pid) {
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
        history.update(pid, total_cpu_time);

        cpu_usage
    }

    fn get_process_io_stats(pid: u32) -> Result<ProcessIOStats> {
        use pid_rusage::RUsageInfoV4;

        let rusage = pid_rusage::pidrusage::<RUsageInfoV4>(pid as i32).map_err(|e| ProcessError::AccessDenied {
            pid: Some(pid),
            message: format!("Failed to get process I/O stats: {}", e),
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
            let error_code = std::io::Error::last_os_error().raw_os_error().unwrap_or(-1);
            return process_error(ProcessError::SystemCall {
                pid: Some(pid as u32),
                call: "proc_pidinfo".to_string(),
                code: error_code,
            });
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

    /// Returns a monitor for process information
    pub fn info_monitor(&self) -> ProcessInfoMonitorImpl {
        ProcessInfoMonitorImpl { process: self.clone() }
    }

    /// Returns a monitor for process resource usage
    pub fn resource_monitor(&self) -> ProcessResourceMonitorImpl {
        ProcessResourceMonitorImpl { process: self.clone() }
    }

    /// Returns a monitor for process I/O operations
    pub fn io_monitor(&self) -> ProcessIOMonitorImpl {
        ProcessIOMonitorImpl {
            process: self.clone(),
            last_update: Instant::now(),
            last_read_bytes: 0,
            last_write_bytes: 0,
        }
    }

    /// Returns a monitor for process relationships
    pub fn relationship_monitor(&self) -> ProcessRelationshipMonitorImpl {
        ProcessRelationshipMonitorImpl { process: self.clone() }
    }
}

impl ProcessMetricsStream {
    /// Creates a new ProcessMetricsStream for monitoring a specific process
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID to monitor
    /// * `interval` - The minimum duration between updates
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::ProcessMetricsStream;
    /// use std::time::Duration;
    ///
    /// let pid = std::process::id();
    /// let stream = ProcessMetricsStream::new(pid, Duration::from_secs(1));
    /// ```
    pub fn new(pid: u32, interval: Duration) -> Self {
        Self {
            pid,
            update_interval: interval,
            last_update: Instant::now(),
        }
    }

    /// Checks if enough time has elapsed and returns new process metrics if available
    ///
    /// Returns None if the update interval hasn't elapsed yet, or Some(`Result<Process>`)
    /// containing either the updated process metrics or an error if the process couldn't be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::ProcessMetricsStream;
    /// use std::time::Duration;
    ///
    /// let mut stream = ProcessMetricsStream::new(std::process::id(), Duration::from_secs(1));
    ///
    /// // Check for an update
    /// if let Some(result) = stream.next_update() {
    ///     match result {
    ///         Ok(process) => println!("Process updated: {}", process.name),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// } else {
    ///     println!("No update available yet");
    /// }
    /// ```
    pub fn next_update(&mut self) -> Option<Result<Process>> {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.last_update = now;
            Some(Process::get_by_pid(self.pid))
        } else {
            None
        }
    }

    /// Returns the process ID being monitored
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::ProcessMetricsStream;
    /// use std::time::Duration;
    ///
    /// let pid = std::process::id();
    /// let stream = ProcessMetricsStream::new(pid, Duration::from_secs(1));
    /// assert_eq!(stream.pid(), pid);
    /// ```
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Returns the update interval
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::ProcessMetricsStream;
    /// use std::time::Duration;
    ///
    /// let interval = Duration::from_secs(2);
    /// let stream = ProcessMetricsStream::new(std::process::id(), interval);
    /// assert_eq!(stream.update_interval(), interval);
    /// ```
    pub fn update_interval(&self) -> Duration {
        self.update_interval
    }

    /// Changes the update interval
    ///
    /// # Arguments
    ///
    /// * `interval` - The new minimum duration between updates
    ///
    /// # Examples
    ///
    /// ```rust
    /// use darwin_metrics::process::ProcessMetricsStream;
    /// use std::time::Duration;
    ///
    /// let mut stream = ProcessMetricsStream::new(std::process::id(), Duration::from_secs(1));
    /// stream.set_update_interval(Duration::from_secs(5));
    /// assert_eq!(stream.update_interval(), Duration::from_secs(5));
    /// ```
    pub fn set_update_interval(&mut self, interval: Duration) {
        self.update_interval = interval;
    }
}

#[async_trait]
impl ProcessInfoMonitor for ProcessInfoMonitorImpl {
    async fn pid(&self) -> Result<u32> {
        Ok(self.process.pid)
    }

    async fn name(&self) -> Result<String> {
        Ok(self.process.name.clone())
    }

    async fn parent_pid(&self) -> Result<Option<u32>> {
        Process::get_parent_pid(self.process.pid)
    }

    async fn start_time(&self) -> Result<SystemTime> {
        Process::get_process_start_time(self.process.pid)
    }

    async fn is_system_process(&self) -> Result<bool> {
        Ok(self.process.is_system_process())
    }
}

#[async_trait]
impl ProcessResourceMonitor for ProcessResourceMonitorImpl {
    async fn cpu_usage(&self) -> Result<f64> {
        Ok(self.process.cpu_usage)
    }

    async fn memory_usage(&self) -> Result<u64> {
        Ok(self.process.memory_usage)
    }

    async fn thread_count(&self) -> Result<u32> {
        Ok(self.process.thread_count)
    }

    async fn is_suspended(&self) -> Result<bool> {
        Ok(self.process.is_suspended)
    }
}

#[async_trait]
impl ProcessIOMonitor for ProcessIOMonitorImpl {
    async fn bytes_read(&self) -> Result<u64> {
        Ok(self.process.io_stats.read_bytes)
    }

    async fn bytes_written(&self) -> Result<u64> {
        Ok(self.process.io_stats.write_bytes)
    }

    async fn read_operations(&self) -> Result<u64> {
        Ok(self.process.io_stats.read_count)
    }

    async fn write_operations(&self) -> Result<u64> {
        Ok(self.process.io_stats.write_count)
    }

    async fn read_rate(&self) -> Result<f64> {
        let current_stats = Process::get_process_io_stats(self.process.pid)?;
        let elapsed = self.last_update.elapsed().as_secs_f64();

        if elapsed > 0.0 {
            let rate = (current_stats.read_bytes - self.last_read_bytes) as f64 / elapsed;
            Ok(rate)
        } else {
            Ok(0.0)
        }
    }

    async fn write_rate(&self) -> Result<f64> {
        let current_stats = Process::get_process_io_stats(self.process.pid)?;
        let elapsed = self.last_update.elapsed().as_secs_f64();

        if elapsed > 0.0 {
            let rate = (current_stats.write_bytes - self.last_write_bytes) as f64 / elapsed;
            Ok(rate)
        } else {
            Ok(0.0)
        }
    }
}

#[async_trait]
impl ProcessRelationshipMonitor for ProcessRelationshipMonitorImpl {
    async fn child_pids(&self) -> Result<Vec<u32>> {
        let children = Process::get_child_processes(self.process.pid)?;
        Ok(children.into_iter().map(|p| p.pid).collect())
    }

    async fn sibling_pids(&self) -> Result<Vec<u32>> {
        if let Ok(Some(parent_pid)) = Process::get_parent_pid(self.process.pid) {
            let siblings = Process::get_child_processes(parent_pid)?;
            Ok(siblings
                .into_iter()
                .filter(|p| p.pid != self.process.pid)
                .map(|p| p.pid)
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn tree_depth(&self) -> Result<u32> {
        let mut depth = 0;
        let mut current_pid = self.process.pid;

        while let Ok(Some(parent_pid)) = Process::get_parent_pid(current_pid) {
            depth += 1;
            if parent_pid == 1 || parent_pid == 0 {
                break;
            }
            current_pid = parent_pid;
        }

        Ok(depth)
    }

    async fn process_group_id(&self) -> Result<u32> {
        let proc_info = proc_pid::pidinfo::<task_info::TaskAllInfo>(self.process.pid as i32, 0).map_err(|e| {
            ProcessError::AccessDenied {
                pid: Some(self.process.pid),
                message: format!("Failed to get process info: {}", e),
            }
        })?;

        Ok(proc_info.pbsd.pbi_pgid as u32)
    }
}
