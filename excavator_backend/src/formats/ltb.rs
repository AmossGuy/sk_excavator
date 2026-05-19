mod parse;
mod raw;

pub use self::parse::*;
use self::raw::*;

use crate::parse::ParseReader;

use std::io::{BufRead, Seek};
use zerocopy::FromBytes;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_SIZE.pow(2);

fn read_struct_array_from_row<T: FromBytes>(reader: &mut ParseReader<impl BufRead + Seek>, row: &LtbHeaderRow) -> anyhow::Result<Vec<T>> {
	read_struct_array_from_row_2(reader, row, false).map(|(vec, _)| vec)
}
	
fn read_struct_array_from_row_2<T: FromBytes>(reader: &mut ParseReader<impl BufRead + Seek>, row: &LtbHeaderRow, lenient: bool) -> anyhow::Result<(Vec<T>, u32)> {
	let unknown_count_thing = row.unknown.get();
	let array_count = row.entry_count.get();
	let array_pointer = row.entry_pointer.get();
	
	if !lenient && unknown_count_thing != array_count {
		anyhow::bail!("count mismatch?");
	}
	
	let vec = reader
		.read_struct_array::<T>(array_pointer, array_count.into())?
		.collect::<Result<Vec<_>, _>>()?;
	Ok((vec, unknown_count_thing))
}
