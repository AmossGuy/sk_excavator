use std::{error, fmt, io};
use std::io::Read;
use zerocopy::{FromBytes, IntoBytes, LittleEndian as LE, U16, U32};
use zerocopy_derive::*;

pub const WFLZ_MAGIC: [u8; 4] = *b"WFLZ";

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct WflzHeader {
	magic: [u8; 4],
	compressed_size: U32<LE>,
	decompressed_size: U32<LE>,
	// Since this first block is part of the header, it does not count towards compressed_size.
	// The literal bytes specified by it, however, do count.
	// This means that compressed_size is always exactly 4 less than you might expect.
	first_block: WflzBlock,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct WflzBlock {
	backref_dist: U16<LE>,
	backref_length: u8,
	literals_length: u8,
}

impl WflzBlock {
	fn is_terminator(&self) -> bool {
		self.as_bytes().iter().all(|&x| x == 0)
	}
}

#[derive(Debug)]
pub enum WflzReadError {
	Io(io::Error),
	WrongMagic,
	InvalidBackref,
	BiggerThanExpected,
}

impl fmt::Display for WflzReadError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(e) => e.fmt(f),
			Self::WrongMagic => write!(f, "wrong wflz magic"),
			Self::InvalidBackref => write!(f, "invalid wflz backref"),
			Self::BiggerThanExpected => write!(f, "wflz decompression result too big"),
		}
	}
}

impl error::Error for WflzReadError {}

impl From<io::Error> for WflzReadError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

pub fn decompress<R: Read>(reader: &mut R) -> Result<Box<[u8]>, WflzReadError> {
	let mut wflz = WflzReader::new(reader)?;
	wflz.decompress_all()?;
	Ok(wflz.data)
}

struct WflzReader<R: Read> {
	reader: R,
	data: Box<[u8]>,
	write_index: usize,
}

impl<R: Read> WflzReader<R> {
	fn new(mut reader: R) -> Result<Self, WflzReadError> {
		let header = WflzHeader::read_from_io(&mut reader)?;
		let WflzHeader { magic, compressed_size, decompressed_size, first_block } = header;
		let (_compressed_size, decompressed_size) = (compressed_size.get(), decompressed_size.get());
		
		if magic != WFLZ_MAGIC {
			return Err(WflzReadError::WrongMagic);
		} else if first_block.backref_dist != 0 || first_block.backref_length != 0 {
			return Err(WflzReadError::InvalidBackref);
		}
		
		let decompressed_size_usize = usize::try_from(decompressed_size).unwrap_or(usize::MAX);
		let mut this = Self {
			reader,
			data: vec![0; decompressed_size_usize].into_boxed_slice(),
			write_index: 0,
		};
		
		this.read_literals(first_block.literals_length)?;
		
		Ok(this)
	}
	
	fn decompress_all(&mut self) -> Result<(), WflzReadError> {
		loop {
			let block = self.read_block()?;
			if block.is_terminator() {
				break;
			}
			self.read_backref(block.backref_dist.get(), block.backref_length)?;
			self.read_literals(block.literals_length)?;
		}
		Ok(())
	}
	
	fn read_block(&mut self) -> Result<WflzBlock, WflzReadError> {
		let block = WflzBlock::read_from_io(&mut self.reader)?;
		Ok(block)
	}
	
	fn read_backref(&mut self, dist: u16, length: u8) -> Result<(), WflzReadError> {
		let (dist, length) = (usize::from(dist), usize::from(length));
		let length = length + std::mem::size_of::<WflzBlock>();
		
		let _ = self.data.get(self.write_index..).ok_or(WflzReadError::BiggerThanExpected)?;
		
		let offset = self.write_index.checked_sub(dist)
			.ok_or(WflzReadError::InvalidBackref)?;
		let slice = self.data.get_mut(offset..)
			.ok_or(WflzReadError::BiggerThanExpected)?;
		
		// The bounds checks inside the loop get optimized out thanks to the one at the top
		// Loop unrolling and autovectorization also kicks in, which seems good
		//
		// The arithmetic can't overflow since these numbers are converted from smaller types
		// (...unless you compile this with 16-bit usize for some reason)
		let _ = slice.get(dist + length - 1).ok_or(WflzReadError::BiggerThanExpected)?;
		for i in 0..length {
			slice[dist + i] = slice[i];
		}
		
		self.write_index += length;
		Ok(())
	}
	
	fn read_literals(&mut self, length: u8) -> Result<(), WflzReadError> {
		let length = usize::from(length);
		
		let buf = self.data.get_mut(self.write_index..)
			.and_then(|x| x.get_mut(..length))
			.ok_or(WflzReadError::BiggerThanExpected)?;
		
		self.reader.read_exact(buf)?;
		
		self.write_index += length;
		Ok(())
	}
}
