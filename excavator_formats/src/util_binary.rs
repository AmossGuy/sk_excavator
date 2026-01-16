//! General functions used for parsing binary formats.

use std::io::{BufRead, Read, Seek, SeekFrom};

use binrw::{BinRead, BinResult, Endian, VecArgs};

/// Using `BufReader::seek` always discards the internal buffer, even if the seek position is within it.
/// This function wraps `BufReader::seek_relative`, so the buffer is used if applicable.
pub fn seek_absolute<R: BufRead + Seek>(reader: &mut R, position: u64) -> std::io::Result<()> {
	if let Some(offset) = position.checked_signed_diff(reader.stream_position()?) {
		reader.seek_relative(offset)?;
	} else {
		reader.seek(SeekFrom::Start(position))?;
	}
	Ok(())
}

pub fn read_pointers<R: Read + Seek>(reader: &mut R, count: usize) -> BinResult<Vec<u64>> {
	Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count,
			inner: <_>::default(),
		},
	)
}
