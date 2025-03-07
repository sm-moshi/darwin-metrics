use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Network {
    pub name: String,
    pub is_active: bool,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
}

impl Network {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_active: false,
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
        }
    }

    pub fn get_stats() -> Result<Vec<Self>, Error> {
        Err(Error::not_implemented("Network statistics collection"))
    }
}
