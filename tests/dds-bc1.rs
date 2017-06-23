extern crate block_compression as bc;
extern crate image;

use std::path::Path;
use std::fs::File;

use bc::format::dds::Texture;

fn read_dds(path: &str) -> bc::Result<Texture> {
	let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(path);
	let mut dds = File::open(file_path).unwrap();
	bc::format::dds::read(&mut dds)
}

fn texture_to_bmp(path: &str, texture: &Texture) {
	let mut output = File::create(path).unwrap();
	let mut bmp = image::bmp::BMPEncoder::new(&mut output);

	let (width, height) = texture.dimensions();

	let _ = bmp.encode(texture.as_bytes(), width, height, image::ColorType::RGBA(8)).unwrap();
}

#[test]
fn read_uncompressed() {
	let result = read_dds("uncomp/rust-uncomp-no-mipmaps.dds");

	assert!(result.is_ok());
}

#[test]
#[ignore]
fn read_uncompressed_to_bmp() {
	let texture = read_dds("uncomp/rust-uncomp-no-mipmaps.dds").unwrap();

	texture_to_bmp("test_uncomp.bmp", &texture);
}

#[test]
fn read_compressed_bc1() {
	let result = read_dds("bc1/rust-bc1-linear-no-mipmaps.dds");

	assert!(result.is_ok());
}

#[test]
#[ignore]
fn read_compressed_bc1_to_bmp() {
	let texture = read_dds("bc1/rust-bc1-linear-no-mipmaps.dds").unwrap();

	let (width, height) = texture.dimensions();

	let uncomp = bc::bc1::decode(texture.as_bytes(), width, height).unwrap();

	let mut output = File::create("test_bc1.bmp").unwrap();
	let mut bmp = image::bmp::BMPEncoder::new(&mut output);

	let _ = bmp.encode(&uncomp, width, height, image::ColorType::RGB(8)).unwrap();
}
