use std::process::Command;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::collections::HashMap;

use super::*;

#[test]
fn test_get_current_process() {
    let pid = std::process::id();
    let process = Process::get_by_pid(pid).unwrap();
    assert_eq!(process.pid, pid);
    assert!(!process.name.is_empty());
}

#[test]
fn test_get_all_processes() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());
    
    // Verify basic process information
    for process in processes {
        assert!(process.pid > 0);
        assert!(!process.name.is_empty());
        assert!(process.memory_usage > 0);
        assert!(process.thread_count > 0);
    }
}

#[test]
fn test_parent_child_relationship() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());

    // Find a process that has a parent
    for process in processes {
        if let Ok(Some(parent_pid)) = Process::get_parent_pid(process.pid) {
            // Try to get the parent process
            if let Ok(parent) = Process::get_by_pid(parent_pid) {
                assert_eq!(parent.pid, parent_pid);
                return;
            }
        }
    }
}

#[test]
fn test_process_tree() {
    let processes = Process::get_all().unwrap();
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
}

#[test]
fn test_get_process_start_time() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());

    let now = SystemTime::now();
    for process in processes {
        if let Ok(start_time) = Process::get_process_start_time(process.pid) {
            assert!(start_time <= now, "Process start time is in the future");
            
            // Check that the process isn't unrealistically old
            if let Ok(age) = now.duration_since(start_time) {
                assert!(age <= Duration::from_secs(60 * 60 * 24 * 365 * 50), 
                    "Process is unrealistically old");
            }
        return;
        }
    }
}

#[test]
fn test_is_system_process() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());

    let mut found_system = false;
    let mut found_user = false;

    for process in processes {
        if process.is_system_process() {
            found_system = true;
        } else {
            found_user = true;
        }

        if found_system && found_user {
            break;
        }
    }

    assert!(found_system, "No system processes found");
    assert!(found_user, "No user processes found");
}

#[test]
fn test_get_child_processes() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());

    // Look for a process that has children
    for process in processes {
        let children = Process::get_child_processes(process.pid).unwrap();
        if !children.is_empty() {
            // Verify child process information
            for child in children {
                assert!(child.pid > 0);
                assert!(!child.name.is_empty());
                
                // Verify parent-child relationship
                if let Ok(Some(parent_pid)) = Process::get_parent_pid(child.pid) {
                    assert_eq!(parent_pid, process.pid);
                }
            }
            return;
        }
    }
}

#[test]
fn test_process_metrics_stream() {
    let pid = std::process::id();
    let process = Process::get_by_pid(pid).unwrap();
    
    // Basic metrics validation
    assert!(process.cpu_usage >= 0.0);
    assert!(process.memory_usage > 0);
    assert!(process.thread_count > 0);
    
    // I/O stats validation
    assert!(process.io_stats.read_bytes >= 0);
    assert!(process.io_stats.write_bytes >= 0);
    assert!(process.io_stats.read_count >= 0);
    assert!(process.io_stats.write_count >= 0);
}

#[test]
fn test_get_all_via_libproc() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());

    // Verify process information obtained via libproc
    for process in processes {
        assert!(process.pid > 0);
        assert!(!process.name.is_empty());
        assert!(process.memory_usage > 0);
        assert!(process.thread_count > 0);
    }
}

#[test]
fn test_special_parent_pid_cases() {
    // Test PID 0 (kernel_task)
    assert_eq!(Process::get_parent_pid(0).unwrap(), None);
    
    // Test PID 1 (launchd)
    assert_eq!(Process::get_parent_pid(1).unwrap(), None);
}

#[test]
fn test_process_metrics_stream_struct() {
    let pid = std::process::id();
    let process = Process::get_by_pid(pid).unwrap();

    assert_eq!(process.pid, pid);
    assert!(!process.name.is_empty());
    assert!(process.cpu_usage >= 0.0);
    assert!(process.memory_usage > 0);
    assert!(process.thread_count > 0);
}

// Synchronous tests moved from mod.rs
#[test]
fn test_process_new_sync() {
    let process = Process::new(1, "test");
    assert_eq!(process.pid, 1);
    assert_eq!(process.name, "test");
    assert_eq!(process.cpu_usage, 0.0);
    assert_eq!(process.memory_usage, 0);
    assert_eq!(process.thread_count, 0);
    assert!(!process.is_suspended);
}

#[test]
fn test_get_all_processes_sync() {
    let processes = Process::get_all().unwrap();
    assert!(!processes.is_empty());
}

#[test]
fn test_get_process_by_pid_sync() {
    let pid = std::process::id();
    let process = Process::get_by_pid(pid).unwrap();
    assert_eq!(process.pid, pid);
    assert!(!process.name.is_empty());
}

#[test]
fn test_get_parent_pid_sync() {
    let pid = std::process::id();
    let parent_pid = Process::get_parent_pid(pid).unwrap();
    assert!(parent_pid.is_some());
}

#[test]
fn test_get_process_start_time_sync() {
    let pid = std::process::id();
    let start_time = Process::get_process_start_time(pid).unwrap();
    assert!(start_time <= SystemTime::now());
}

#[test]
fn test_get_child_processes_sync() {
    let pid = 1; // init process
    let children = Process::get_child_processes(pid).unwrap();
    assert!(!children.is_empty());
}

#[test]
fn test_is_system_process_sync() {
    let process = Process::new(1, "launchd");
    assert!(process.is_system_process());

    let process = Process::new(1000, "user_process");
    assert!(!process.is_system_process());
}

#[test]
fn test_process_io_stats_sync() {
    let stats = ProcessIOStats::default();
    assert_eq!(stats.read_bytes, 0);
    assert_eq!(stats.write_bytes, 0);
    assert_eq!(stats.read_count, 0);
    assert_eq!(stats.write_count, 0);
}
