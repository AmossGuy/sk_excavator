use crate::parse::*;
use crate::formats::anb::decompress_wflz;
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
	is_compressed: U32<LE>,
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
	pub fn decompress(&self) -> anyhow::Result<Box<[u8]>> {
		match self.meta.is_compressed.get() {
			0 => Ok(self.data.clone()),
			1 => decompress_wflz(&mut std::io::Cursor::new(&self.data)).map_err(|e| e.into()),
			x => Err(anyhow::anyhow!("Invalid value for compression toggle: {}", x)),
		}
	}
	
	pub fn size(&self) -> [u32; 2] {
		[self.meta.image_width.get(), self.meta.image_height.get()]
	}
	
	pub fn paletted(&self) -> bool {
		// Is this really how this is determined?
		self.meta.unknown_more[2] != u32::MAX
	}
	
	pub fn meta_debug(&self) -> String {
		let meta = &self.meta;
		// let more_count = meta.unknown_more.iter().filter(|x| x.get() != u32::MAX).count();
		format!(
			"unknown a: {}, compressed: {}, width: {}, height: {}, unknown 1: 0x{:X}, unknown 2: {}, unknown 3: 0x{:X}, unknown last: {}",
			meta.unknown_00, meta.is_compressed, meta.image_width, meta.image_height,
			meta.unknown_more[0], meta.unknown_more[1], meta.unknown_more[2], meta.unknown_more[13],
		)
	}
}
