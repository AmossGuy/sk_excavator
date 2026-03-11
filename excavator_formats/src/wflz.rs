use zerocopy::{*, LittleEndian as LE};
use crate::util_binary::{ParserReflect, ParserReflectContext, ParserStruct};

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

impl ParserReflect for WflzBlock {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		let self_offset = std::ptr::from_ref(self).addr() - context.file().as_ptr().addr();
		let after_offset = self_offset + std::mem::size_of::<Self>();
		
		let literals = ParserStruct::<[u8]>::new(context.file(), after_offset).retrieve_with_len(self.literals_length.into());
		context.bullshit(literals);
		
		// todo: get next block if there is one
	}
}

/*
struct SliceThing<'a>(&'a [u8], usize);

impl<'a> ParserReflect for SliceThing<'a> {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {}
}

impl<'a> std::fmt::Debug for SliceThing<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		self.0.fmt(f)
	}
}
*/
