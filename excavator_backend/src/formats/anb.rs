use zerocopy::{*, byteorder::{LittleEndian, U32, U64}};
use zerocopy_derive::*;

use super::binary::{ParserReflect, ParserReflectContext, ParserStruct, ParserStructError};
use super::wflz::WflzHeader;

type LE = LittleEndian;

const ANB_MAGIC: [u8; 4] = *b"YCSN";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AnbHeader {
	magic: [u8; 4],
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
	frame_info_1_pointer: U64<LE>,
	unknown_28: U32<LE>,
	frame_info_2_count: U32<LE>,
	frame_info_2_pointer: U64<LE>,
	unknown_38: U32<LE>,
	unknown_3C: U32<LE>,
	unknown_40: U32<LE>,
	unknown_44: U32<LE>,
	pub data_pointer: U64<LE>,
}

impl AnbHeader {
	fn is_magic_correct(&self) -> bool {
		self.magic == ANB_MAGIC
	}
}

impl ParserReflect for AnbHeader {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.follow_pointer::<AnbFrameInfo1Header>(self.frame_info_1_pointer.get() as usize);
		frame_info_2_pointer_table(context, self.frame_info_2_pointer.get() as usize, self.frame_info_2_count.get() as usize);
		context.follow_pointer::<AnbDataStart>(self.data_pointer.get() as usize);
	}
}

fn frame_info_2_pointer_table(context: &mut ParserReflectContext, start_offset: usize, count: usize) {
	let mut offset = start_offset;
	for i in 0..count {
		// either that first entry isn't a pointer, or something's wrong with my interpretation of something else
		// idk i'm just guessing how this format works
		if i != 0 {
			context.follow_pointer::<PointerTableEntry<AnbFrameInfo2Entry>>(offset);
		}
		offset += std::mem::size_of::<PointerTableEntry<AnbFrameInfo2Entry>>();
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbFrameInfo1Header {
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
struct AnbFrameInfo1Entry {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	entry_secondary_count: U32<LE>,
	entry_secondary_pointer: U64<LE>,
	unknown_28: U32<LE>,
	unknown_2C: U32<LE>,
	unknown_30: U32<LE>,
	unknown_34: U32<LE>,
}

impl ParserReflect for AnbFrameInfo1Entry {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		frame_info_1_secondary_pointer_table(context, self.entry_secondary_pointer.get() as usize, self.entry_secondary_count.get() as usize);
	}
}

fn frame_info_1_secondary_pointer_table(context: &mut ParserReflectContext, start_offset: usize, count: usize) {
	let mut offset = start_offset;
	for _i in 0..count {
		context.follow_pointer::<PointerTableEntry<AnbFrameInfo1EntrySecondary>>(offset);
		offset += std::mem::size_of::<PointerTableEntry<AnbFrameInfo1EntrySecondary>>();
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbFrameInfo1EntrySecondary {
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
}

impl ParserReflect for AnbFrameInfo1EntrySecondary {
	fn get_subordinates(&self, _context: &mut ParserReflectContext) {}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbFrameInfo1Final {
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
#[allow(non_snake_case)]
struct AnbFrameInfo2Entry {
	unknown_00: U32<LE>,
	entry_secondary_count: U32<LE>,
	entry_secondary_pointer: U64<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
}

impl ParserReflect for AnbFrameInfo2Entry {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		frame_info_2_secondary_pointer_table(context, self.entry_secondary_pointer.get() as usize, self.entry_secondary_count.get() as usize);
	}
}

fn frame_info_2_secondary_pointer_table(context: &mut ParserReflectContext, start_offset: usize, count: usize) {
	let mut offset = start_offset;
	for i in 0..count {
		if i != 0 {
			context.follow_pointer::<PointerTableEntry<AnbFrameInfo2EntrySecondary>>(offset);
		}
		offset += std::mem::size_of::<PointerTableEntry<AnbFrameInfo2EntrySecondary>>();
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbFrameInfo2EntrySecondary {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	entry_tertiary_pointer: U64<LE>
}

impl ParserReflect for AnbFrameInfo2EntrySecondary {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.follow_pointer::<AnbFrameInfo2EntryTertiary>(self.entry_tertiary_pointer.get() as usize);
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbFrameInfo2EntryTertiary {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	sprite_width: U32<LE>,
	sprite_height: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
	unknown_20: U32<LE>,
	unknown_24: U32<LE>,
}

impl ParserReflect for AnbFrameInfo2EntryTertiary {
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
struct AnbBlock {
	magic: [u8; 4], // always FF FF FF 00
	length: U32<LE>,
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

pub fn get_the_stupid_sprite_size(file: &[u8]) -> Result<[u32; 2], ParserStructError> {
	let anb_header = ParserStruct::<AnbHeader>::new(file, 0).retrieve()?;
	println!("anb_header.frame_info_2_pointer: {:?}", anb_header.frame_info_2_pointer);
	let frame_info_2_entry_pointer = ParserStruct::<U64<LE>>::new(file, anb_header.frame_info_2_pointer.get() as usize + 8).retrieve()?;
	println!("frame_info_2_entry_pointer: {:?}", frame_info_2_entry_pointer);
	let frame_info_2_entry = ParserStruct::<AnbFrameInfo2Entry>::new(file, frame_info_2_entry_pointer.get() as usize).retrieve()?;
	let frame_info_2_entry_secondary_pointer = ParserStruct::<U64<LE>>::new(file, frame_info_2_entry.entry_secondary_pointer.get() as usize).retrieve()?;
	println!("frame_info_2_entry_secondary_pointer: {:?}", frame_info_2_entry_secondary_pointer);
	// let frame_info_2_entry_secondary = ParserStruct::<AnbFrameInfo2EntrySecondary>::new(file, frame_info_2_entry_secondary_pointer.get() as usize).retrieve()?;
	// println!("frame_info_2_entry_secondary.entry_tertiary_pointer: {:?}", frame_info_2_entry_secondary.entry_tertiary_pointer);
	// let frame_info_2_entry_tertiary = ParserStruct::<AnbFrameInfo2EntryTertiary>::new(file, frame_info_2_entry_secondary.entry_tertiary_pointer.get() as usize).retrieve()?;
	let frame_info_2_entry_tertiary = ParserStruct::<AnbFrameInfo2EntryTertiary>::new(file, frame_info_2_entry_secondary_pointer.get() as usize).retrieve()?;
	Ok([frame_info_2_entry_tertiary.sprite_width.get(), frame_info_2_entry_tertiary.sprite_height.get()])
}
