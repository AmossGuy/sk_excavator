use zerocopy::{FromBytes, LittleEndian as LE, U32, U64};
use zerocopy_derive::*;

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[expect(non_snake_case)]
pub struct Header {
	pub magic: [u8; 4],
	pub unknown_04: U32<LE>,
	pub unknown_08: U32<LE>,
	pub unknown_0C: U32<LE>,
	pub unknown_10: U32<LE>,
	pub unknown_14: U32<LE>,
	pub unknown_18: U32<LE>,
	pub unknown_1C: U32<LE>,
	pub root_node_pointer: U64<LE>,
}
