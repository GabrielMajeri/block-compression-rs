#[repr(C, packed)]
#[derive(Serialize, Deserialize)]
struct HeaderExt {
	// TODO: how to get DXGI_FORMAT?
	format: u32,
	dimension: ResourceDimension,
	misc_flags: MiscFlags1,
	// For a normal texture, this should be 1.
	// For cubemaps, should be the number of cubes.
	array_size: u32,
	misc_flags2: MiscFlags2
}

#[repr(u32)]
#[derive(Serialize, Deserialize)]
enum ResourceDimension {
	Texture1D = 2,
	Texture2D = 3,
	Texture3D = 4
}

bitflags! {
	#[derive(Serialize, Deserialize)]
	struct MiscFlags1: u32 {
		const CUBEMAP_TEXTURE = 0x4;
	}
}
bitflags! {
	#[derive(Serialize, Deserialize)]
	struct MiscFlags2: u32 {
		const ALPHA_MODE_UNKNOWN = 0x0;
		const ALPHA_MODE_STRAIGHT = 0x1;
		const ALPHA_MODE_PREMULTIPLIED = 0x2;
		const ALPHA_MODE_OPAQUE = 0x3;
		const ALPHA_MODE_CUSTOM = 0x4;
	}
}
