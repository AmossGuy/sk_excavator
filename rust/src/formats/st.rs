use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, NullString};

use super::util_binary::read_pointers;

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StlHeader {
	entry_count: u32,
	field_count: u32,
	data_pointer: u64,
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StbOrStmHeader {
	entry_count: u32,
	field_count: u32,
	checksums_pointer: u64,
	data_pointer: u64,
	extra1: StbOrStmHeaderExtra,
	extra2: StbOrStmHeaderExtra,
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0")]
struct StbOrStmHeaderExtra {
	extra_entry_count: u32,
	pointer: u64,
}

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, repr = u64)]
enum MagicTen {
	HexTen = 0x10,
}

#[derive(BinRead, BinWrite, Clone, Debug)]
#[brw(little)]
#[expect(dead_code)] // we'll get back to this
struct StbOrStmDataExtra {
	piece_count: u64,
	magic: MagicTen,
	#[br(count = piece_count)]
	pieces: Vec<(u32, u32)>,
}

// TODO: stick in an Option field with the stb/stm exclusive data. or maybe our own enum
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

pub struct StReadOutcome {
	pub field_count: usize,
	pub strings: Vec<String>,
}

pub fn read_st<R: BufRead + Seek>(reader: &mut R, stl: bool) -> BinResult<StReadOutcome> {
	reader.rewind()?;
	let header: StHeaderCommon = if stl {
		StlHeader::read(reader)?.into()
	} else {
		StbOrStmHeader::read(reader)?.into()
	};
	
	// TODO: checked conversion and multiplication
	let entry_count = header.entry_count as usize;
	let field_count = header.field_count as usize;
	let raw_count = entry_count * field_count;
	
	reader.seek(SeekFrom::Start(header.data_pointer))?;
	let string_pointers = read_pointers(reader, raw_count)?;
	let strings: Vec<String> = string_pointers.iter().map(|pointer| -> BinResult<_> {
		reader.seek(SeekFrom::Start(*pointer))?;
		Ok(NullString::read_le(reader)?.to_string())
	}).collect::<BinResult<_>>()?;
	
	Ok(StReadOutcome {
		field_count,
		strings,
	})
	
	// Debugging output garbage. The next time I work on these things, I should make sure this function only reads raw data, and have it be processed/displayed elsewhere.
	/*
	let mut checksums = None;
	if let Some(header_full) = header_full {
		reader.seek(SeekFrom::Start(header_full.checksums_pointer))?;
		checksums = Some(read_pointers(reader, raw_count)?);
	}
	let checksums = checksums;
	
	println!("Entry count: {}", header.entry_count);
	println!("Field count: {}", header.field_count);
	
	if let Some(header_full) = header_full {
		for extra in [(1, header_full.extra1), (2, header_full.extra2)] {
			println!("Extra data {} count: {}", extra.0, extra.1.extra_entry_count);
		}
	}
	
	for i in raw_count.saturating_sub(20)..raw_count {
		let string = &strings[i];
		if let Some(chk) = &checksums {
			let their_checksum = chk[i];
			println!("{:?} {:08X}", string, their_checksum);
		} else {
			println!("{:?}", string);
		}
	}
	
	if let Some(header_full) = header_full {
		for extra in [(1, header_full.extra1), (2, header_full.extra2)] {
			println!("(Extra data {})", extra.0);
			reader.seek(SeekFrom::Start(extra.1.pointer))?;
			let extra_entry_count = extra.1.extra_entry_count as usize; // TODO: Checked conversion again...
			let pointers = read_pointers(reader, extra_entry_count)?;
			for (i, pointer) in pointers.iter().enumerate() {
				seek_absolute(reader, *pointer)?;
				let extra_data = StbOrStmDataExtra::read(reader)?;
				if i >= pointers.len().saturating_sub(20) {
					println!("{:?}", extra_data.pieces);
				}
			}
		}
	}
	
	//println!("{:?}", idk);
	//todo!("we've gotta show some entries and those possibly-checksums side by side, i guess");
	Ok(())
	*/
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;
	
	const STL_HEADER_SAMPLE_RAW: [u8; 24] = [
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x6B, 0x0B, 0x00, 0x00,
		0x01, 0x00, 0x00, 0x00,
		0xCD, 0xAB, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
	];
	const STL_HEADER_SAMPLE: StlHeader = StlHeader {
		entry_count: 2923,
		field_count: 1,
		data_pointer: 0x1ABCD,
	};
	
	#[test]
	fn stl_header_deserialize() {
		let mut reader = Cursor::new(STL_HEADER_SAMPLE_RAW);
		let result = StlHeader::read(&mut reader).unwrap();
		assert_eq!(result, STL_HEADER_SAMPLE);
	}
	
	#[test]
	fn stl_header_serialize() {
		let mut writer = Cursor::new(Vec::<u8>::new());
		STL_HEADER_SAMPLE.write(&mut writer).unwrap();
		let result = writer.into_inner();
		assert_eq!(result, STL_HEADER_SAMPLE_RAW);
	}
	
	const STM_HEADER_SAMPLE_RAW: [u8; 64] = [
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x6B, 0x0B, 0x00, 0x00,
		0x08, 0x00, 0x00, 0x00,
		0xCD, 0xAB, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
		0xCD, 0xAB, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00,
		0xCE, 0x03, 0x00, 0x00,
		0xCD, 0xAB, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00,
		0x02, 0x00, 0x00, 0x00,
		0xCD, 0xAB, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
	];
	const STM_HEADER_SAMPLE: StbOrStmHeader = StbOrStmHeader {
		entry_count: 2923,
		field_count: 8,
		checksums_pointer: 0x1ABCD,
		data_pointer: 0x2ABCD,
		extra1: StbOrStmHeaderExtra {
			extra_entry_count: 974,
			pointer: 0x3ABCD,
		},
		extra2: StbOrStmHeaderExtra {
			extra_entry_count: 2,
			pointer: 0x4ABCD,
		},
	};
	
	#[test]
	fn stm_header_deserialize() {
		let mut reader = Cursor::new(STM_HEADER_SAMPLE_RAW);
		let result = StbOrStmHeader::read(&mut reader).unwrap();
		assert_eq!(result, STM_HEADER_SAMPLE);
	}
	
	#[test]
	fn stm_header_serialize() {
		let mut writer = Cursor::new(Vec::<u8>::new());
		STM_HEADER_SAMPLE.write(&mut writer).unwrap();
		let result = writer.into_inner();
		assert_eq!(result, STM_HEADER_SAMPLE_RAW);
	}
}
