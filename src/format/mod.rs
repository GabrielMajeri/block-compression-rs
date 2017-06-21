use std::result;

// Error type.
mod error;

pub use self::error::Error;
type Result<T> = result::Result<T, Error>;

/// Support for reading and writing DDS (Direct Draw Surface) files,
/// most commonly used by DirectX applications.
pub mod dds;
