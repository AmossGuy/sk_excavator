//! General functions used for parsing binary formats.

use std::io::{BufRead, Read, Seek};

/// Struct parser thingy. Takes a function that must read the exact number of bytes passed in.
pub fn parse_struct<F, T>(data: &[u8], func: F) -> Result<T, StructParserError>
where
F: FnOnce(&mut StructParser) -> Result<T, StructParserError>,
{
	let mut parser = StructParser::new(data);
	let result = func(&mut parser)?;
	parser.finish()?;
	Ok(result)
}

pub struct StructParser<'a> {
	data: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StructParserError {
	/// The end of the data was reached too early.
	UnexpectedEnd,
	/// There was still some data left when `finish` was called.
	LeftoverData,
}

macro_rules! read_number_impl {
	($method:ident, $type:ty) => {
		pub fn $method(&mut self) -> Result<$type, StructParserError> {
			const SIZE: usize = std::mem::size_of::<$type>();
			let bytes = self.read_bytes::<SIZE>()?;
			Ok(<$type>::from_le_bytes(bytes))
		}
	}
}

impl<'a> StructParser<'a> {
	fn new(data: &'a [u8]) -> Self {
		Self { data }
	}
		
	fn finish(self) -> Result<(), StructParserError> {
		if self.data.len() == 0 {
			Ok(())
		} else {
			Err(StructParserError::LeftoverData)
		}
	}
	
	pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], StructParserError> {
		// check that there are enough bytes
		if self.data.len() < N {
			return Err(StructParserError::UnexpectedEnd);
		}
		// now we know unwrapping these won't fail
		let bytes = self.data.first_chunk::<N>().unwrap(); // get the bytes we're reading
		self.data = self.data.get(N..).unwrap(); // remove the bytes we're reading from the data
		Ok(*bytes)
	}
	
	read_number_impl!(read_u32, u32);
	read_number_impl!(read_u64, u64);
}

// this is just an experiment of the error type conversion
/*
pub enum ReaderError {
	IoError(std::io::Error),
	UnexpectedValue,
	PointerOutOfRange,
}

pub type ReaderResult<T> = Result<T, ReaderError>;

impl From<std::io::Error> for ReaderError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

fn test<R: BufRead + Seek>(reader: &mut R) -> ReaderResult<()> {
	reader.seek_relative(10)?;
	Ok(())
}
*/

/// Using `BufReader::seek` always discards the internal buffer, even if the seek position is within it.
/// This function wraps `BufReader::seek_relative`, so the buffer is used if applicable.
pub fn seek_absolute<R: BufRead + Seek>(reader: &mut R, position: u64) -> std::io::Result<()> {
	let offset = position as i64 - reader.stream_position()? as i64;
	reader.seek_relative(offset)?;
	Ok(())
}

pub fn read_pile_o_pointers<R: Read>(reader: &mut R, count: usize) -> std::io::Result<Vec<u64>> {
	const SIZE: usize = size_of::<u64>();
	let mut buf = vec![0u8; SIZE * count];
	reader.read_exact(&mut buf)?;
	let mut result = Vec::with_capacity(count);
	for i in 0..count {
		let slice: [u8; SIZE] = buf[i*SIZE..(i+1)*SIZE].try_into().unwrap();
		result.push(u64::from_le_bytes(slice));
	}
	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn parse_nothing_struct() {
		let result = parse_struct(&Vec::new(), |_parser| Ok("return value"));
		assert_eq!(result, Ok("return value"));
	}
	
	#[test]
	fn parse_number_struct() {
		let data: Vec<u8> = vec![0x44, 0x33, 0x22, 0x11, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11];
		let result = parse_struct(&data, |parser| {
			let n = parser.read_u32()?;
			let m = parser.read_u64()?;
			Ok((n, m))
		});
		assert_eq!(result, Ok((0x11223344, 0x1122334455667788)));
	}
	
	#[test]
	fn parse_error_unexpected_end() {
		let data: [u8; 8] = *b"whatever";
		let result = parse_struct(&data, |parser| {
			let bytes = parser.read_bytes::<1000>()?;
			Ok(bytes)
		});
		assert_eq!(result, Err(StructParserError::UnexpectedEnd));
	}
	
	#[test]
	fn parse_error_leftover_data() {
		let data = [0x11u8; 30];
		let result = parse_struct(&data, |parser| {
			let n = parser.read_u32()?;
			let m = parser.read_u64()?;
			Ok((n, m))
		});
		assert_eq!(result, Err(StructParserError::LeftoverData));
	}
}
