use crate::parse::*;
use std::io::{BufRead, Seek};
use zerocopy::{*, LittleEndian as LE};
use zerocopy_derive::*;

pub const ANB_MAGIC: [u8; 4] = *b"YCSN";

pub fn parse_anb<R: BufRead + Seek>(reader: &mut R) -> ParseResult<ParsedAnb> {
	let mut reader = ParseReader::new(reader);
	let header = reader.read_struct::<AnbHeader>(0)?;
	check_magic(ANB_MAGIC, header.magic)?;
	
	let root_pointer = reader.read_struct::<U64<LE>>(header.root_pointer.get())?.get();
	
	let root = ParsedAnbNode::recursive_read(&mut reader, root_pointer);
	Ok(ParsedAnb { root })
}

pub struct ParsedAnb {
	root: ParseResult<ParsedAnbNode>,
}

impl ParsedAnb {
	pub fn root(&self) -> &ParseResult<ParsedAnbNode> {
		&self.root
	}
}

pub struct ParsedAnbNode {
	data: ParsedData,
	children: Vec<ParseResult<Self>>,
}

impl ParsedAnbNode {
	fn recursive_read<R: BufRead + Seek>(reader: &mut ParseReader<R>, offset: u64) -> ParseResult<Self> {
		let mut cursor = reader.cursor(offset)?;
		
		let raw_node = cursor.read_struct::<AnbTreeNode>()?;
		let data = ParsedData::read(cursor, raw_node.kind.get())?;
		
		let children_pointers = reader
			.read_struct_array::<U64<LE>>(raw_node.children_pointer.get(), raw_node.children_count.get().into())?
			.collect::<Result<Vec<_>, _>>()?;
		let children = children_pointers.iter().map(|offset| {
			Self::recursive_read(reader, offset.get())
		}).collect::<Vec<_>>();
		
		Ok(Self { data, children })
	}
	
	pub fn kind(&self) -> u32 {
		self.data.kind()
	}
	
	pub fn data(&self) -> &ParsedData {
		&self.data
	}
	
	pub fn children(&self) -> impl Iterator<Item = &ParseResult<Self>> {
		self.children.iter()
	}
}

#[non_exhaustive]
pub enum ParsedData {
	FrameWflz { metadata: FrameWflzMetadata, data: Box<[u8]> },
	Unknown { kind: u32 },
}

// gotta encapsulate ParsedData more
pub use super::wflz::decompress as decompress_wflz;

impl ParsedData {
	fn kind(&self) -> u32 {
		match self {
			Self::FrameWflz { .. } => 1,
			Self::Unknown { kind } => *kind,
		}
	}
	
	fn read<R: BufRead + Seek>(mut cursor: ParseCursor<'_, R>, kind: u32) -> ParseResult<Self> {
		Ok(match kind {
			1 => {
				let attached = cursor.read_struct::<FrameWflzAttached>()?;
				
				let reader = cursor.uncursor();
				
				let cursor = reader.cursor(attached.wflz_pointer.get())?;
				let data = read_data_block(cursor)?;
				
				Self::FrameWflz { metadata: FrameWflzMetadata::from(&attached), data }
			},
			_ => Self::Unknown { kind },
		})
	}
}

fn read_data_block<R: BufRead + Seek>(mut cursor: ParseCursor<'_, R>) -> ParseResult<Box<[u8]>> {
	let block_header = cursor.read_struct::<AnbDataBlockHeader>()?;
	check_magic([0xFF, 0xFF, 0xFF, 0], block_header.magic)?;
	let mut data = vec![0; block_header.length.get() as usize].into_boxed_slice();
	cursor.inner_reader().read_exact(&mut data)?;
	Ok(data)
}

#[derive(Clone, Debug)]
pub struct FrameWflzMetadata {
	pub image_width: u32,
	pub image_height: u32,
	pub unknown_a: u32,
	pub unknown_b: u32,
}

impl From<&FrameWflzAttached> for FrameWflzMetadata {
	fn from(value: &FrameWflzAttached) -> Self {
		Self {
			image_width: value.image_width.get(),
			image_height: value.image_height.get(),
			unknown_a: value.unknown_08.get(),
			unknown_b: value.unknown_0C.get(),
		}
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct AnbHeader {
	magic: [u8; 4],
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	unknown_10: U32<LE>,
	unknown_14: U32<LE>,
	unknown_18: U32<LE>,
	unknown_1C: U32<LE>,
	root_pointer: U64<LE>,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct AnbTreeNode {
	kind: U32<LE>,
	children_count: U32<LE>,
	children_pointer: U64<LE>,
	// The actual information is stored directly after this struct, the format determined by the kind field
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct AnbDataBlockHeader {
	magic: [u8; 4], // always FF FF FF 00
	length: U32<LE>,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct FrameWflzAttached {
	image_width: U32<LE>,
	image_height: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	wflz_pointer: U64<LE>,
}
