// OUTDATED: DELETE THIS FILE SOON
/*
use std::ffi::CString;
use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinResult, BinWrite, NullString};

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

#[derive(BinRead, BinWrite, Copy, Clone, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct StmHeader {
	string_count: u32,
	stb_field_count: u32, // 8 in dialogue.stm, 1 in menus.stm; name is hypothesis
	something_pointer: u64, // look at this more later
	identifier_pointer: u64, // pointer to an area with pointers to strings that seem to be internal names for pieces of text
	// more stuff
}

pub fn look_inside_stm<R: BufRead + Seek>(reader: &mut R) -> BinResult<()> {
	reader.rewind()?;
	let header = StmHeader::read(reader)?;
	
	/*
	reader.seek(SeekFrom::Start(header.identifier_pointer))?;
	loop {
		let pointer = u64::read_le(reader)?;
		// I do not yet know the correct method to determine the length of this section of the file
		// For now, use a heuristic: these files aren't big enough for more than 5 nybbles to be used in a pointer
		// This will overshoot the end a little, since immediately after is the short string "ID"
		if pointer > 0xF_FF_FF {
			break;
		}
		
		let string: Option<String> = match pointer {
			0 => None,
			_ => {
				seek_absolute(reader, pointer)?;
				Some(NullString::read_le(reader)?.to_string())
			},
		};
		println!("{}", string.unwrap_or("<null>".to_string()));
	}
	*/
	
	reader.seek(SeekFrom::Start(header.identifier_pointer))?;
	let mut field_name_pointers = Vec::new();
	for _i in 0..header.stb_field_count {
		field_name_pointers.push(u64::read_le(reader)?);
	}
	for pointer in field_name_pointers {
		seek_absolute(reader, pointer)?;
		let string = NullString::read_le(reader)?.to_string();
		println!("{}", string);
	}
	
	Ok(())
}
*/
