use crate::{
    error::Result,
    utils::bindings::{kinfo_proc, mach_host_self, proc_info, proc_pidinfo},
};
use std::{
    ffi::{c_void, CString},
    mem, ptr,
};

use super::gpu_impl::Gpu;

impl Gpu {
    /// Estimates the GPU utilization percentage
    ///
    /// This method uses various heuristics to estimate GPU utilization since
    /// there's no direct API to get this information on macOS.
    pub fn estimate_utilization(&self) -> Result<f32> {
        // Get GPU characteristics
        let characteristics = self.get_characteristics();

        // For Apple Silicon, use a different estimation approach
        if characteristics.is_apple_silicon {
            self.estimate_apple_silicon_utilization()
        } else if characteristics.is_integrated {
            // For integrated GPUs, estimate based on CPU usage
            self.estimate_integrated_gpu_utilization()
        } else {
            // For discrete GPUs, use a combination of process activity and memory usage
            self.estimate_discrete_gpu_utilization()
        }
    }

    // Estimate utilization for Apple Silicon GPUs
    fn estimate_apple_silicon_utilization(&self) -> Result<f32> {
        // For Apple Silicon, we can use a combination of:
        // 1. Process activity (Metal, GPU-intensive apps)
        // 2. Memory pressure
        // 3. System load

        // Get process activity score (0-100)
        let process_score = self.get_gpu_process_activity()?;

        // Get memory pressure score (0-100)
        let memory_score = self.get_memory_pressure_score()?;

        // Get system load score (0-100)
        let load_score = self.get_system_load_score()?;

        // Weighted average of scores
        // Process activity is the strongest indicator
        let utilization = (process_score * 0.6 + memory_score * 0.2 + load_score * 0.2).min(100.0);

        Ok(utilization as f32)
    }

    // Estimate utilization for integrated GPUs
    fn estimate_integrated_gpu_utilization(&self) -> Result<f32> {
        // For integrated GPUs, CPU usage is a good proxy for GPU usage
        // since they share resources

        // Get CPU usage
        let cpu_usage = self.get_cpu_usage()?;

        // Get process activity score
        let process_score = self.get_gpu_process_activity()?;

        // Weighted average, favoring process activity if GPU processes are running
        let utilization = if process_score > 20.0 {
            (process_score * 0.7 + cpu_usage * 0.3).min(100.0)
        } else {
            (cpu_usage * 0.7).min(100.0)
        };

        Ok(utilization as f32)
    }

    // Estimate utilization for discrete GPUs
    fn estimate_discrete_gpu_utilization(&self) -> Result<f32> {
        // For discrete GPUs, we use:
        // 1. Process activity (Metal, GPU-intensive apps)
        // 2. Memory usage
        // 3. Temperature (if available)

        // Get process activity score
        let process_score = self.get_gpu_process_activity()?;

        // Get memory usage percentage
        let memory_info = self.estimate_memory_info()?;
        let memory_percentage =
            if memory_info.total > 0 { (memory_info.used as f64 / memory_info.total as f64) * 100.0 } else { 0.0 };

        // Get temperature score if available
        let temp_score = if let Ok(temp) = self.get_temperature() {
            // Convert temperature to a 0-100 score
            // Assume idle temp is around 40°C and max is around 90°C
            ((temp as f64 - 40.0) / 50.0 * 100.0).max(0.0).min(100.0)
        } else {
            0.0
        };

        // Weighted average of scores
        let utilization = (process_score * 0.5 + memory_percentage * 0.3 + temp_score * 0.2).min(100.0);

        Ok(utilization as f32)
    }

    // Get a score based on GPU-intensive processes running
    fn get_gpu_process_activity(&self) -> Result<f64> {
        // List of process names that typically use the GPU intensively
        let gpu_process_names = [
            "Metal",
            "OpenGL",
            "Unity",
            "Unreal",
            "Blender",
            "Final Cut",
            "Premiere",
            "After Effects",
            "Photoshop",
            "Lightroom",
            "DaVinci",
            "Maya",
            "Cinema 4D",
            "Steam",
            "VLC",
            "QuickTime",
            "Chrome",
            "Safari",
            "Firefox",
        ];

        let mut gpu_process_count = 0;
        let mut total_cpu_usage = 0.0;

        unsafe {
            // Get process list using sysctl
            let mut mib = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_ALL, 0];
            let mut buffer_size: libc::size_t = 0;

            // First call to get buffer size
            let mut result = libc::sysctl(mib.as_mut_ptr(), 3, ptr::null_mut(), &mut buffer_size, ptr::null_mut(), 0);

            if result != 0 || buffer_size == 0 {
                return Ok(50.0); // Default to 50% if sysctl fails
            }

            // Allocate buffer
            let mut buffer = vec![0u8; buffer_size];

            // Second call to get process list
            result = libc::sysctl(
                mib.as_mut_ptr(),
                3,
                buffer.as_mut_ptr() as *mut c_void,
                &mut buffer_size,
                ptr::null_mut(),
                0,
            );

            // Get process info structure for CPU usage
            let mut proc_info: proc_info = mem::zeroed();
            let proc_info_size = mem::size_of::<proc_info>();

            if result != 0 {
                return Ok(50.0); // Default to 50% if sysctl fails
            }

            // Calculate number of processes
            let proc_size = std::mem::size_of::<kinfo_proc>();
            let nprocs = buffer_size / proc_size;

            // Iterate through processes
            for i in 0..nprocs {
                let proc_ptr = buffer.as_ptr().add(i * proc_size) as *const kinfo_proc;
                let proc = &*proc_ptr;

                // Get process name
                let mut name_buffer = [0i8; libc::MAXCOMLEN + 1];
                for j in 0..libc::MAXCOMLEN {
                    name_buffer[j] = proc.kp_eproc.p_comm[j] as i8; // Convert u8 to i8
                }

                let c_name = CString::from_raw(name_buffer.as_mut_ptr());
                let name = c_name.to_string_lossy().into_owned();
                std::mem::forget(c_name); // Prevent double free

                // Check if process is in GPU process list
                for gpu_name in &gpu_process_names {
                    if name.contains(gpu_name) {
                        gpu_process_count += 1;

                        // Get process CPU usage
                        let pid = proc.kp_proc.p_pid;
                        let result = proc_pidinfo(
                            pid,
                            libc::PROC_PIDTASKINFO,
                            0,
                            &mut proc_info as *mut proc_info as *mut c_void,
                            proc_info_size as i32, // Convert to i32
                        );

                        if i64::from(result) == proc_info_size as i64 {
                            // Add CPU usage percentage - Note: we need to modify this part
                            // since ptinfo doesn't exist in proc_info
                            // This is a placeholder - we'll need to use the correct fields
                            total_cpu_usage += 1.0; // Placeholder value
                        }

                        break;
                    }
                }
            }
        }

        // Calculate GPU process activity score
        // More GPU processes and higher CPU usage = higher score
        let process_count_score = (gpu_process_count as f64 * 10.0).min(50.0);
        let cpu_usage_score = (total_cpu_usage * 2.0_f64).min(50.0);

        let activity_score = process_count_score + cpu_usage_score;

        Ok(activity_score)
    }

    // Get a score based on memory pressure
    fn get_memory_pressure_score(&self) -> Result<f64> {
        // Get memory info
        let memory_info = self.estimate_memory_info()?;

        // Calculate memory pressure score
        let memory_usage_percent = if memory_info.total > 0 {
            (memory_info.used as f64 / memory_info.total as f64) * 100.0
        } else {
            50.0 // Default if we can't calculate
        };

        Ok(memory_usage_percent)
    }

    // Get a score based on system load
    fn get_system_load_score(&self) -> Result<f64> {
        unsafe {
            let mut load_avg: [f64; 3] = [0.0; 3];

            if libc::getloadavg(load_avg.as_mut_ptr(), 3) < 0 {
                return Ok(50.0); // Default if getloadavg fails
            }

            // Convert 1-minute load average to a 0-100 score
            // Normalize by number of CPUs
            let num_cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
            let normalized_load = if num_cpus > 0 {
                (load_avg[0] / num_cpus as f64) * 100.0
            } else {
                load_avg[0] * 25.0 // Assume 4 CPUs if we can't get the count
            };

            Ok(normalized_load.min(100.0))
        }
    }

    // Get CPU usage percentage
    fn get_cpu_usage(&self) -> Result<f64> {
        unsafe {
            // Get host CPU load info
            let mut cpu_load: libc::host_cpu_load_info_data_t = mem::zeroed();
            let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;

            let kr = libc::host_statistics(
                mach_host_self(),
                libc::HOST_CPU_LOAD_INFO,
                &mut cpu_load as *mut libc::host_cpu_load_info_data_t as *mut libc::integer_t,
                &mut count,
            );

            if kr != libc::KERN_SUCCESS {
                return Ok(50.0); // Default if host_statistics fails
            }

            // Calculate CPU usage from load info
            let user = cpu_load.cpu_ticks[libc::CPU_STATE_USER as usize] as f64;
            let system = cpu_load.cpu_ticks[libc::CPU_STATE_SYSTEM as usize] as f64;
            let idle = cpu_load.cpu_ticks[libc::CPU_STATE_IDLE as usize] as f64;
            let nice = cpu_load.cpu_ticks[libc::CPU_STATE_NICE as usize] as f64;

            let total = user + system + idle + nice;
            let usage = if total > 0.0 { ((user + system) / total) * 100.0 } else { 0.0 };

            Ok(usage)
        }
    }
}
