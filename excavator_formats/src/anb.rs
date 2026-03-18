use zerocopy::{*, byteorder::{LittleEndian, U32, U64}};

use crate::util_binary::{ParserReflect, ParserReflectContext};
use crate::wflz::WflzHeader;

type LE = LittleEndian;

const ANB_MAGIC: [u8; 4] = *b"YCSN";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AnbHeader {
	pub magic: [u8; 4],
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
	pub frame_info_1_pointer: U64<LE>,
	unknown_28: U32<LE>,
	unknown_2C: U32<LE>,
	unknown_30: U32<LE>,
	unknown_34: U32<LE>,
	unknown_38: U32<LE>,
	unknown_3C: U32<LE>,
	unknown_40: U32<LE>,
	unknown_44: U32<LE>,
	pub data_pointer: U64<LE>,
}

impl AnbHeader {
	pub fn is_magic_correct(&self) -> bool {
		self.magic == ANB_MAGIC
	}
}

impl ParserReflect for AnbHeader {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.follow_pointer::<AnbFrameInfo1Header>(self.frame_info_1_pointer.get() as usize);
		context.follow_pointer::<AnbDataStart>(self.data_pointer.get() as usize);
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AnbFrameInfo1Header {
	unknown_00: U32<LE>, // possibly a pointer, but it points to the second half of our current version of AnbHeader, meaning that that needs to be split to reflect the format correctly
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	pointer_table_pointer: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	frame_count: U32<LE>,
}

impl ParserReflect for AnbFrameInfo1Header {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		frame_info_1_pointer_table(context, self.pointer_table_pointer.get() as usize, self.frame_count.get() as usize);
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct PointerTableEntry<T> {
	pointer: U64<LE>,
	phantom: std::marker::PhantomData<for<'a> fn(&'a [u8]) -> &'a T>,
}

impl<T: FromBytes + KnownLayout + Immutable + ParserReflect + 'static> ParserReflect for PointerTableEntry<T> {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.follow_pointer::<T>(self.pointer.get() as usize);
	}
}

fn frame_info_1_pointer_table(context: &mut ParserReflectContext, start_offset: usize, frame_count: usize) {
	let mut offset = start_offset;
	for _i in 0..frame_count {
		context.follow_pointer::<PointerTableEntry<AnbFrameInfo1Entry>>(offset);
		offset += std::mem::size_of::<PointerTableEntry<AnbFrameInfo1Entry>>();
	}
	context.follow_pointer::<PointerTableEntry<AnbFrameInfo1Final>>(offset);
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AnbFrameInfo1Entry {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
	unknown_20: U32<LE>,
	unknown_24: U32<LE>,
	unknown_28: U32<LE>,
	unknown_2C: U32<LE>,
	unknown_30: U32<LE>,
	unknown_34: U32<LE>,
}

impl ParserReflect for AnbFrameInfo1Entry {
	fn get_subordinates(&self, _context: &mut ParserReflectContext) {}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AnbFrameInfo1Final {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
}

impl ParserReflect for AnbFrameInfo1Final {
	fn get_subordinates(&self, _context: &mut ParserReflectContext) {}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct AnbDataStart {
	unknown: [U32<LE>; 6],
	pub wflz: WflzHeader,
}

impl ParserReflect for AnbDataStart {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		self.wflz.get_subordinates(context);
	}
}

/*
#[derive(Debug)]
pub struct AnbBlock {
	pub magic: [u8; 4], // always FF FF FF 00
	pub length: U32<LE>,
}

impl ParserReflect for AnbBlock {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		let file = context.file();
		
		let self_offset = std::ptr::from_ref(self).addr() - file.as_ptr().addr();
		let content_offset = self_offset + std::mem::size_of::<Self>();
		
		let content_s = ParserStruct::<[u8]>::new(file, content_offset).retrieve_with_len(self.length.get() as usize);
		context.bullshit(content_s);
	}
}
*/
