//! Temperature metrics collection

use super::Collector;
use anyhow::Result;
use core_foundation::{
    array::*,
    base::*,
    dictionary::*,
    number::{kCFNumberIntType, CFNumberCreate, CFNumberRef},
    string::*,
};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::time::Instant;
use sysinfo::{CpuRefreshKind, System};

// IOKit constants
const K_HIDPAGE_APPLE_VENDOR: i32 = 0xff00;
const K_HIDUSAGE_APPLE_VENDOR_TEMPERATURE_SENSOR: i32 = 0x0005;
const K_IOHIDEVENT_TYPE_TEMPERATURE: i64 = 15;

// Define IOHIDServiceClientRef type
type IOHIDServiceClientRef = *mut c_void;

// Core Foundation helper functions
fn cfstr(string: &str) -> CFStringRef {
    unsafe {
        CFStringCreateWithCString(
            kCFAllocatorDefault,
            string.as_ptr() as *const i8,
            kCFStringEncodingUTF8,
        )
    }
}

fn cfnum(value: i32) -> CFNumberRef {
    unsafe {
        CFNumberCreate(
            kCFAllocatorDefault,
            kCFNumberIntType,
            &value as *const _ as *const c_void,
        )
    }
}

fn from_cfstr(cf_string: CFStringRef) -> String {
    let length = unsafe { CFStringGetLength(cf_string) };
    let mut buffer = vec![0u8; length as usize * 4 + 1]; // UTF-8 can be up to 4 bytes per character

    unsafe {
        CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as CFIndex,
            kCFStringEncodingUTF8,
        );
        CFRelease(cf_string as *const c_void);

        // Use unsafe for from_ptr
        let cstr = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        cstr.to_string_lossy().into_owned()
    }
}

// External functions from IOKit
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDEventSystemClientCreate(allocator: CFAllocatorRef) -> *mut c_void;
    fn IOHIDEventSystemClientSetMatching(client: *mut c_void, matching: CFDictionaryRef);
    fn IOHIDEventSystemClientCopyServices(client: *mut c_void) -> CFArrayRef;
    fn IOHIDServiceClientCopyProperty(service: *mut c_void, key: CFStringRef) -> CFTypeRef;
    fn IOHIDServiceClientCopyEvent(
        service: *mut c_void,
        type_: i64,
        usage: i32,
        options: i64,
    ) -> *mut c_void;
    fn IOHIDEventGetFloatValue(event: *mut c_void, field: i64) -> f64;
}

const MAX_HISTORY: usize = 100;

/// Temperature reading from a sensor
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub name: String,
    pub temperature: f32,
    pub location: SensorLocation,
}

/// Known sensor locations
#[derive(Debug, Clone, PartialEq)]
pub enum SensorLocation {
    Cpu,
    Gpu,
    Memory,
    Storage,
    Battery,
    Other(String),
}

/// Temperature statistics
#[derive(Debug, Clone)]
pub struct TemperatureStats {
    pub temperatures: Vec<f32>,
    pub frequencies: Vec<u64>,
    pub last_update: Instant,
}

impl Default for TemperatureStats {
    fn default() -> Self {
        Self {
            temperatures: Vec::new(),
            frequencies: Vec::new(),
            last_update: Instant::now(),
        }
    }
}

/// Temperature history tracking
#[derive(Debug, Default, Clone)]
pub struct TemperatureHistory {
    pub temperature_history: VecDeque<Vec<f32>>,
    pub frequency_history: VecDeque<Vec<u64>>,
}

impl TemperatureHistory {
    fn new() -> Self {
        let mut temperature_history = VecDeque::with_capacity(MAX_HISTORY);
        let mut frequency_history = VecDeque::with_capacity(MAX_HISTORY);

        // Initialize with empty vectors
        for _ in 0..MAX_HISTORY {
            temperature_history.push_back(Vec::new());
            frequency_history.push_back(Vec::new());
        }

        Self {
            temperature_history,
            frequency_history,
        }
    }

    pub fn update(&mut self, stats: &TemperatureStats) {
        self.temperature_history.pop_front();
        self.temperature_history
            .push_back(stats.temperatures.clone());

        self.frequency_history.pop_front();
        self.frequency_history.push_back(stats.frequencies.clone());
    }
}

/// Temperature metrics collector
pub struct TemperatureCollector {
    sys: System,
    history: TemperatureHistory,
}

impl TemperatureCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: System::new(),
            history: TemperatureHistory::new(),
        })
    }

    pub fn get_history(&self) -> &TemperatureHistory {
        &self.history
    }
}

impl Default for TemperatureCollector {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Collector for TemperatureCollector {
    type Stats = TemperatureStats;

    fn collect(&mut self) -> Result<Self::Stats> {
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        let mut frequencies = Vec::new();
        for cpu in self.sys.cpus() {
            frequencies.push(cpu.frequency());
        }

        // Note: Currently we don't have direct temperature readings
        // This could be expanded in the future to use platform-specific APIs
        let temperatures = Vec::new();

        let stats = TemperatureStats {
            temperatures,
            frequencies,
            last_update: Instant::now(),
        };

        self.history.update(&stats);

        Ok(stats)
    }
}

impl TemperatureCollector {
    fn read_sensor(&self, service: IOHIDServiceClientRef) -> Option<(String, f32)> {
        unsafe {
            // Get sensor name
            let name = IOHIDServiceClientCopyProperty(service, cfstr("Product"));
            if name.is_null() {
                return None;
            }

            // Cast to CFStringRef before passing to from_cfstr
            let name = from_cfstr(name as CFStringRef);

            // Get temperature reading
            let event = IOHIDServiceClientCopyEvent(service, K_IOHIDEVENT_TYPE_TEMPERATURE, 0, 0);
            if event.is_null() {
                return None;
            }

            let temp = IOHIDEventGetFloatValue(event, K_IOHIDEVENT_TYPE_TEMPERATURE << 16);
            CFRelease(event as _);

            Some((name, temp as f32))
        }
    }
}

impl Drop for TemperatureCollector {
    fn drop(&mut self) {
        // No cleanup needed
    }
}
