use std::collections::HashMap;
use std::io::{BufRead, Seek, SeekFrom, Write};

use binrw::{BinRead, BinResult, BinWrite, Endian, NullString, VecArgs};
use serde::{Serialize, Deserialize};

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StlHeader {
	entry_count: u32,
	field_count: u32,
	data_pointer: u64,
}

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StbOrStmHeader {
	entry_count: u32,
	field_count: u32,
	checksums_pointer: u64,
	data_pointer: u64,
	// more stuff?
}

#[derive(Copy, Clone, Debug)]
struct StHeaderCommon {
	entry_count: u32,
	field_count: u32,
	data_pointer: u64,
}

impl From<StlHeader> for StHeaderCommon {
	fn from(value: StlHeader) -> Self {
		Self {
			entry_count: value.entry_count,
			field_count: value.field_count,
			data_pointer: value.data_pointer,
		}
	}
}

impl From<StbOrStmHeader> for StHeaderCommon {
	fn from(value: StbOrStmHeader) -> Self {
		Self {
			entry_count: value.entry_count,
			field_count: value.field_count,
			data_pointer: value.data_pointer,
		}
	}
}

pub fn read_st_wip<R: BufRead + Seek>(reader: &mut R, stl: bool) -> BinResult<()> {
	reader.rewind()?;
	let (header, header_full) = if stl {
		let h = StlHeader::read(reader)?;
		(StHeaderCommon::from(h), None)
	} else {
		let h = StbOrStmHeader::read(reader)?;
		(StHeaderCommon::from(h), Some(h))
	};
	
	// TODO: checked conversion and multiplication
	let entry_count = header.entry_count as usize;
	let field_count = header.field_count as usize;
	let raw_count = entry_count * field_count;
	
	reader.seek(SeekFrom::Start(header.data_pointer))?;
	let string_pointers = Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count: raw_count,
			inner: <_>::default(),
		},
	)?;
	let strings: Vec<String> = string_pointers.iter().map(|pointer| -> BinResult<_> {
		reader.seek(SeekFrom::Start(*pointer))?;
		Ok(NullString::read_le(reader)?.to_string())
	}).collect::<BinResult<_>>()?;
	
	let mut pointers2 = None;
	if let Some(header_full) = header_full {
		reader.seek(SeekFrom::Start(header_full.checksums_pointer))?;
		pointers2 = Some(Vec::<u32>::read_options(
			reader,
			Endian::Little,
			VecArgs {
				count: raw_count,
				inner: <_>::default(),
			},
		)?);
	}
	let pointers2 = pointers2;
	
	for i in raw_count.saturating_sub(20)..raw_count {
		let string = &strings[i];
		if let Some(p2) = &pointers2 {
			let their_checksum = p2[i];
			println!("{:?} {:08X?}", string, their_checksum);
		} else {
			println!("{:?}", string);
		}
	}
	
	//println!("{:?}", idk);
	//todo!("we've gotta show some entries and those possibly-checksums side by side, i guess");
	Ok(())
}
