use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, NullString};

use super::util_binary::{read_pointers, seek_absolute};

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
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
	extra1: StbOrStmHeaderExtra,
	extra2: StbOrStmHeaderExtra,
}

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
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
struct StbOrStmDataExtra {
	piece_count: u64,
	magic: MagicTen,
	#[br(count = piece_count)]
	pieces: Vec<(u32, u32)>,
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
	let string_pointers = read_pointers(reader, raw_count)?;
	let strings: Vec<String> = string_pointers.iter().map(|pointer| -> BinResult<_> {
		reader.seek(SeekFrom::Start(*pointer))?;
		Ok(NullString::read_le(reader)?.to_string())
	}).collect::<BinResult<_>>()?;
	
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
					println!("{:?}", extra_data);
				}
			}
		}
	}
	
	//println!("{:?}", idk);
	//todo!("we've gotta show some entries and those possibly-checksums side by side, i guess");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;
	
	fn stl_header_sample() -> (Vec<u8>, StlHeader) {
		let raw = vec![
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x6B, 0x0B, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
			0xCD, 0xAB, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
		];
		let header = StlHeader {
			entry_count: 2923,
			field_count: 1,
			data_pointer: 0x1ABCD,
		};
		(raw, header)
	}
	
	#[test]
	fn stl_header_deserialize() {
		let (raw, header) = stl_header_sample();
		let mut reader = Cursor::new(raw);
		let result = StlHeader::read(&mut reader).unwrap();
		assert_eq!(result, header);
	}
	
	#[test]
	fn stl_header_serialize() {
		let (raw, header) = stl_header_sample();
		let mut writer = Cursor::new(Vec::<u8>::new());
		header.write(&mut writer).unwrap();
		let result = writer.into_inner();
		assert_eq!(result, raw);
	}
}
