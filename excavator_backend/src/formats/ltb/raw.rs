use zerocopy::{FromBytes, LittleEndian as LE, U32, U64};
use zerocopy_derive::*;

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct LtbHeader {
	pub unknown_00: U32<LE>,
	pub unknown_04: U32<LE>,
	pub unknown_08: U32<LE>,
	pub unknown_0C: U32<LE>,
	pub rows: [LtbHeaderRow; 8],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct LtbHeaderRow {
	pub unknown: U32<LE>, // usually the same as entry_count below. what does that mean?
	pub entry_count: U32<LE>,
	pub entry_pointer: U64<LE>,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct LayerMetadata {
	pub name: [u8; 32],
	pub unknown_arr1: [U32<LE>; 14],
	pub unknown_38: U32<LE>,
	pub chunkmap_width: U32<LE>,
	pub chunkmap_height: U32<LE>,
	pub chunkmap_offset: U32<LE>,
	pub unknown_arr2: [U32<LE>; 6],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct ImageMetadata {
	pub unknown_00: U32<LE>,
	pub is_compressed: U32<LE>,
	pub image_width: U32<LE>,
	pub image_height: U32<LE>,
	pub unknown_more: [U32<LE>; 14],
	pub data_size: U32<LE>,
}
