use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use pack1::{U32LE, U64LE};

use crate::util_binary::*;

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
struct PakHeader {
	idk: U32LE,
	file_count: U32LE,
	data_table_offset: U64LE,
	name_table_offset: U64LE,
}

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
struct PakFileHeader {
	file_size: U64LE,
	idk1: U64LE,
	idk2: U64LE,
	idk3: U64LE,
}

#[derive(Clone, Debug)]
pub struct PakIndex {
	pub files: Vec<(CString, PakIndexFileEntry)>,
}

#[derive(Clone, Debug)]
pub struct PakIndexFileEntry {
	pub data_start: u64,
	pub data_length: u64,
}

impl PakIndex {
	pub fn create_index<R: BufRead + Seek>(reader: &mut R) -> std::io::Result<Self> {
		reader.rewind()?;
		let mut header_buf = [0u8; size_of::<PakHeader>()];
		reader.read_exact(&mut header_buf)?;
		let header: PakHeader = bytemuck::cast(header_buf);
		
		let file_count_usize = header.file_count.get() as usize;
		reader.seek(SeekFrom::Start(header.data_table_offset.get()))?;
		let data_pointers = read_pile_o_pointers(reader, file_count_usize)?;
		reader.seek(SeekFrom::Start(header.name_table_offset.get()))?;
		let name_pointers = read_pile_o_pointers(reader, file_count_usize)?;
		
		let mut file_names = Vec::<CString>::with_capacity(file_count_usize);
		for i in 0..file_count_usize {
			seek_absolute(reader, name_pointers[i])?;
			let mut name_buf = Vec::<u8>::new();
			reader.read_until(0, &mut name_buf)?;
			file_names.push(CString::from_vec_with_nul(name_buf).unwrap());
		}
		
		let mut entries = Vec::<PakIndexFileEntry>::with_capacity(file_count_usize);
		for i in 0..file_count_usize {
			seek_absolute(reader, data_pointers[i])?;
			let mut file_header_buf = [0u8; size_of::<PakFileHeader>()];
			reader.read_exact(&mut file_header_buf)?;
			let file_header: PakFileHeader = bytemuck::cast(file_header_buf);
			
			let data_start = reader.stream_position()?;
			let data_length = file_header.file_size.get();
			entries.push(PakIndexFileEntry { data_start, data_length });
		}
		
		Ok(Self {
			files: file_names.into_iter().zip(entries).collect(),
		})
	}
}

pub fn read_whole_file<R: BufRead + Seek>(file_entry: &PakIndexFileEntry, reader: &mut R) -> std::io::Result<Vec<u8>> {
	reader.seek(SeekFrom::Start(file_entry.data_start))?;
	let mut data_buf = vec![0u8; file_entry.data_length as usize];
	reader.read_exact(&mut data_buf)?;
	Ok(data_buf)
}
