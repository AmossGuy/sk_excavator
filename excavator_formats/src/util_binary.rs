//! General functions used for parsing binary formats.

use std::io::{BufRead, Read, Seek, SeekFrom};
use std::marker::PhantomData;

use binrw::{BinRead, BinResult, Endian, VecArgs};

use zerocopy::*;

/// Using `BufReader::seek` always discards the internal buffer, even if the seek position is within it.
/// This function wraps `BufReader::seek_relative`, so the buffer is used if applicable.
pub(crate) fn seek_absolute<R: BufRead + Seek>(reader: &mut R, position: u64) -> std::io::Result<()> {
	if let Some(offset) = position.checked_signed_diff(reader.stream_position()?) {
		reader.seek_relative(offset)?;
	} else {
		reader.seek(SeekFrom::Start(position))?;
	}
	Ok(())
}

pub(crate) fn read_pointers<R: Read + Seek>(reader: &mut R, count: usize) -> BinResult<Vec<u64>> {
	Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count,
			inner: <_>::default(),
		},
	)
}

#[derive(Debug)]
pub enum ParserStructError {
	OutOfBounds,
	CastError(String),
}

pub struct ParserStruct<'a, T: FromBytes + KnownLayout + Immutable> {
	file: &'a [u8],
	offset: usize,
	phantom: PhantomData<&'a T>,
}

impl<'a, T: FromBytes + KnownLayout + Immutable> ParserStruct<'a, T> {
	pub fn new(file: &'a [u8], offset: usize) -> Self {
		Self { file, offset, phantom: PhantomData }
	}
	
	pub fn retrieve(&self) -> Result<&T, ParserStructError> {
		let slice = self.file.get(self.offset..)
			.ok_or(ParserStructError::OutOfBounds)?;
		let (retrieved, _) = T::ref_from_prefix(slice)
			.map_err(|e| ParserStructError::CastError(e.to_string()))?;
		Ok(retrieved)
	}
}
