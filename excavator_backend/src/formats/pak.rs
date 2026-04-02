use crate::parse::{ParseLogger, ParseReader, ParseError, ParseResult};
use bstr::BString;
use std::io::{BufRead, Seek};
use zerocopy::byteorder::{LittleEndian as LE, U32, U64};
use zerocopy_derive::*;

pub struct PakParser<R: BufRead + Seek, L: ParseLogger<R> = ()> {
	reader: ParseReader<R, L>,
	header: PakHeader,
}

impl<R: BufRead + Seek, L: ParseLogger<R>> PakParser<R, L> {	
	pub fn new(reader: R, logger: L) -> ParseResult<Self> {
		let mut reader = ParseReader::new(reader, logger);
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
			.read_struct_array::<U64<LE>>(self.name_table_offset(), self.file_count().into())?
			.collect::<Result<Vec<_>, _>>()?;
		
		Ok(name_pointers.into_iter().zip(0u32..).map(|(pointer, i)| {
			let name = self.reader.read_null_terminated_string(pointer.get())?;
			Ok((i, name))
		}))
	}
	
	pub fn file_position_size(&mut self, index: u32) -> ParseResult<(u64, u64)> {
		let data_entry_offset = self.reader
			.read_struct_array::<U64<LE>>(self.data_table_offset(), self.file_count().into())?
			.nth_u64(index.into()).ok_or(ParseError)??.get();
		let mut cursor = self.reader.cursor(data_entry_offset)?;
		let entry_header = cursor.read_struct::<PakEntryHeader>()?;
		Ok((cursor.stream_position()?, entry_header.file_size.get()))
	}
	
	pub fn parse_all(&mut self) {
		let files: Result<Vec<_>, _> = self.files().map(|iter| iter.collect());
		if let Ok(files) = files {
			for result in files {
				if let Ok((i, _)) = result {
					let _ = self.file_position_size(i);
				}
			}
		}
	}
	
	pub fn collect_log(self) -> L::Out {
		self.reader.collect_log()
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
