use zerocopy::byteorder::*;
use zerocopy_derive::*;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct Header {
	pub magic: [u8; 4],
	pub file_count: U32<LE>,
	pub data_array_pointer: U64<LE>,
	pub name_array_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct FileHeader {
	pub size: U64<LE>,
	pub time: U64<LE>,
	pub filename_hash: U32<LE>,
	pub flags: U32<LE>,
	pub specials: U32<LE>,
	pub padding: U32<LE>,
}
