use bstr::BString;
use std::io::{BufRead, Seek};
use zerocopy::{FromBytes, LittleEndian as LE, U32, U64};
use zerocopy_derive::*;

use crate::parse_old::ParseReader;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct StlHeader {
	magic: U64<LE>, // always zeros
	entry_count: U32<LE>,
	field_count: U32<LE>,
	data_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct StbOrStmHeader {
	magic: U64<LE>, // always zeros
	entry_count: U32<LE>,
	field_count: U32<LE>,
	checksums_pointer: U64<LE>,
	data_pointer: U64<LE>,
	extra1: StbOrStmHeaderExtra,
	extra2: StbOrStmHeaderExtra,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct StbOrStmHeaderExtra {
	magic: U32<LE>, // always zeros
	extra_entry_count: U32<LE>,
	pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[expect(dead_code)] // we'll get back to this
struct StbOrStmDataExtra {
	piece_count: U64<LE>,
	magic: U64<LE>, // always 0x10
	// followed by twice as many u32's as the piece_count
}

// Right now it's just pub for the sake of throwing something together. I don't yet know whether I'll eventually end up with this type of struct being consistently pub, or never pub.
#[derive(Copy, Clone, Debug)]
pub struct StHeaderCommon {
	pub entry_count: u32,
	pub field_count: u32,
	pub data_pointer: u64,
}

impl From<StlHeader> for StHeaderCommon {
	fn from(value: StlHeader) -> Self {
		Self {
			entry_count: value.entry_count.get(),
			field_count: value.field_count.get(),
			data_pointer: value.data_pointer.get(),
		}
	}
}

impl From<StbOrStmHeader> for StHeaderCommon {
	fn from(value: StbOrStmHeader) -> Self {
		Self {
			entry_count: value.entry_count.get(),
			field_count: value.field_count.get(),
			data_pointer: value.data_pointer.get(),
		}
	}
}

pub fn read_st_header<R: BufRead + Seek>(reader: &mut R, is_stl: bool) -> anyhow::Result<StHeaderCommon> {
	let mut parser = ParseReader::new(reader);
	let header = if is_stl {
		parser.read_struct::<StlHeader>(0)?.into()
	} else {
		parser.read_struct::<StbOrStmHeader>(0)?.into()
	};
	Ok(header)
}

pub fn read_st_cell<R: BufRead + Seek>(reader: &mut R, header: &StHeaderCommon, index: usize) -> anyhow::Result<BString> {
	let mut parser = ParseReader::new(reader);
	let string_count = u64::from(header.entry_count) * u64::from(header.field_count);
	let mut arrayer = parser.read_struct_array::<U64<LE>>(header.data_pointer, string_count)?;
	let string_pointer = arrayer.nth(index).ok_or(anyhow::anyhow!("string index out of range: {}", index))??.get();
	Ok(parser.read_null_terminated_string(string_pointer)?)
}
