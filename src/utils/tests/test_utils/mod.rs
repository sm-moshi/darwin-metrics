pub mod iokit_mock;
mod mocks;

// Explicitly re-export the mocks we need
pub use iokit_mock::MockIOKit;
pub use mocks::{create_string_dictionary, create_test_dictionary};
