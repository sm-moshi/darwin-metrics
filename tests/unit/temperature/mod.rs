use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    /// Global mutex for temperature tests to prevent concurrent hardware access
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

mod monitors;
