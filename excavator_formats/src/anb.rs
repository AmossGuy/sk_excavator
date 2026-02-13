use zerocopy::{*, byteorder::{LittleEndian, U32, U64}};

use crate::util_binary::{ParserStruct, ParserStructError};
use crate::wflz::WflzHeader;

type LE = LittleEndian;

const ANB_MAGIC: [u8; 4] = *b"YCSN";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct AnbHeader {
	pub magic: [u8; 4],
	unknown: [U32<LE>; 17],
	pub data_pointer: U64<LE>,
}

impl AnbHeader {
	pub fn is_magic_correct(&self) -> bool {
		self.magic == ANB_MAGIC
	}
	
	pub fn get_subordinate_data<'a>(&self, file: &'a[u8]) -> Result<ParserStruct<'a, AnbDataStart>, ParserStructError> {
		// move this logic to a pointer newtype i think
		let offset: usize = self.data_pointer.get().try_into()
			.map_err(|_| ParserStructError::OutOfBounds)?;
		Ok(ParserStruct::<AnbDataStart>::new(file, offset))
	}
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct AnbDataStart {
	unknown: [U32<LE>; 6],
	pub wflz: WflzHeader,
}
