mod smc_tests;
mod gpu_tests;
mod thermal_tests;
mod fan_tests;
mod service_tests;
mod types_tests;

pub(crate) use self::{
    fan_tests::*,
    gpu_tests::*,
    service_tests::*,
    smc_tests::*,
    thermal_tests::*,
    types_tests::*,
}; 