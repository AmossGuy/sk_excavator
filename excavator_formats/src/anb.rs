use zerocopy::{*, byteorder::{LittleEndian, U32, U64}};

use crate::util_binary::{ParserReflect, ParserReflectContext};
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
}

impl ParserReflect for AnbHeader {
	fn get_subordinates(&self, context: &mut ParserReflectContext) {
		context.follow_pointer::<AnbDataStart>(self.data_pointer.get() as usize);
	}
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
