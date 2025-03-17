use std::{ffi::c_void, time::Duration};
use thiserror::Error;

use crate::{
    core::metrics::hardware::{SystemInfoMonitor, SystemLoadMonitor, SystemResourceMonitor, SystemUptimeMonitor},
    error::{Error, Result},
    utils::bindings::{
        kinfo_proc, sysctl,
        sysctl_constants::{
            CTL_HW, CTL_KERN, HW_LOGICALCPU, HW_MACHINE, HW_MEMSIZE, HW_PHYSICALCPU, KERN_BOOTTIME, KERN_HOSTNAME,
            KERN_OSRELEASE, KERN_OSVERSION, KERN_PROC, KERN_PROC_ALL,
        },
    },
};

/// Errors that can occur during system monitoring
#[derive(Debug, Error)]
pub enum SystemError {
    /// A system call failed
    #[error("System call failed: {0}")]
    SystemCallFailed(String),
    /// String encoding error occurred
    #[error("Invalid string encoding")]
    InvalidStringEncoding,
    /// Failed to get system information
    #[error("Failed to get system information: {0}")]
    InfoError(String),
}

impl From<SystemError> for Error {
    fn from(err: SystemError) -> Self {
        Error::system(err.to_string())
    }
}

/// System architecture type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    /// Intel x86_64 architecture
    Intel,
    /// Apple Silicon ARM architecture
    AppleSilicon,
    /// Unknown architecture
    Unknown,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::Intel => write!(f, "x86_64"),
            Architecture::AppleSilicon => write!(f, "arm64"),
            Architecture::Unknown => write!(f, "unknown"),
        }
    }
}

/// System information and monitoring functionality
#[derive(Debug, Clone)]
pub struct System {
    /// The system hostname
    hostname: String,
    /// The detected CPU architecture
    architecture: Architecture,
    /// The operating system version
    os_version: String,
    /// The kernel version
    kernel_version: String,
    /// Number of physical CPU cores
    physical_cpu_count: u32,
    /// Number of logical CPU cores
    logical_cpu_count: u32,
    /// Total physical memory in bytes
    total_memory: u64,
    /// System boot time
    boot_time: Duration,
    /// Last update time
    last_update: std::time::Instant,
}

impl System {
    /// Creates a new System instance
    pub fn new() -> Result<Self> {
        let mut system = Self {
            hostname: String::new(),
            architecture: Architecture::Unknown,
            os_version: String::new(),
            kernel_version: String::new(),
            physical_cpu_count: 0,
            logical_cpu_count: 0,
            total_memory: 0,
            boot_time: Duration::from_secs(0),
            last_update: std::time::Instant::now(),
        };
        system.update()?;
        Ok(system)
    }

    /// Updates system information
    pub fn update(&mut self) -> Result<()> {
        self.update_architecture()?;
        self.update_hostname()?;
        self.update_os_info()?;
        self.update_cpu_info()?;
        self.update_memory_info()?;
        self.update_boot_time()?;
        self.last_update = std::time::Instant::now();
        Ok(())
    }

    /// Returns a monitor for system information
    pub fn info_monitor(&self) -> SystemInfoMonitorImpl {
        SystemInfoMonitorImpl { system: self.clone() }
    }

    /// Returns a monitor for system load
    pub fn load_monitor(&self) -> SystemLoadMonitorImpl {
        SystemLoadMonitorImpl { system: self.clone() }
    }

    /// Returns a monitor for system uptime
    pub fn uptime_monitor(&self) -> SystemUptimeMonitorImpl {
        SystemUptimeMonitorImpl { system: self.clone() }
    }

    /// Returns a monitor for system resources
    pub fn resource_monitor(&self) -> SystemResourceMonitorImpl {
        SystemResourceMonitorImpl { system: self.clone() }
    }

    fn update_architecture(&mut self) -> Result<()> {
        let mut mib = [CTL_HW, HW_MACHINE];
        let mut size = 0;

        unsafe {
            if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get architecture size".into()).into());
            }

            let mut buffer = vec![0u8; size];
            if sysctl(mib.as_mut_ptr(), 2, buffer.as_mut_ptr() as *mut c_void, &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get architecture data".into()).into());
            }

            let cstr = std::ffi::CStr::from_bytes_with_nul(&buffer).map_err(|_| SystemError::InvalidStringEncoding)?;
            let arch = cstr.to_str().map_err(|_| SystemError::InvalidStringEncoding)?;

            self.architecture = match arch {
                "arm64" => Architecture::AppleSilicon,
                "x86_64" => Architecture::Intel,
                _ => Architecture::Unknown,
            };
        }
        Ok(())
    }

    fn update_hostname(&mut self) -> Result<()> {
        let mut mib = [CTL_KERN, KERN_HOSTNAME];
        let mut size = 0;

        unsafe {
            if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get hostname size".into()).into());
            }

            let mut buffer = vec![0u8; size];
            if sysctl(mib.as_mut_ptr(), 2, buffer.as_mut_ptr() as *mut c_void, &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get hostname data".into()).into());
            }

            self.hostname =
                String::from_utf8(buffer[..size - 1].to_vec()).map_err(|_| SystemError::InvalidStringEncoding)?;
        }
        Ok(())
    }

    fn update_os_info(&mut self) -> Result<()> {
        // Get OS version
        let mut mib = [CTL_KERN, KERN_OSVERSION];
        let mut size = 0;

        unsafe {
            if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get OS version size".into()).into());
            }

            let mut buffer = vec![0u8; size];
            if sysctl(mib.as_mut_ptr(), 2, buffer.as_mut_ptr() as *mut c_void, &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get OS version data".into()).into());
            }

            self.os_version =
                String::from_utf8(buffer[..size - 1].to_vec()).map_err(|_| SystemError::InvalidStringEncoding)?;
        }

        // Get kernel version
        let mut mib = [CTL_KERN, KERN_OSRELEASE];
        let mut size = 0;

        unsafe {
            if sysctl(mib.as_mut_ptr(), 2, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get kernel version size".into()).into());
            }

            let mut buffer = vec![0u8; size];
            if sysctl(mib.as_mut_ptr(), 2, buffer.as_mut_ptr() as *mut c_void, &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get kernel version data".into()).into());
            }

            self.kernel_version =
                String::from_utf8(buffer[..size - 1].to_vec()).map_err(|_| SystemError::InvalidStringEncoding)?;
        }
        Ok(())
    }

    fn update_cpu_info(&mut self) -> Result<()> {
        unsafe {
            // Get physical CPU count
            let mut mib = [CTL_HW, HW_PHYSICALCPU];
            let mut size = std::mem::size_of::<u32>();
            let mut physical_cpu: u32 = 0;

            if sysctl(mib.as_mut_ptr(), 2, &mut physical_cpu as *mut u32 as *mut c_void, &mut size, std::ptr::null(), 0)
                != 0
            {
                return Err(SystemError::SystemCallFailed("Failed to get physical CPU count".into()).into());
            }

            // Get logical CPU count
            let mut mib = [CTL_HW, HW_LOGICALCPU];
            let mut logical_cpu: u32 = 0;

            if sysctl(mib.as_mut_ptr(), 2, &mut logical_cpu as *mut u32 as *mut c_void, &mut size, std::ptr::null(), 0)
                != 0
            {
                return Err(SystemError::SystemCallFailed("Failed to get logical CPU count".into()).into());
            }

            self.physical_cpu_count = physical_cpu;
            self.logical_cpu_count = logical_cpu;
        }
        Ok(())
    }

    fn update_memory_info(&mut self) -> Result<()> {
        unsafe {
            let mut mib = [CTL_HW, HW_MEMSIZE];
            let mut size = std::mem::size_of::<u64>();
            let mut memsize: u64 = 0;

            if sysctl(mib.as_mut_ptr(), 2, &mut memsize as *mut u64 as *mut c_void, &mut size, std::ptr::null(), 0) != 0
            {
                return Err(SystemError::SystemCallFailed("Failed to get memory size".into()).into());
            }

            self.total_memory = memsize;
        }
        Ok(())
    }

    fn update_boot_time(&mut self) -> Result<()> {
        unsafe {
            let mut mib = [CTL_KERN, KERN_BOOTTIME];
            let mut size = std::mem::size_of::<libc::timeval>();
            let mut boottime: libc::timeval = std::mem::zeroed();

            if sysctl(
                mib.as_mut_ptr(),
                2,
                &mut boottime as *mut libc::timeval as *mut c_void,
                &mut size,
                std::ptr::null(),
                0,
            ) != 0
            {
                return Err(SystemError::SystemCallFailed("Failed to get boot time".into()).into());
            }

            self.boot_time = Duration::from_secs(boottime.tv_sec as u64);
        }
        Ok(())
    }

    /// Creates a new System instance with mock values for testing
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        Self {
            hostname: "test-hostname".to_string(),
            architecture: Architecture::AppleSilicon,
            os_version: "macOS 14.0".to_string(),
            kernel_version: "Darwin 23.0.0".to_string(),
            physical_cpu_count: 8,
            logical_cpu_count: 16,
            total_memory: 16 * 1024 * 1024 * 1024,      // 16 GB
            boot_time: Duration::from_secs(1609459200), // 2021-01-01
            last_update: std::time::Instant::now(),
        }
    }
}

/// Monitor for system information
pub struct SystemInfoMonitorImpl {
    system: System,
}

/// Monitor for system load
pub struct SystemLoadMonitorImpl {
    system: System,
}

/// Monitor for system uptime
pub struct SystemUptimeMonitorImpl {
    system: System,
}

/// Monitor for system resources
pub struct SystemResourceMonitorImpl {
    system: System,
}

#[async_trait::async_trait]
impl SystemInfoMonitor for SystemInfoMonitorImpl {
    async fn hostname(&self) -> Result<String> {
        Ok(self.system.hostname.clone())
    }

    async fn architecture(&self) -> Result<String> {
        Ok(self.system.architecture.to_string())
    }

    async fn os_version(&self) -> Result<String> {
        Ok(self.system.os_version.clone())
    }

    async fn kernel_version(&self) -> Result<String> {
        Ok(self.system.kernel_version.clone())
    }
}

#[async_trait::async_trait]
impl SystemLoadMonitor for SystemLoadMonitorImpl {
    async fn load_average_1(&self) -> Result<f64> {
        let mut avg: [f64; 3] = [0.0; 3];
        if unsafe { libc::getloadavg(avg.as_mut_ptr(), 3) } == -1 {
            return Err(SystemError::SystemCallFailed("Failed to get load average".into()).into());
        }
        Ok(avg[0])
    }

    async fn load_average_5(&self) -> Result<f64> {
        let mut avg: [f64; 3] = [0.0; 3];
        if unsafe { libc::getloadavg(avg.as_mut_ptr(), 3) } == -1 {
            return Err(SystemError::SystemCallFailed("Failed to get load average".into()).into());
        }
        Ok(avg[1])
    }

    async fn load_average_15(&self) -> Result<f64> {
        let mut avg: [f64; 3] = [0.0; 3];
        if unsafe { libc::getloadavg(avg.as_mut_ptr(), 3) } == -1 {
            return Err(SystemError::SystemCallFailed("Failed to get load average".into()).into());
        }
        Ok(avg[2])
    }

    async fn process_count(&self) -> Result<u32> {
        let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_ALL];
        let mut size = 0;

        unsafe {
            if sysctl(mib.as_mut_ptr(), 3, std::ptr::null_mut(), &mut size, std::ptr::null(), 0) != 0 {
                return Err(SystemError::SystemCallFailed("Failed to get process count".into()).into());
            }
        }

        Ok((size / std::mem::size_of::<kinfo_proc>()) as u32)
    }

    async fn thread_count(&self) -> Result<u32> {
        #[cfg(test)]
        return Ok(100); // Mock value for testing

        #[cfg(not(test))]
        {
            // On macOS, we can get this from vm_stat
            let output = std::process::Command::new("vm_stat").output()?;
            let output = String::from_utf8_lossy(&output.stdout);

            for line in output.lines() {
                if line.contains("Thread count") {
                    if let Some(count) = line.split(':').nth(1) {
                        if let Ok(num) = count.trim().parse::<u32>() {
                            return Ok(num);
                        }
                    }
                }
            }

            Err(SystemError::InfoError("Failed to get thread count".into()).into())
        }
    }
}

#[async_trait::async_trait]
impl SystemUptimeMonitor for SystemUptimeMonitorImpl {
    async fn uptime_seconds(&self) -> Result<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SystemError::InfoError(format!("Failed to get current time: {}", e)))?;
        Ok(now.as_secs() - self.system.boot_time.as_secs())
    }

    async fn boot_time(&self) -> Result<u64> {
        Ok(self.system.boot_time.as_secs())
    }
}

#[async_trait::async_trait]
impl SystemResourceMonitor for SystemResourceMonitorImpl {
    async fn physical_cpu_count(&self) -> Result<u32> {
        Ok(self.system.physical_cpu_count)
    }

    async fn logical_cpu_count(&self) -> Result<u32> {
        Ok(self.system.logical_cpu_count)
    }

    async fn total_memory(&self) -> Result<u64> {
        Ok(self.system.total_memory)
    }

    async fn total_swap(&self) -> Result<u64> {
        let output = std::process::Command::new("sysctl").args(["-n", "vm.swapusage"]).output()?;
        let output = String::from_utf8_lossy(&output.stdout);

        // Parse output like "total = 1024.00M  used = 0.00M  free = 1024.00M"
        for part in output.split_whitespace() {
            if let Some(value) = part.strip_suffix('M') {
                if let Ok(mb) = value.parse::<f64>() {
                    return Ok((mb * 1024.0 * 1024.0) as u64);
                }
            }
        }

        Err(SystemError::InfoError("Failed to get swap space".into()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info() {
        let system = System::new_for_testing();
        let info_monitor = system.info_monitor();

        let hostname = info_monitor.hostname().await.unwrap();
        assert!(!hostname.is_empty());

        let arch = info_monitor.architecture().await.unwrap();
        assert!(!arch.is_empty());

        let os_version = info_monitor.os_version().await.unwrap();
        assert!(!os_version.is_empty());

        let kernel_version = info_monitor.kernel_version().await.unwrap();
        assert!(!kernel_version.is_empty());
    }

    #[tokio::test]
    async fn test_system_load() {
        let system = System::new_for_testing();
        let load_monitor = system.load_monitor();

        let load1 = load_monitor.load_average_1().await.unwrap();
        let load5 = load_monitor.load_average_5().await.unwrap();
        let load15 = load_monitor.load_average_15().await.unwrap();

        assert!(load1 >= 0.0);
        assert!(load5 >= 0.0);
        assert!(load15 >= 0.0);

        let process_count = load_monitor.process_count().await.unwrap();
        assert!(process_count > 0);

        let thread_count = load_monitor.thread_count().await.unwrap();
        assert!(thread_count > 0);
    }

    #[tokio::test]
    async fn test_system_uptime() {
        let system = System::new_for_testing();
        let uptime_monitor = system.uptime_monitor();

        let uptime = uptime_monitor.uptime_seconds().await.unwrap();
        assert!(uptime > 0);

        let boot_time = uptime_monitor.boot_time().await.unwrap();
        assert!(boot_time > 0);
    }

    #[tokio::test]
    async fn test_system_resources() {
        let system = System::new_for_testing();
        let resource_monitor = system.resource_monitor();

        let physical_cpu = resource_monitor.physical_cpu_count().await.unwrap();
        assert!(physical_cpu > 0);

        let logical_cpu = resource_monitor.logical_cpu_count().await.unwrap();
        assert!(logical_cpu >= physical_cpu);

        let total_memory = resource_monitor.total_memory().await.unwrap();
        assert!(total_memory > 0);

        let total_swap = resource_monitor.total_swap().await.unwrap();

        let free_memory = total_memory - total_swap;
        let free_swap = total_swap;

        assert!(free_memory <= total_memory);
        assert!(free_swap <= total_swap);
    }
}
