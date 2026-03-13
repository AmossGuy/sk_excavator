use zerocopy::{*, LittleEndian as LE};
use crate::util_binary::{ParserReflect, ParserReflectContext, ParserStruct, ParserStructError, StructRole};

const WFLZ_MAGIC: [u8; 4] = *b"WFLZ";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct WflzHeader {
	pub magic: [u8; 4],
	pub compressed_size: U32<LE>,
	pub decompressed_size: U32<LE>,
}

impl WflzHeader {
	pub fn is_magic_correct(&self) -> bool {
		self.magic == WFLZ_MAGIC
	}
	
	pub fn first_block<'a>(&self, file: &'a [u8]) -> ParserStruct<'a, WflzBlock> {
		let self_offset = std::ptr::from_ref(self).addr() - file.as_ptr().addr();
		let after_offset = self_offset + std::mem::size_of::<Self>();
		ParserStruct::new(file, after_offset)
	}
}

impl ParserReflect for WflzHeader {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.ingest(self.first_block(context.file()));
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct WflzBlock {
	backref_dist: U16<LE>,
	backref_length: u8,
	literals_length: u8,
}

impl WflzBlock {
	// Gotta make a ParserSlice type or something
	pub fn literals<'a>(&self, file: &'a [u8]) -> Result<&'a [u8], ParserStructError> {
		let self_offset = std::ptr::from_ref(self).addr() - file.as_ptr().addr();
		let literals_offset = self_offset + std::mem::size_of::<Self>();
		
		ParserStruct::<[u8]>::new(file, literals_offset).retrieve_with_len(usize::from(self.literals_length))
	}
	
	pub fn next_block<'a>(&self, file: &'a [u8]) -> Option<ParserStruct<'a, WflzBlock>> {
		if self.backref_dist == 0 && self.backref_length == 0 && self.literals_length == 0 {
			return None;
		}
		
		let self_offset = std::ptr::from_ref(self).addr() - file.as_ptr().addr();
		let next_offset = self_offset + std::mem::size_of::<Self>() + usize::from(self.literals_length);
		
		Some(ParserStruct::new(file, next_offset))
	}
}

impl ParserReflect for WflzBlock {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		let file = context.file();
		
		context.bullshit(self.literals(file));
		
		if let Some(next_block) = self.next_block(file) {
			context.ingest(next_block);
		}
	}
	
	fn role(&self) -> StructRole {
		StructRole::CompressionBlock
	}
}

pub struct WflzDecompressor<'a> {
	file: &'a [u8],
	cursor: usize,
	decompressed_data: Vec<u8>,
}

impl<'a> WflzDecompressor<'a> {
	pub fn new(file: &'a [u8], header_offset: usize) -> Result<Self, ParserStructError> {
		let header = ParserStruct::<WflzHeader>::new(file, header_offset).retrieve()?;
		
		let cursor = header.first_block(file).get_offset();
		let decompressed_data = Vec::with_capacity(header.decompressed_size.get() as usize);
		Ok(Self { file, cursor, decompressed_data })
	}
	
	fn decompress_block(&mut self, block: &'a WflzBlock) -> Result<Option<ParserStruct<'a, WflzBlock>>, ParserStructError> {
		//
		let backref_slice_start = self.decompressed_data.len().saturating_sub(usize::from(block.backref_dist));
		let backref_slice_end = std::cmp::min(
			backref_slice_start.saturating_add(usize::from(block.backref_length)),
			self.decompressed_data.len()
		);
		self.decompressed_data.extend_from_within(backref_slice_start..backref_slice_end);
		
		let literals_slice = block.literals(self.file)?;
		self.decompressed_data.extend(literals_slice);
		
		Ok(block.next_block(self.file))
	}
	
	pub fn decompress_all(mut self) -> Result<Box<[u8]>, ParserStructError> {
		loop {
			let current_block = ParserStruct::<WflzBlock>::new(self.file, self.cursor).retrieve()?;
			if let Some(next_block) = self.decompress_block(current_block)? {
				self.cursor = next_block.get_offset();
			} else {
				break;
			}
		}
		Ok(Box::from(self.decompressed_data))
	}
}
