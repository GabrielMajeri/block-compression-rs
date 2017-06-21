/// Support for reading and writing DDS (Direct Draw Surface) files,
/// most commonly used by DirectX applications.
pub mod dds;

use std::result;
use std::io;

/// Error type used by the format module.
#[derive(Debug)]
pub enum Error {
	/// Some unexpected data was encountered while reading a file.
	FormatError(String),
	/// An I/O error was encountered while reading / writing an image.
	IoError(io::Error)
}

// To allow the using "?" syntax for IO functions.
impl From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::IoError(err)
	}
}

type Result<T> = result::Result<T, Error>;
