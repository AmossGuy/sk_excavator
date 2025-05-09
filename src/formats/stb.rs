use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, Endian, NullString, VecArgs};

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StbHeader {
	entry_count: u32,
	field_count: u32,
	something_pointer: u64, // look at this more later
	field_data_pointer: u64,
	// more stuff?
}

pub fn read_stb<R: BufRead + Seek>(reader: &mut R) -> BinResult<(u32, Vec<String>)> {
	reader.rewind()?;
	let header = StbHeader::read(reader)?;
	
	reader.seek(SeekFrom::Start(header.field_data_pointer))?;
	let string_pointers = Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count: (header.entry_count * header.field_count) as usize,
			inner: <_>::default(),
		},
	)?;
	let strings: BinResult<Vec<_>> = string_pointers.iter().map(|pointer| -> BinResult<_> {
		reader.seek(SeekFrom::Start(*pointer))?;
		Ok(NullString::read_le(reader)?.to_string())
	}).collect();
	
	Ok((header.field_count, strings?))
}
