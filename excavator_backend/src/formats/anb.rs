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
	kind: u32,
	wflz_file_offset: Option<u64>,
	children: Vec<ParseResult<Self>>,
}

impl ParsedAnbNode {
	fn recursive_read<R: BufRead + Seek>(reader: &mut ParseReader<R>, offset: u64) -> ParseResult<Self> {
		let raw_node = reader.read_struct::<AnbTreeNode>(offset)?;
		let kind = raw_node.kind.get();
		let wflz_file_offset = None; // one sec
		
		let children_pointers = reader
			.read_struct_array::<U64<LE>>(raw_node.children_pointer.get(), raw_node.children_count.get().into())?
			.collect::<Result<Vec<_>, _>>()?;
		let children = children_pointers.iter().map(|offset| {
			Self::recursive_read(reader, offset.get())
		}).collect::<Vec<_>>();
		
		Ok(Self { kind, wflz_file_offset, children })
	}
	
	pub fn kind(&self) -> u32 {
		self.kind
	}
	
	pub fn children(&self) -> impl Iterator<Item = &ParseResult<Self>> {
		self.children.iter()
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

#[derive(Debug)]
struct AnbDataBlock {
	magic: [u8; 4], // always FF FF FF 00
	length: U32<LE>,
}
