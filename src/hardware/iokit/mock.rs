//! This module is only used to re-export the MockIOKit from the test_utils module.
//!
//! The actual implementation has been moved to src/utils/tests/test_utils/iokit_mock.rs
//! for better organization of test utilities.

pub use crate::utils::tests::test_utils::iokit_mock::MockIOKit;
