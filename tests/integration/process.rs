use darwin_metrics::core::metrics::Process;
use darwin_metrics::hardware::memory::types::MemoryInfo;
use darwin_metrics::error::Result;
use std::{
    process::Command,
    time::{Duration, SystemTime},
};

#[test]
fn test_process_integration() -> Result<()> {
    // Get the current process
    let current_pid = std::process::id();
    let process = Process::get_process_by_pid_sync(current_pid)?;
    
    // Basic assertions about the current process
    assert_eq!(process.pid, current_pid);
    assert!(!process.name.is_empty(), "Process name should not be empty");
    assert!(process.thread_count > 0, "Thread count should be positive");
    assert!(!process.is_suspended, "Current process should not be suspended");
    
    Ok(())
}

#[test]
fn test_process_metrics() -> Result<()> {
    // Get the current process
    let current_pid = std::process::id();
    let process = Process::get_process_by_pid_sync(current_pid)?;
    
    // Validate basic metrics
    assert!(process.cpu_usage >= 0.0, "CPU usage should be non-negative");
    assert!(process.memory_usage > 0, "Memory usage should be positive");
    
    // IO stats might be zero or positive
    assert!(process.io_stats.read_bytes >= 0, "Read bytes should be non-negative");
    assert!(process.io_stats.write_bytes >= 0, "Write bytes should be non-negative");
    
    Ok(())
}

// This test is marked as ignored because it creates a real process
// and might be flaky in CI environments
#[test]
#[ignore] // Ignore in CI environments
fn test_process_creation_and_monitoring() -> Result<()> {
    // Create a real process
    let sleep_duration = 2;
    let child = Command::new("sleep")
        .arg(sleep_duration.to_string())
        .spawn()
        .expect("Failed to spawn sleep process");
    
    let pid = child.id();
    
    // Give the system a moment to register the process
    std::thread::sleep(Duration::from_millis(100));
    
    // Get the process
    let process = Process::get_process_by_pid_sync(pid)?;
    
    // Validate the process
    assert_eq!(process.pid, pid);
    assert!(process.name.contains("sleep"), "Process name should contain 'sleep'");
    
    // Get the start time
    let start_time = Process::get_process_start_time_sync(pid)?;
    assert!(start_time <= SystemTime::now(), "Start time should be in the past");
    
    // Wait for the process to complete
    std::thread::sleep(Duration::from_secs(sleep_duration + 1));
    
    // Process should no longer exist
    let result = Process::get_process_by_pid_sync(pid);
    assert!(result.is_err(), "Process should no longer exist");
    
    Ok(())
} 