pub struct SystemMetrics {
    pub architecture: String,
    pub frequency: CoreFrequencies,
    pub power: PowerConsumption,
    pub temperature: CoreTemperatures,
}

pub async fn get_system_metrics() -> Result<SystemMetrics, Error> {
    let architecture = architecture::detect_architecture();
    let frequency = frequency::get_core_frequencies().await?;
    let power = power::get_power_consumption().await?;
    let temperature = temperature::get_core_temperatures().await?;

    Ok(SystemMetrics {
        architecture,
        frequency,
        power,
        temperature,
    })
}

pub mod architecture;
pub mod frequency;
pub mod power;
pub mod temperature;
