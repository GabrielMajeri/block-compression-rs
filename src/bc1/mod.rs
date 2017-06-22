//! # Algorithm information
//! This section is recommended reading for people who want to decide which BCn algorithm to use.
//! Also for people who want to learn about BC1 internals.
//!
//! The Block-Compression 1 algorithm (also known as DXT1) is a fixed block-size compression texture compression algorithm.
//!
//! **Input data**: uncompressed R8-G8-B8 image (only a 1-bit alpha is supported, i.e. either fully opaque or transparent, and requires special support in shader).
//! **Output data**: compressed "blocks" - each block is 8-bytes long, and stores information for a 4x4 pixel area in the original image.
//!
//! Block structure:
//! ```rust,no_run
//! #[repr(C)]
//! struct Block {
//! 	color0: R5-G6-B5,
//! 	color1: R5-G6-B5,
//! 	palette: [u8; 4]
//! }
//! ```
//! Basically, a block represents a "line" in the RGB color space (`color0` and `color1` being the two endpoints),
//! and the palette is used to linearly interpolate between them. This can produce banding artifacts, which is why higher quality algorithms like BC7 exist.

use super::{Result, Error};
use std::mem;
use std::slice;

// TODO: support mipmaps

// Colors are stored as RGB 5:6:5.
#[repr(C)]
#[derive(Copy, Clone)]
struct R5G6B5(u16);

#[repr(C)]
#[derive(Copy, Clone)]
struct Block {
	color0: R5G6B5,
	color1: R5G6B5,

	palette: [u8; 4]
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct R8G8B8(u8, u8, u8);

impl R5G6B5 {
	fn as_r8g8b8(&self) -> R8G8B8 {
		let r = self.0 >> 11;
		let g = (self.0 >> 5) & ((1 << 6) - 1);
		let b = self.0 & ((1 << 5) - 1);

		const RB: f32 = 255.0 / 31.0;
		const G: f32 = 255.0 / 63.0;

		let r8 = (r as f32 * RB).floor() as u8;
		let g8 = (g as f32 * G).floor() as u8;
		let b8 = (b as f32 * RB).floor() as u8;

		R8G8B8(r8, g8, b8)
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
		let ptr = mem::transmute::<*const _, *mut R8G8B8>(output.as_mut_ptr());
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

			for offset_x in 0..4 {
				for offset_y in 0..4 {
					// TODO: finish color interpolation code.
					set_pixel(out_x + offset_x, out_y + offset_y, block.color0.as_r8g8b8());
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
