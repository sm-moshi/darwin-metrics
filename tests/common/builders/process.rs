use darwin_metrics::core::metrics::Process;
use darwin_metrics::memory::types::MemoryInfo;
use darwin_metrics::core::types::ProcessIOStats;
use std::time::{Duration, SystemTime};

/// Builder for creating test process instances
pub struct TestProcessBuilder {
    pid: Option<u32>,
    name: Option<String>,
    command: Option<String>,
    memory: Option<MemoryInfo>,
    cpu_usage: Option<f64>,
    start_time: Option<SystemTime>,
    parent_pid: Option<u32>,
    is_system: Option<bool>,
    io_stats: Option<ProcessIOStats>,
}

impl TestProcessBuilder {
    /// Create a new TestProcessBuilder with default values
    pub fn new() -> Self {
        Self {
            pid: Some(1),
            name: Some("test_process".to_string()),
            command: Some("/usr/bin/test_process".to_string()),
            memory: Some(MemoryInfo::default()),
            cpu_usage: Some(5.0),
            start_time: Some(SystemTime::now() - Duration::from_secs(3600)),
            parent_pid: Some(0),
            is_system: Some(false),
            io_stats: Some(ProcessIOStats::default()),
        }
    }
    
    /// Set the process ID
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }
    
    /// Set the process name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set the command
    pub fn with_command(mut self, command: &str) -> Self {
        self.command = Some(command.to_string());
        self
    }
    
    /// Set the memory
    pub fn with_memory(mut self, memory: MemoryInfo) -> Self {
        self.memory = Some(memory);
        self
    }
    
    /// Set the CPU usage
    pub fn with_cpu_usage(mut self, cpu_usage: f64) -> Self {
        self.cpu_usage = Some(cpu_usage);
        self
    }
    
    /// Set the start time
    pub fn with_start_time(mut self, start_time: SystemTime) -> Self {
        self.start_time = Some(start_time);
        self
    }
    
    /// Set the parent process ID
    pub fn with_parent_pid(mut self, parent_pid: u32) -> Self {
        self.parent_pid = Some(parent_pid);
        self
    }
    
    /// Set whether the process is a system process
    pub fn with_system_process(mut self, is_system: bool) -> Self {
        self.is_system = Some(is_system);
        self
    }
    
    /// Set the I/O statistics
    pub fn with_io_stats(mut self, io_stats: ProcessIOStats) -> Self {
        self.io_stats = Some(io_stats);
        self
    }
    
    /// Build a mock Process instance
    pub fn build(self) -> Process {
        Process {
            pid: self.pid.unwrap_or(1),
            name: self.name.unwrap_or_else(|| "test_process".to_string()),
            command: self.command.unwrap_or_else(|| "/usr/bin/test_process".to_string()),
            memory: self.memory.unwrap_or_else(MemoryInfo::default),
            cpu_usage: self.cpu_usage.unwrap_or(5.0),
            start_time: self.start_time.unwrap_or_else(|| SystemTime::now() - Duration::from_secs(3600)),
            parent_pid: self.parent_pid,
            is_system: self.is_system.unwrap_or(false),
            io_stats: self.io_stats.unwrap_or_else(ProcessIOStats::default),
        }
    }
}

/// A mock implementation of Process for testing
pub struct MockProcess {
    process: Process,
    start_time: SystemTime,
    parent_pid: Option<u32>,
}

impl MockProcess {
    /// Get the process
    pub fn process(&self) -> &Process {
        &self.process
    }
    
    /// Get the process start time
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }
    
    /// Get the parent process ID
    pub fn parent_pid(&self) -> Option<u32> {
        self.parent_pid
    }
} 