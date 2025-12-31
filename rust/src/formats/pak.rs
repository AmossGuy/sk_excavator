use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite};

use super::util_binary::{read_pointers, seek_absolute};

/// The header at the beginning of a `.bin` archive.
#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0")]
struct PakHeader {
	/// The number of files in the archive.
	file_count: u32,
	/// Pointer to a table of pointers to each files' header.
	data_table_offset: u64,
	/// Pointer to a table of pointers to the null-terminated filenames.
	name_table_offset: u64,
}

/// The header for a single file. Immediately precedes the contents of the file.
#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little)]
struct PakFileHeader {
	file_size: u64,
	idk1: u64,
	idk2: u64,
	idk3: u64,
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
	pub fn create_index<R: BufRead + Seek>(reader: &mut R) -> BinResult<Self> {
		reader.rewind()?;
		let header = PakHeader::read(reader)?;
		
		let file_count_usize = header.file_count as usize;
		reader.seek(SeekFrom::Start(header.data_table_offset))?;
		let data_pointers = read_pointers(reader, file_count_usize)?;
		reader.seek(SeekFrom::Start(header.name_table_offset))?;
		let name_pointers = read_pointers(reader, file_count_usize)?;
		
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
			let file_header = PakFileHeader::read(reader)?;
			
			let data_start = reader.stream_position()?;
			let data_length = file_header.file_size;
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
