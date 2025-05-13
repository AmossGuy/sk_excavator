//! General functions used for parsing binary formats.

use std::io::{BufRead, Read, Seek, SeekFrom};

// The standard library's version of this function isn't stabilized yet, so I just committed lazy and copy-pasted it
// (source: core/num/uint_macros.rs)
// TODO: Use standard library version when stabilized, copying it is silly
const fn u64_checked_signed_diff(lhs: u64, rhs: u64) -> Option<i64> {
	let res = lhs.wrapping_sub(rhs) as i64;
	let overflow = (lhs >= rhs) == (res < 0);
	
	if !overflow {
		Some(res)
	} else {
		None
	}
}

/// Using `BufReader::seek` always discards the internal buffer, even if the seek position is within it.
/// This function wraps `BufReader::seek_relative`, so the buffer is used if applicable.
pub fn seek_absolute<R: BufRead + Seek>(reader: &mut R, position: u64) -> std::io::Result<()> {
	if let Some(offset) = u64_checked_signed_diff(position, reader.stream_position()?) {
		reader.seek_relative(offset)?;
	} else {
		reader.seek(SeekFrom::Start(position))?;
	}
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
