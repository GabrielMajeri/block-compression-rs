//! Tools for working with block compressed textures.
//!
//! # Examples
//! ```rust
//! extern crate block_compression as bc;
//! ```

#![deny(missing_docs)]

// Used in some file formats' headers.
#[macro_use]
extern crate bitflags;

// Used for reading / writing the files with the help of structures.
extern crate bincode;

// Needed for `bincode`.
#[macro_use]
extern crate serde_derive;

/// Enum containing all supported block-compression algorithms.
#[derive(Copy, Clone)]
pub enum BCAlgorithm {
	/// Block compression 1, stores RGB data, with an optional 1-bit alpha.
	/// This is the recommended format for most textures, providing best compression,
	/// while the others should be used in special cases, as documented.
	BC1
}

mod error;

pub use error::{Error, Result};

/// BC1 stores compressed RGB data, with an optional 1-bit alpha channel.
pub mod bc1;

/// Tools to operate on file formats that store block-compressed data.
///
/// This includes file formats such as *.dds, *.ktx.
pub mod format;
