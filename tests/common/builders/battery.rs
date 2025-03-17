use crate::{
    battery::Battery,
    error::Result,
    tests::common::mocks::battery::MockIOKit,
};

pub struct TestBatteryBuilder {
    is_present: bool,
    is_charging: bool,
    cycle_count: i64,
    percentage: f64,
    temperature: f64,
    time_remaining: i64,
    design_capacity: f64,
    current_capacity: f64,
}

impl TestBatteryBuilder {
    pub fn new() -> Self {
        Self {
            is_present: true,
            is_charging: false,
            cycle_count: 100,
            percentage: 75.0,
            temperature: 35.0,
            time_remaining: 7200,
            design_capacity: 10000.0,
            current_capacity: 8000.0,
        }
    }

    pub fn present(mut self, is_present: bool) -> Self {
        self.is_present = is_present;
        self
    }

    pub fn charging(mut self, is_charging: bool) -> Self {
        self.is_charging = is_charging;
        self
    }

    pub fn cycle_count(mut self, cycle_count: i64) -> Self {
        self.cycle_count = cycle_count;
        self
    }

    pub fn percentage(mut self, percentage: f64) -> Self {
        self.percentage = percentage;
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn capacity(mut self, current: f64, design: f64) -> Self {
        self.current_capacity = current;
        self.design_capacity = design;
        self
    }

    pub fn time_remaining(mut self, time_remaining: i64) -> Self {
        self.time_remaining = time_remaining;
        self
    }

    pub fn build(self) -> Result<Battery> {
        let mock_iokit = MockIOKit::new()?
            .with_battery_info(
                self.is_present,
                self.is_charging,
                self.cycle_count,
                self.percentage,
                self.temperature,
                self.time_remaining,
                self.design_capacity,
                self.current_capacity,
            )?;
        Battery::new(Box::new(mock_iokit))
    }
}

impl Default for TestBatteryBuilder {
    fn default() -> Self {
        Self::new()
    }
} 