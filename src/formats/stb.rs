use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, Endian, NullString, VecArgs};

#[derive(Clone, Debug)]
pub struct StbReadData {
	entry_count: usize,
	field_count: usize,
	strings: Vec<String>,
	// mystery_data: Vec<u32>,
}

impl StbReadData {
	pub fn entry_count(&self) -> usize {
		self.entry_count
	}
	
	pub fn field_count(&self) -> usize {
		self.field_count
	}
	
	pub fn field_names(&self) -> &[String] {
		&self.strings[0..self.field_count]
	}
	
	pub fn entry(&self, index: usize) -> &[String] {
		let offset = (index + 1) * self.field_count;
		&self.strings[offset..offset + self.field_count]
	}
	
	/*
	pub fn entry_mystery_data(&self, index: usize) -> &u32 {
		&self.mystery_data[index]
	}
	*/
}

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StbHeader {
	entry_count: u32,
	field_count: u32,
	something_pointer: u64, // look at this more later
	field_data_pointer: u64,
	// more stuff?
}

pub fn read_stb_wip<R: BufRead + Seek>(reader: &mut R) -> BinResult<()> {
	reader.rewind()?;
	let header = StbHeader::read(reader)?;
	
	// TODO: checked conversion
	let entry_count = header.entry_count as usize;
	let field_count = header.field_count as usize;
	
	reader.seek(SeekFrom::Start(header.field_data_pointer))?;
	let string_pointers = Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count: entry_count * field_count, // TODO: checked multiplication
			inner: <_>::default(),
		},
	)?;
	let strings: Vec<String> = string_pointers.iter().map(|pointer| -> BinResult<_> {
		reader.seek(SeekFrom::Start(*pointer))?;
		Ok(NullString::read_le(reader)?.to_string())
	}).collect::<BinResult<_>>()?;
	
	let idk = StbReadData {
		entry_count, field_count, strings,
	};
	
	//println!("{:?}", idk);
	todo!("we've gotta show some entries and those possibly-checksums side by side, i guess");
	Ok(())
}
