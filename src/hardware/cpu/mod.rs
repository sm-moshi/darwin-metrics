mod cpu;
mod frequency;

pub use cpu::CPU;
pub use frequency::FrequencyMetrics;

pub const MAX_CORES: u32 = 64;
pub const MAX_FREQUENCY_MHZ: f64 = 5000.0;

pub trait CpuMetrics {
    fn get_cpu_usage(&self) -> f64;
    fn get_cpu_temperature(&self) -> Option<f64>;
    fn get_cpu_frequency(&self) -> f64;
}
