//! # Reference
//! See:
//! - [Microsoft Docs sample loader](https://docs.microsoft.com/en-us/windows/uwp/gaming/complete-code-for-ddstextureloader)
//! - [MSDN DDS Programming Guide](https://msdn.microsoft.com/library/windows/desktop/bb943991)

use std::io;
use std::mem;
use bincode;
use super::{Error, Result};

#[repr(C, packed)]
#[derive(Serialize, Deserialize)]
struct Header {
	size: u32,
	flags: HeaderFlags,
	height: u32,
	width: u32,
	// Pitch (scan line length) for uncompressed textures.
	// Size in bytes of top-level texture for compressed textures.
	pitch_or_linear_size: u32,
	// Depth for 3D textures.
	depth: u32,
	mipmap_count: u32,
	_unused1: [u32; 11],
	format: PixelFormat,
	caps: Capabilities,
	caps2: Capabilties2,
	_unused2: [u32; 3]
}

bitflags! {
	#[derive(Serialize, Deserialize)]
	struct HeaderFlags: u32 {
		const CAPS = 0x1;
		const HEIGHT = 0x2;
		const WIDTH = 0x4;
		const PIXEL_FORMAT = 0x1000;

		// All DDS files should have these bits set.
		// However, when reading, do not rely on other programs to write these.
		const REQUIRED = CAPS.bits | HEIGHT.bits | WIDTH.bits | PIXEL_FORMAT.bits;

		const UNCOMPRESSED_PITCH = 0x8;
		const COMPRESSED_PITCH = 0x80000;

		const HAS_MIPMAPS = 0x20000;

		const DEPTH_TEXTURE = 0x800000;
	}
}

bitflags! {
	#[derive(Serialize, Deserialize)]
	struct Capabilities: u32 {
		// Required.
		const TEXTURE = 0x1000;

		// Contains more than one type of textures.
		const COMPLEX = 0x8;

		// Contains a mipmap.
		const MIPMAP = 0x400000;
	}
}

bitflags! {
	#[derive(Serialize, Deserialize)]
	struct Capabilties2: u32 {
		const CUBEMAP = 0x200;
		const CUBEMAP_POSITIVE_X = 0x400;
		const CUBEMAP_NEGATIVE_X = 0x800;
		const CUBEMAP_POSITIVE_Y = 0x1000;
		const CUBEMAP_NEGATIVE_Y = 0x2000;
		const CUBEMAP_POSITIVE_Z = 0x4000;
		const CUBEMAP_NEGATIVE_Z = 0x8000;

		// Direct3D 10 and newer require all cubemaps to be complete.
		const CUBEMAP_ALL_FACES = 0x400 | 0x800 | 0x1000 | 0x2000 | 0x4000 | 0x8000;

		const TEXTURE_3D = 0x200000;
	}
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize)]
struct PixelFormat {
	size: u32,
	flags: PixelFormatFlags,
	// Values:
	// - DXT 1 through 5
	// - DX10 indicates the presence of the HeaderExt structure after the Header.
	four_cc: [u8; 4],
	rgb_bit_count: u32,
	red_mask: u32,
	green_mask: u32,
	blue_mask: u32,
	alpha_mask: u32
}

bitflags! {
	#[derive(Serialize, Deserialize)]
	struct PixelFormatFlags: u32 {
		const HAS_ALPHA = 0x1;
	}
}

/// Additional features, added by the DirectX 10 DDS format.
// mod ext;

/// Reads a DDS file.
pub fn read(reader: &mut io::Read) -> Result<()> {
	let mut magic_number: [u8; 4] = unsafe { mem::uninitialized() };

	reader.read_exact(&mut magic_number)?;

	if magic_number != *b"DDS " {
		return Err(Error::FormatError("DDS magic number not found".to_string()));
	}

	let limit = bincode::Bounded(mem::size_of::<Header>() as u64);
	let header: Header = bincode::deserialize_from(reader, limit)?;

	if header.size as usize != mem::size_of::<Header>() {
		let msg = format!("Header size mismatch. Expected: {} bytes, found: {} bytes", mem::size_of::<Header>(), header.size);
		return Err(Error::FormatError(msg))
	}

	let pixel_format = &header.format;

	if pixel_format.size as usize != mem::size_of::<PixelFormat>() {
		let msg = format!("Pixel format data size mismatch. Expected: {} bytes, found: {} bytes.", mem::size_of::<PixelFormat>(), pixel_format.size);
		return Err(Error::FormatError(msg));
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;
	use std::fs::File;

	#[test]
	fn fail_magic_number() {
		let data = "not dds";
		let mut data_view = data.as_bytes();

		let result = read(&mut data_view);

		assert!(result.is_err());
	}

	#[test]
	fn fail_not_enough_data() {
		let data = "DDS 1234";
		let mut data_view = data.as_bytes();

		let result = read(&mut data_view);

		assert!(result.is_err());
	}

	#[test]
	fn read_uncompressed() {
		let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
		let file_path = data_dir.join("uncomp").join("rust-uncomp-no-mipmaps.dds");

		let mut uncomp_dds = File::open(file_path).unwrap();
		let result = read(&mut uncomp_dds);

		assert!(result.is_ok());
	}
}
