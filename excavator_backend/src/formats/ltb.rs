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
	rows: [LtbHeaderRow; 8],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct LtbHeaderRow {
	unknown: U32<LE>, // usually the same as entry_count below. what does that mean?
	entry_count: U32<LE>,
	entry_pointer: U64<LE>,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct ImageMetadata {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	image_width: U32<LE>,
	image_height: U32<LE>,
	unknown_more: [U32<LE>; 14],
	data_size: U32<LE>,
}

pub fn parse_ltb<R: BufRead + Seek>(reader: &mut R) -> ParseResult<ParsedLtb> {
	let mut reader = ParseReader::new(reader);
	let header = reader.read_struct::<LtbHeader>(0)?;
	
	let image_pointers = read_struct_array_from_row::<U64<LE>>(&mut reader, &header.rows[7])?;
	let meta = read_struct_array_from_row::<ImageMetadata>(&mut reader, &header.rows[2])?;
	
	let images = std::iter::zip(image_pointers, meta).map(|(poin, meta)| {
		let mut cursor = reader.cursor(poin.get())?;
		let mut data = vec![0; meta.data_size.get() as usize].into_boxed_slice();
		cursor.inner_reader().read_exact(&mut data)?;
		Ok(ParsedImage { data, meta })
	}).collect::<Vec<_>>();
	
	Ok(ParsedLtb { header, images })
}

fn read_struct_array_from_row<T: FromBytes>(reader: &mut ParseReader<impl BufRead + Seek>, row: &LtbHeaderRow) -> ParseResult<Vec<T>> {
	let array_count = row.entry_count.get();
	let array_pointer = row.entry_pointer.get();
	
	let vec = reader
		.read_struct_array::<T>(array_pointer, array_count.into())?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(vec)
}

pub struct ParsedLtb {
	header: LtbHeader,
	images: Vec<ParseResult<ParsedImage>>,
}

impl ParsedLtb {
	pub fn test(&self) -> String {
		format!("{:?}", self.header)
	}
	
	pub fn images(&self) -> impl Iterator<Item = &ParseResult<ParsedImage>> {
		self.images.iter()
	}
}

pub struct ParsedImage {
	data: Box<[u8]>,
	meta: ImageMetadata,
}

impl ParsedImage {
	pub fn data(&self) -> &[u8] {
		&self.data
	}
	
	pub fn size(&self) -> [u32; 2] {
		[self.meta.image_width.get(), self.meta.image_height.get()]
	}
	
	pub fn meta_debug(&self) -> String {
		let meta = &self.meta;
		format!(
			"unknown a: {}, unknown b: {}, width: {}, height: {}",
			meta.unknown_00, meta.unknown_04, meta.image_width, meta.image_height,
		)
	}
}
