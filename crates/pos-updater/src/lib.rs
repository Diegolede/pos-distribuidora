pub mod error;
pub mod update;

pub use error::Error;
pub use update::{UpdateChecker, UpdateInfo, verify_signature};
