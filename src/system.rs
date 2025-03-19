use std::sync::Arc;

use crate::error::Result;
use crate::hardware::iokit::{IOKit, IOKitImpl};

#[derive(Clone)]
pub struct System {
    io_kit: Arc<Box<dyn IOKit>>,
}

impl System {
    pub fn new() -> Result<Self> {
        let io_kit_impl = IOKitImpl::new()?;
        Ok(Self {
            io_kit: Arc::new(Box::new(io_kit_impl) as Box<dyn IOKit>),
        })
    }

    pub fn io_kit(&self) -> Arc<Box<dyn IOKit>> {
        self.io_kit.clone()
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new().expect("Failed to create default System instance")
    }
}
