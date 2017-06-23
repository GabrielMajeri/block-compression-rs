//! # Reference
//!
//! - [DXT1 on Wikipedia](https://en.wikipedia.org/wiki/S3_Texture_Compression#DXT1)
//! - [BC1 on Microsoft Docs](https://docs.microsoft.com/en-us/windows/uwp/graphics-concepts/block-compression#bc1)
//! - [Nathan Reed's article](http://reedbeta.com/blog/understanding-bcn-texture-compression-formats/#bc1)
//! - [Matej Tomčík's DXT Decompression library](http://www.matejtomcik.com/Public/KnowHow/DXTDecompression/)
//!
//! # Algorithm information
//! This section is recommended reading for people who want to decide which BCn algorithm to use.
//! Also for people who want to learn about BC1 internals.
//!
//! The Block-Compression 1 algorithm (also known as DXT1) is a fixed block-size compression texture compression algorithm.
//!
//! - **Input data**: uncompressed R8-G8-B8 image (only a 1-bit alpha is supported, i.e. either fully opaque or transparent, and requires special support in shader).
//!
//! - **Output data**: compressed "blocks" - each block is 8-bytes long, and stores information for a 4x4 pixel area in the original image.
//!
//! Each block represents a palette of 4 colors (the 2 endpoints, the other 2 are interpolated between them).
//!
//! The interpolation formula is:
//! - color2 = (color0 * 2 + color1) / 3
//! - color3 = (color1 * 2 + color0) / 3
//!
//! The other bits are 2-bit indices of the of the color.
//! e.g, let's say the first 2 bits in the `indices` is 1, that means `color1` is the color of the first pixel (0, 0);
//!
//! Block structure:
//!
//! ```rust,no_run
//! # struct R5G6B5(u16);
//! #[repr(C)]
//! struct Block {
//! 	color0: R5G6B5,
//! 	color1: R5G6B5,
//! 	indices: u32
//! }
//! ```
//!
//! Basically, a block represents a "line" in the RGB color space (`color0` and `color1` being the two endpoints),
//! and the indices used to chose between the colors. This can produce banding artifacts, which is why higher quality algorithms like BC7 exist.

use super::{Result, Error};
use std::mem;
use std::slice;

// TODO: support mipmaps

#[repr(C)]
#[derive(Copy, Clone)]
struct Block {
	color0: R5G6B5,
	color1: R5G6B5,

	indices: u32
}

impl Block {
	fn colors(&self) -> [R8G8B8; 4] {
		let c0 = self.color0.as_r8g8b8();
		let c1 = self.color1.as_r8g8b8();

		let (r0, g0, b0) = (c0.0, c0.1, c0.2);
		let (r1, g1, b1) = (c1.0, c1.1, c1.2);

		let c2;
		let c3;

		if c0.as_u32() > c1.as_u32() {
			let lerp = |el0, el1| ((2 * el0 as u16 + el1 as u16) / 3) as u8;

			c2 = R8G8B8(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1));
			c3 = R8G8B8(lerp(r1, r0), lerp(g1, g0), lerp(b1, b0));
		} else {
			c2 = R8G8B8((r0 + r1) / 2, (g0 + g1) / 2, (b0 + b1) / 2);
			c3 = R8G8B8(0, 0, 0);
		}

		[c0, c1, c2, c3]
	}
}

// Colors are stored as RGB 5:6:5.
#[repr(C)]
#[derive(Copy, Clone)]
struct R5G6B5(u16);

impl R5G6B5 {
	fn as_r8g8b8(&self) -> R8G8B8 {
		let r = self.0 >> 11;
		let g = (self.0 >> 5) & ((1 << 6) - 1);
		let b = self.0 & ((1 << 5) - 1);

		// From https://stackoverflow.com/a/9069480
		let r8 = ((r * 527 + 23) >> 6) as u8;
		let g8 = ((g * 259 + 33) >> 6) as u8;
		let b8 = ((b * 527 + 23) >> 6) as u8;

		R8G8B8(r8, g8, b8)
	}
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct R8G8B8(u8, u8, u8);

impl R8G8B8 {
	fn as_u32(&self) -> u32 {
		(self.0 as u32) | ((self.1 as u32) << 8) | ((self.2 as u32) << 16)
	}
}

/// Decodes an image compressed with the BC1 algorithm.
pub fn decode(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
	if data.len() % BLOCK_SIZE != 0 {
		return Err(Error::FormatError("Length of BC1 data to decode is not a multiple of the block size.".to_string()));
	}

	let src = unsafe {
		let ptr = mem::transmute::<*const _, *const Block>(data.as_ptr());
		let blocks = data.len() / BLOCK_SIZE;
		slice::from_raw_parts(ptr, blocks)
	};

	let output_len = calculate_output_len(data.len());

	let mut output = Vec::with_capacity(output_len);

	unsafe {
		output.set_len(output_len);
	}

	let dest = unsafe {
		let ptr = mem::transmute::<*const u8, *mut R8G8B8>(output.as_mut_ptr());
		let pixels = output_len / 24;
		slice::from_raw_parts_mut(ptr, pixels)
	};

	decode_internal(src, width, height, dest);

	Ok(output)
}

fn calculate_output_len(input_len: usize) -> usize {
	(input_len / BLOCK_SIZE)
		// 4x4 pixels per block
		* 16
		// 3 component pixels, 1 byte / component.
		* 24
}

fn decode_internal(data: &[Block], width: u32, height: u32, output: &mut [R8G8B8]) {
	let columns = columns(width);
	let rows = rows(height);

	let get_block = |x, y| &data[(x * columns + y) as usize];
	let mut set_pixel = |x, y, value| output[(x * width + y) as usize] = value;

	let mut out_x = 0;
	let mut out_y = 0;

	// Each 8-byte block unpacks to a 4x4 pixel area.
	// - (row, column) index the blocks.
	// - (out_x, out_y) index the pixels in the output image.
	for row in 0..rows {
		for column in 0..columns {
			let block = get_block(row, column);

			let colors = block.colors();

			const MASK: u32 = (1 << 2) - 1;

			let indices = block.indices;

			let mut n = 0;

			for offset_x in 0..4 {
				for offset_y in 0..4 {
					let index = (indices >> (n * 2)) & MASK;
					n += 1;

					set_pixel(out_x + offset_x, out_y + offset_y, colors[index as usize]);
				}
			}

			// Advance one block to the right.
			out_y += 4;
		}

		// Start unpacking a new line.
		out_x += 4;
		out_y = 0;
	}
}

const BLOCK_SIZE: usize = 8;

use std::cmp::max;

/// Returns the number if it's not 0, returns 1 otherwise.
fn clamp_non_zero(num: u32) -> u32 {
	max(1, num)
}

fn columns(width: u32) -> u32 {
	clamp_non_zero((width + 3) / 4)
}

fn rows(height: u32) -> u32 {
	clamp_non_zero((height + 3) / 4 )
}
