use crate::parse::{ParseError, ParseReader, ParseResult};
use bstr::BString;
use zerocopy::{*, byteorder::{LittleEndian as LE, U32, U64}};

pub struct PakParser {
	reader: ParseReader,
	header: PakHeader,
}

impl PakParser {
	pub fn new(bytes: crate::io::FileBytes) -> ParseResult<Self> {
		let mut reader = ParseReader::new(bytes, ());
		let header = reader.read_struct::<PakHeader>(0)?;
		Ok(Self { reader, header })
	}
	
	pub fn file_count(&self) -> u32 {
		self.header.file_count.get()
	}
	
	fn data_table_offset(&self) -> u64 {
		self.header.data_table_offset.get()
	}
	
	fn name_table_offset(&self) -> u64 {
		self.header.name_table_offset.get()
	}
	
	pub fn files(&mut self) -> ParseResult<impl Iterator<Item = ParseResult<(u32, BString)>>> {
		let name_pointers = self.reader
			.read_struct_array::<U64<LE>>(self.name_table_offset(), self.file_count())?
			.collect::<Result<Vec<_>, _>>()?;
		
		Ok(name_pointers.into_iter().zip(0u32..).map(|(pointer, i)| {
			let name = self.reader.read_null_terminated_string(pointer.get())?;
			Ok((i, name))
		}))
	}
	
	pub fn archived_file_by_index(&mut self, index: u32) -> ParseResult<crate::io::FileBytes> {
		let file_pointer = self.reader
			.read_struct_array::<U64<LE>>(self.data_table_offset(), self.file_count())?
			.nth(index as usize).ok_or(ParseError)??;
		
		let (file_header, continued) = self.reader.read_struct_continued::<PakEntryHeader>(file_pointer.get())?;
		continued.archived_file(file_header.file_size.get())
	}
}

/// The header at the beginning of a .pak archive.
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct PakHeader {
	/// Always zeros?
	magic: [u8; 4],
	/// The number of files in the archive.
	file_count: U32<LE>,
	/// Pointer to a table of pointers to each files' header.
	data_table_offset: U64<LE>,
	/// Pointer to a table of pointers to the null-terminated filenames.
	name_table_offset: U64<LE>,
}

/// The header for a single file. Immediately precedes the contents of the file.
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct PakEntryHeader {
	file_size: U64<LE>,
	idk1: U64<LE>,
	idk2: U64<LE>,
	idk3: U64<LE>,
}
