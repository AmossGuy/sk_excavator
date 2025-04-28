use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};

use crate::util_binary::{parse_struct, read_pile_o_pointers, seek_absolute};

#[derive(Copy, Clone, Debug)]
struct StlHeader {
	string_count: u32,
	string_table_offset: u64,
}

pub fn read_stl<R: BufRead + Seek>(reader: &mut R) -> anyhow::Result<Vec<String>> {
	reader.rewind()?;
	let mut header_buf = [0u8; 24];
	reader.read_exact(&mut header_buf)?;
	let header: StlHeader = parse_struct(&header_buf, |parser| {
		parser.expect_u32(0)?;
		parser.expect_u32(0)?;
		let string_count = parser.read_u32()?;
		parser.expect_u32(1)?;
		let string_table_offset = parser.read_u64()?;
		Ok(StlHeader { string_count, string_table_offset })
	})?;
	
	reader.seek(SeekFrom::Start(header.string_table_offset as u64))?;
	let text_pointers = read_pile_o_pointers(reader, header.string_count as usize)?; // code smell much?
	
	let mut strings = Vec::<String>::with_capacity(header.string_count as usize);
	for i in 0..header.string_count as usize {
		seek_absolute(reader, text_pointers[i])?;
		let mut string_buf = Vec::<u8>::new();
		reader.read_until(0, &mut string_buf)?;
		strings.push(CString::from_vec_with_nul(string_buf).unwrap().into_string().unwrap());
	}
	Ok(strings)
}
