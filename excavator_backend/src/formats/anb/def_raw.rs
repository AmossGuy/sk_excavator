use zerocopy::byteorder::*;
use zerocopy_derive::*;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[expect(non_snake_case)]
pub struct Header {
	pub magic: [u8; 4],
	pub unknown_04: U32<LE>,
	pub unknown_08: U32<LE>,
	pub unknown_0C: U32<LE>,
	pub unknown_10: U32<LE>,
	pub unknown_14: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeCommon {
	pub kind: U32<LE>,
	pub child_count: U32<LE>,
	pub child_array_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaRect {
	pub center_x: F32<LE>,
	pub center_y: F32<LE>,
	pub center_z: F32<LE>,
	pub extents_x: F32<LE>,
	pub extents_y: F32<LE>,
	pub extents_z: F32<LE>,
	pub angle: F32<LE>,
	pub padding: U32<LE>,
}
