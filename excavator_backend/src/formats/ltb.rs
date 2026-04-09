use crate::parse::*;
use std::io::{BufRead, Seek};
use zerocopy::{*, LittleEndian as LE};
use zerocopy_derive::*;

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct LtbHeader {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	entries: [LtbHeaderRow; 8],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct LtbHeaderRow {
	unknown: U32<LE>, // usually the same as entry_count below. what does that mean?
	entry_count: U32<LE>,
	entry_pointer: U64<LE>,
}

pub fn parse_ltb<R: BufRead + Seek>(reader: &mut R) -> ParseResult<ParsedLtb> {
	let mut reader = ParseReader::new(reader);
	let header = reader.read_struct::<LtbHeader>(0)?;
	
	let wflz_array_count = header.entries[7].entry_count.get();
	let wflz_array_pointer = header.entries[7].entry_pointer.get();
	
	let wflz_pointers = reader
		.read_struct_array::<U64<LE>>(wflz_array_pointer, wflz_array_count.into())?
		.collect::<Result<Vec<_>, _>>()?;
	
	let wflz_data = wflz_pointers.into_iter().map(|poin| {
		let mut cursor = reader.cursor(poin.get())?;
		Ok(super::wflz::extract_wflz_from_reader(cursor.inner_reader())?)
	}).collect::<Vec<_>>();
	
	Ok(ParsedLtb { header, wflz_data })
}

pub struct ParsedLtb {
	header: LtbHeader,
	wflz_data: Vec<ParseResult<Box<[u8]>>>,
}

impl ParsedLtb {
	pub fn test(&self) -> String {
		format!("{:?}", self.header)
	}
	
	pub fn wflz_data_iter(&self) -> impl Iterator<Item = &ParseResult<Box<[u8]>>> {
		self.wflz_data.iter()
	}
}
