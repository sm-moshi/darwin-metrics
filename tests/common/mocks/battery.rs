use crate::{
    battery::BatteryInfo,
    error::Result,
    hardware::iokit::{FanInfo, GpuStats, IOKit, ThermalInfo, ThreadSafeAnyObject},
    utils::safe_dictionary::SafeDictionary,
};
use objc2::runtime::{AnyObject, Class};
use objc2::{class, msg_send};
use objc2_foundation::{NSMutableDictionary, NSNumber, NSObject, NSString};
use std::time::Duration;
use darwin_metrics::{
    error::{Error, Result},
    hardware::iokit::{GpuStats, IOKit, ThermalInfo, ThreadSafeAnyObject},
    temperature::types::Fan,
    utils::core::dictionary::SafeDictionary,
};

#[derive(Debug)]
pub struct MockIOKit {
    battery_info: Option<BatteryInfo>,
}

impl MockIOKit {
    pub fn new() -> Result<Self> {
        unsafe {
            let _: &Class = class!(NSObject);
            let _: &Class = class!(NSMutableDictionary);
            let _: &Class = class!(NSNumber);
            let _: &Class = class!(NSString);
        }
        Ok(Self { battery_info: None })
    }

    pub fn with_battery_info(
        mut self,
        is_present: bool,
        is_charging: bool,
        cycle_count: i64,
        percentage: f64,
        temperature: f64,
        time_remaining: i64,
        design_capacity: f64,
        current_capacity: f64,
    ) -> Result<Self> {
        self.battery_info = Some(BatteryInfo {
            present: is_present,
            percentage: percentage as i64,
            cycle_count,
            is_charging,
            is_external: !is_charging,
            temperature,
            power_draw: 0.0,
            design_capacity: design_capacity as i64,
            current_capacity: current_capacity as i64,
            time_remaining: Some(Duration::from_secs(time_remaining as u64)),
        });
        Ok(self)
    }
}

impl Default for MockIOKit {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self { battery_info: self.battery_info.clone() }
    }
}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, _name: &str) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(None)
    }

    fn io_service_get_matching_service(&self, _matching: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        let obj = NSObject::new();
        Ok(ThreadSafeAnyObject::new(obj))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64> {
        Ok(42.0)
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![])
    }

    fn check_thermal_throttling(&self, _plane: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(15.0)
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(self.battery_info.as_ref().map(|info| info.temperature))
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let mut dict = SafeDictionary::new();
        if let Some(info) = &self.battery_info {
            dict.set_bool("BatteryInstalled", info.present);
            dict.set_bool("IsCharging", info.is_charging);
            dict.set_bool("ExternalConnected", info.is_external);
            dict.set_f64("CycleCount", info.cycle_count as f64);
            dict.set_f64("CurrentCapacity", info.percentage as f64);
            dict.set_f64("MaxCapacity", info.current_capacity as f64);
            dict.set_f64("DesignCapacity", info.design_capacity as f64);
            dict.set_f64("Temperature", info.temperature);
        }
        Ok(dict)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_number_property(&self, _dict: &SafeDictionary, _key: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u64],
        _output: &mut [u64],
    ) -> Result<()> {
        Ok(())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.get_battery_info()
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(4)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(8)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        Ok(vec![0.5, 0.6, 0.7, 0.8])
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let mut dict = SafeDictionary::new();
        dict.set_f64("CPU_0_DIE_TEMP", 42.0);
        dict.set_f64("GPU_0_DIE_TEMP", 55.0);
        dict.set_f64("FAN_0_SPEED", 2000.0);
        dict.set_f64("HS_0_TEMP", 40.0);
        dict.set_f64("AMBIENT_TEMP", 25.0);
        dict.set_f64("BATTERY_TEMP", self.get_battery_temperature()?.unwrap_or(35.0));
        dict.set_bool("THERMAL_THROTTLING", false);
        Ok(ThermalInfo::new(dict))
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats::default())
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo { 
            current_speed: 2000,
            target_speed: 2000,
            min_speed: Some(1000),
            max_speed: Some(4000),
            index: _fan_index,
            speed_rpm: 2000,
            percentage: 50.0 
        })
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        Ok(vec![Fan {
            name: "System Fan".to_string(),
            speed_rpm: 2000,
            min_speed: 1000,
            max_speed: 4000,
            target_speed: 2000,
        }])
    }
} 