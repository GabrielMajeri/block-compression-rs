//! # Reference
//! See:
//! - [Microsoft Docs sample loader](https://docs.microsoft.com/en-us/windows/uwp/gaming/complete-code-for-ddstextureloader)
//! - [MSDN DDS Programming Guide](https://msdn.microsoft.com/library/windows/desktop/bb943991)

use std::io;
use std::mem;
use std::slice;
use super::{Error, Result};

/// Reads a DDS file.
pub fn read(reader: &mut io::Read) -> Result<()> {
	#[repr(C, packed)]
	struct Header {
		magic_number: [u8; 4]
	}

	let mut header = unsafe { mem::uninitialized() };

	let header_bytes = unsafe {
		slice::from_raw_parts_mut(
			mem::transmute::<&mut Header, *mut u8>(&mut header),
			mem::size_of::<Header>()
		)
	};

	reader.read_exact(header_bytes)?;

	if header.magic_number != *b"DDS " {
		return Err(Error::FormatError("DDS magic number not found".to_string()));
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;
	use std::fs::File;

	#[test]
	fn read_uncompressed() {
		let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

		let mut uncomp_dds = File::open(data_dir.join("uncomp").join("rust-uncomp-no-mipmaps.dds")).unwrap();
		let _r = read(&mut uncomp_dds).unwrap();
	}
}
