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
