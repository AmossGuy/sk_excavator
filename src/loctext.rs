use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};

use bytemuck::{Pod, Zeroable};
use pack1::U32LE;

use crate::util_binary::*;

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
struct StlHeader {
	idk: [u8; 8],
	string_count: U32LE,
	why_is_it_1: [u8; 4],
	table_offset: U32LE,
	who_knows: [u8; 4],
}

pub fn read_stl<R: BufRead + Seek>(reader: &mut R) -> std::io::Result<Vec<String>> {
	reader.rewind()?;
	let mut header_buf = [0u8; size_of::<StlHeader>()];
	reader.read_exact(&mut header_buf)?;
	let header: StlHeader = bytemuck::cast(header_buf);
	
	#[allow(non_snake_case)] // EMOTIONS (i'm up too late)
	let string_count_AAAAA = header.string_count.get() as usize;
	
	reader.seek(SeekFrom::Start(header.table_offset.get() as u64))?;
	let text_pointers = read_pile_o_pointers(reader, string_count_AAAAA)?; // code smell much?
	
	let mut strings = Vec::<String>::with_capacity(string_count_AAAAA);
	for i in 0..string_count_AAAAA as usize {
		seek_absolute(reader, text_pointers[i])?;
		let mut string_buf = Vec::<u8>::new();
		reader.read_until(0, &mut string_buf)?;
		strings.push(CString::from_vec_with_nul(string_buf).unwrap().into_string().unwrap());
	}
	Ok(strings)
}
