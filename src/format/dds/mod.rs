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
		const PF_HAS_ALPHA = 0x1;
		// Older flag, should not be used. Preffer PF_HAS_ALPHA instead.
		const PF_ALPHA = 0x2;
		const PF_FOUR_CC = 0x4;
		const PF_RGB = 0x40;
	}
}

/// A texture loaded from a DDS file.
pub struct Texture {
	width: u32, height: u32,
	data: Vec<u8>
}

impl Texture {
	/// Returns the width and height of the texture.
	pub fn dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	/// Returns a slice of the raw bytes of the texture.
	pub fn as_raw(&self) -> &[u8] {
		&self.data
	}
}

/// Additional features, added by the DirectX 10 DDS format.
// mod ext;

/// Reads a DDS file.
pub fn read(reader: &mut io::Read) -> Result<Texture> {
	{
		let mut magic_number: [u8; 4] = unsafe { mem::uninitialized() };

		reader.read_exact(&mut magic_number)?;

		if magic_number != *b"DDS " {
			return Err(Error::FormatError("DDS magic number not found".to_string()));
		}
	}

	let header: Header;

	{
		// Since bincode does not fail if there is not data in the reader, we must read the buffer beforehand
		// and then ask bincode to deserialize it.
		let mut buf = [0u8; 124];

		reader.read_exact(&mut buf)?;

		header = bincode::deserialize(&buf)?;
	}

	if header.size as usize != mem::size_of::<Header>() {
		let msg = format!("Header size mismatch. Expected: {} bytes, found: {} bytes", mem::size_of::<Header>(), header.size);
		return Err(Error::FormatError(msg))
	}

	let pixel_format = &header.format;

	if pixel_format.size as usize != mem::size_of::<PixelFormat>() {
		let msg = format!("Pixel format data size mismatch. Expected: {} bytes, found: {} bytes.", mem::size_of::<PixelFormat>(), pixel_format.size);
		return Err(Error::FormatError(msg));
	}

	let width = header.width;
	let height = header.height;

	// Parse the pixel format structure to get information.
	if pixel_format.flags.intersects(PF_FOUR_CC) {
		unimplemented!();
	} else {
		let has_alpha = pixel_format.flags.intersects(PF_HAS_ALPHA | PF_ALPHA);
		let bpp = if has_alpha { 32 } else { 24 };

		let data_len = width * height * (bpp / 8);

		let mut data = Vec::with_capacity(data_len as usize);

		unsafe {
			data.set_len(data_len as usize);
		}

		reader.read_exact(&mut data)?;

		let texture = Texture {
			width, height,
			data
		};

		Ok(texture)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::{Path, PathBuf};
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
	fn fail_wrong_size() {
		#[derive(Serialize)]
		struct Data {
			magic: [u8; 4],
			header: Header
		}

		let mut hdr: Data = unsafe { mem::zeroed() };

		hdr.magic = *b"DDS ";
		hdr.header.size = 10;

		let hdr_bound = bincode::Bounded(128);

		{
			let hdr = bincode::serialize(&hdr, hdr_bound).unwrap();

			let mut hdr_view = &hdr[..];

			let result = read(&mut hdr_view);

			assert!(result.is_err());
		}

		hdr.header.format.size = 31;

		{
			let hdr = bincode::serialize(&hdr, hdr_bound).unwrap();

			let mut hdr_view = &hdr[..];

			let result = read(&mut hdr_view);

			assert!(result.is_err());
		}
	}

	use ::image;

	fn data_dir() -> PathBuf {
		Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
	}

	fn read_uncompressed_dds() -> Result<Texture> {
		let file_path = data_dir().join("uncomp").join("rust-uncomp-no-mipmaps.dds");
		let mut uncomp_dds = File::open(file_path).unwrap();
		read(&mut uncomp_dds)
	}

	#[test]
	fn read_uncompressed() {
		let result = read_uncompressed_dds();

		assert!(result.is_ok());
	}

	#[test]
	#[ignore]
	fn read_uncompressed_to_bmp() {
		let texture = read_uncompressed_dds().unwrap();

		let mut output = File::create(&Path::new("test.bmp")).unwrap();

		let mut bmp = image::bmp::BMPEncoder::new(&mut output);

		let _ = bmp.encode(&texture.data, texture.width, texture.height, image::ColorType::RGBA(8)).unwrap();
	}
}
