use zerocopy::{*, byteorder::{LittleEndian, U32}};

type LE = LittleEndian;

const WFLZ_MAGIC: [u8; 4] = *b"WFLZ";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct WflzHeader {
	pub magic: [u8; 4],
	compressed_size: U32<LE>,
	decompressed_size: U32<LE>,
}

impl WflzHeader {
	pub fn is_magic_correct(&self) -> bool {
		self.magic == WFLZ_MAGIC
	}
}
