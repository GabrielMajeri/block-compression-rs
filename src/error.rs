use std::io;
use std::result;
use std::fmt;
use bincode;

/// Error type for the library.
#[derive(Debug)]
pub enum Error {
	/// Some unexpected data was encountered while reading a file.
	FormatError(String),
	/// An I/O error was encountered while reading / writing an image.
	IoError(io::Error)
}

/// Type returned by most of the library's functions.
pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::FormatError(ref msg) => msg.fmt(formatter),
			Error::IoError(ref err) => err.fmt(formatter)
		}
	}
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn debug_and_display_trait() {
		let err = Error::FormatError("Uh-oh, something's wrong!".to_string());

		let debug = format!("{:?}", err);
		let display = format!("{}", err);

		assert_eq!(debug, "FormatError(\"Uh-oh, something\\\'s wrong!\")");
		assert_eq!(display, "Uh-oh, something's wrong!");
	}

	#[test]
	fn from_io_error() {
		let err = io::Error::new(io::ErrorKind::NotFound, "something not found");

		let _ = Error::from(err);
	}
}
