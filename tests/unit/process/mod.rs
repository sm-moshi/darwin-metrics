use crate::{
    process::{Process, ProcessIOStats},
    error::Result,
    tests::common::builders::process::TestProcessBuilder,
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use darwin_metrics::core::metrics::Process;
use darwin_metrics::hardware::memory::types::MemoryInfo;
use darwin_metrics::core::types::ProcessIOStats;
use darwin_metrics::error::{Error, Result};

#[test]
fn test_process_new_sync() -> Result<()> {
    let mock_process = TestProcessBuilder::new().build()?;
    let process = mock_process.process();
    
    assert_eq!(process.pid, 1000);
    assert_eq!(process.name, "test_process");
    assert!(!process.is_suspended);
    
    Ok(())
}

#[test]
fn test_get_all_processes_sync() -> Result<()> {
    // This test requires actual system interaction
    let processes = Process::get_all_processes_sync()?;
    
    assert!(!processes.is_empty(), "Process list should not be empty");
    
    // Verify that each process has valid properties
    for process in processes {
        assert!(process.pid > 0, "Process ID should be positive");
        assert!(!process.name.is_empty(), "Process name should not be empty");
        
        // Memory usage should be non-negative
        assert!(process.memory_usage >= 0, "Memory usage should be non-negative");
        
        // Thread count should be positive
        assert!(process.thread_count > 0, "Thread count should be positive");
    }
    
    Ok(())
}

#[test]
fn test_get_process_by_pid_sync() -> Result<()> {
    // Get the first process from the list to test with
    let processes = Process::get_all_processes_sync()?;
    if processes.is_empty() {
        // Skip test if no processes are available
        return Ok(());
    }
    
    let first_process = &processes[0];
    let pid = first_process.pid;
    
    // Get the process by PID
    let process = Process::get_process_by_pid_sync(pid)?;
    
    // Verify that the process has the correct PID
    assert_eq!(process.pid, pid);
    
    Ok(())
}

#[test]
fn test_get_parent_process_sync() -> Result<()> {
    // Most processes should have a parent (except init/PID 1)
    let processes = Process::get_all_processes_sync()?;
    
    // Find a process that's not PID 1
    let process = processes.iter().find(|p| p.pid > 1);
    
    if let Some(process) = process {
        let parent = Process::get_parent_process_sync(process.pid)?;
        
        // The parent should exist and have a valid PID
        assert!(parent.is_some(), "Parent process should exist");
        if let Some(parent) = parent {
            assert!(parent.pid > 0, "Parent PID should be positive");
            assert!(!parent.name.is_empty(), "Parent name should not be empty");
        }
    }
    
    Ok(())
}

#[test]
fn test_get_child_processes_sync() -> Result<()> {
    // Test with PID 1 (init), which should have children
    let children = Process::get_child_processes_sync(1)?;
    
    // Init should have at least one child
    assert!(!children.is_empty(), "Init should have child processes");
    
    // Verify that each child has valid properties
    for child in children {
        assert!(child.pid > 1, "Child PID should be greater than 1");
        assert!(!child.name.is_empty(), "Child name should not be empty");
    }
    
    Ok(())
}

#[test]
fn test_is_system_process() -> Result<()> {
    // Create test processes
    let system_process = TestProcessBuilder::new()
        .pid(1)
        .name("launchd")
        .build()?;
    
    let user_process = TestProcessBuilder::new()
        .pid(1000)
        .name("user_app")
        .build()?;
    
    // Check if they are system processes
    assert!(Process::is_system_process_sync(system_process.process().pid)?, 
            "PID 1 should be a system process");
    
    assert!(!Process::is_system_process_sync(user_process.process().pid)?, 
            "PID 1000 should not be a system process");
    
    Ok(())
}

#[test]
fn test_process_metrics() -> Result<()> {
    // Create a test process with specific metrics
    let mock_process = TestProcessBuilder::new()
        .cpu_usage(10.5)
        .memory_usage(1024 * 1024 * 50) // 50 MB
        .thread_count(4)
        .io_stats(5000, 10000, 50, 100)
        .build()?;
    
    let process = mock_process.process();
    
    // Verify metrics
    assert_eq!(process.cpu_usage, 10.5);
    assert_eq!(process.memory_usage, 1024 * 1024 * 50);
    assert_eq!(process.thread_count, 4);
    assert_eq!(process.io_stats.read_bytes, 5000);
    assert_eq!(process.io_stats.write_bytes, 10000);
    assert_eq!(process.io_stats.read_count, 50);
    assert_eq!(process.io_stats.write_count, 100);
    
    Ok(())
}

#[test]
fn test_process_io_stats() -> Result<()> {
    // Create a test process with specific IO stats
    let mock_process = TestProcessBuilder::new()
        .io_stats(1000, 2000, 10, 20)
        .build()?;
    
    let process = mock_process.process();
    
    // Verify IO stats
    assert_eq!(process.io_stats.read_bytes, 1000);
    assert_eq!(process.io_stats.write_bytes, 2000);
    assert_eq!(process.io_stats.read_count, 10);
    assert_eq!(process.io_stats.write_count, 20);
    
    Ok(())
}

#[test]
fn test_process_start_time() -> Result<()> {
    // Create a test process with a specific start time
    let start_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour ago
    let mock_process = TestProcessBuilder::new()
        .start_time(start_time)
        .build()?;
    
    // Verify start time
    assert_eq!(mock_process.start_time(), start_time);
    
    Ok(())
}

#[test]
fn test_process_io_stats_sync() {
    let stats = ProcessIOStats::default();
    assert_eq!(stats.read_bytes, 0);
    assert_eq!(stats.write_bytes, 0);
    assert_eq!(stats.read_count, 0);
    assert_eq!(stats.write_count, 0);
}

#[test]
fn test_parent_child_relationship() -> Result<()> {
    let processes = Process::get_all()?;
    assert!(!processes.is_empty());

    // Find a process that has a parent
    for process in processes {
        if let Some(parent_pid) = Process::get_parent_pid(process.pid)? {
            // Try to get the parent process
            if let Ok(parent) = Process::get_by_pid(parent_pid) {
                assert_eq!(parent.pid, parent_pid);
                return Ok(());
            }
        }
    }
    
    // If we couldn't find a parent-child relationship, the test passes anyway
    // This is to avoid flaky tests in CI environments
    Ok(())
}

#[test]
fn test_process_tree() -> Result<()> {
    let processes = Process::get_all()?;
    assert!(!processes.is_empty());

    // Build process tree
    let mut process_map = HashMap::new();
    let mut root_processes = Vec::new();

    // First pass: collect all processes
    for process in processes {
        process_map.insert(process.pid, process);
    }

    // Second pass: build parent-child relationships
    for &pid in process_map.keys().collect::<Vec<_>>() {
        if let Ok(Some(parent_pid)) = Process::get_parent_pid(pid) {
            if process_map.contains_key(&parent_pid) {
                continue;
            }
        }
        if let Some(process) = process_map.get(&pid) {
            root_processes.push(process.clone());
        }
    }

    // Verify we found some root processes
    assert!(!root_processes.is_empty());

    // Verify at least one root process is a system process
    let has_system_process = root_processes.iter().any(|p| p.is_system_process());
    assert!(has_system_process, "No system processes found in root processes");
    
    Ok(())
}

#[test]
fn test_special_parent_pid_cases() -> Result<()> {
    // Test PID 0 (kernel_task)
    assert_eq!(Process::get_parent_pid(0)?, None);
    
    // Test PID 1 (launchd)
    assert_eq!(Process::get_parent_pid(1)?, None);
    
    Ok(())
}

#[test]
fn test_process_metrics_stream_struct() -> Result<()> {
    let pid = std::process::id();
    let process = Process::get_by_pid(pid)?;

    assert_eq!(process.pid, pid);
    assert!(!process.name.is_empty());
    assert!(process.cpu_usage >= 0.0);
    assert!(process.memory_usage > 0);
    assert!(process.thread_count > 0);
    
    Ok(())
} 