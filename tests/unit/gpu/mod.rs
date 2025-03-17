use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    /// Global mutex for GPU tests to prevent concurrent access
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

mod monitors;
mod integration {}
