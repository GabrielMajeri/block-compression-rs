use std::io;
use bincode;

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

impl From<bincode::Error> for Error {
	fn from(err: bincode::Error) -> Error {
		use bincode::ErrorKind;
		match *err {
			ErrorKind::IoError(err) => Error::IoError(err),
			// ErrorKind::InvalidEncoding is impossible, because we don't decode / encode UTF-8 strings.
			// ErrorKind::SequenceMustHaveLength is also impossible, we don't decode / encode slices.
			// ErrorKind::SizeLimit could occur during serialization, but we always know the size beforehand.
			_ => panic!("Unexpected bincode error.")
		}
	}
}
