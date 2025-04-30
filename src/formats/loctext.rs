use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite};

use super::util_binary::{read_pile_o_pointers, seek_absolute};

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StlHeader {
	string_count: u32,
	language_variants: u32, // One for all languages except Japanese, which has two
	string_table_offset: u64,
}

pub fn read_stl<R: BufRead + Seek>(reader: &mut R) -> BinResult<Vec<String>> {
	reader.rewind()?;
	let header = StlHeader::read(reader)?;
	let string_count_adjusted = (header.string_count * header.language_variants) as usize;
	
	reader.seek(SeekFrom::Start(header.string_table_offset as u64))?;
	let text_pointers = read_pile_o_pointers(reader, string_count_adjusted)?;
	
	let mut strings = Vec::<String>::with_capacity(string_count_adjusted);
	for i in 0..string_count_adjusted {
		seek_absolute(reader, text_pointers[i])?;
		let mut string_buf = Vec::<u8>::new();
		reader.read_until(0, &mut string_buf)?;
		strings.push(CString::from_vec_with_nul(string_buf).unwrap().into_string().unwrap());
	}
	Ok(strings)
}
