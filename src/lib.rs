//! # darwin-metrics
//!
//! `darwin-metrics` is a Rust library that provides native access to macOS system metrics
//! through low-level system APIs. This crate offers efficient, safe, and async-capable
//! interfaces for monitoring system resources on macOS.
//!

pub mod battery;
pub mod error;
pub mod hardware;
pub mod power;
pub mod process;
pub mod system;
pub mod utils;

pub use battery::*;
pub use hardware::*;
pub use power::*;
// Error is already re-exported below, so we don't need this line
pub use process::*;
pub use system::*;

// Re-export important types at the crate root
pub use error::{Error, Result};
