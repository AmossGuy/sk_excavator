//! General functions used for parsing binary formats.

use std::io::{BufRead, Read, Seek};

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
