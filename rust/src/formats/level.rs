use std::ffi::CStr;
use std::io::{BufRead, Seek, SeekFrom};

use binrw::{BinRead, BinWrite};

/*
#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0")]
struct LvbHeader {
	something1: u32,
	pointer1: u64,
	maybe_quantity: u32,
	something2: u32,
	pointer2: u64,
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0")]
struct LvbThingy {
	field0: u32,
	bleh_a: u16,
	maybe_x: u16,
	bleh_b: u16,
	maybe_y: u16,
	field3: u32,
	field4: u32,
	field5: u32,
	field6: u32,
	field7: u32,
	field8: u32,
	increasing: u64,
}

pub fn lvb_wip<R: BufRead + Seek>(reader: &mut R) -> BinResult<()> {
	let header = LvbHeader::read(reader)?;
	
	reader.seek(SeekFrom::Start(header.pointer2))?;
	for _i in 0..header.maybe_quantity {
		let thingy = LvbThingy::read(reader)?;
		println!("({}, {})", thingy.maybe_x, thingy.maybe_y);
	}
	
	Ok(())
}
*/

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
struct HeaderElement {
	value_a: u32,
	value_b: u32,
	pointer: u64,
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little, magic = b"\0\0\0\0\0\0\0\0")]
struct LtbHeader {
	first_value_a: u32,
	first_value_b: u32,
	elements: [HeaderElement; 8],
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little)]
#[expect(dead_code)] // we'll get back to this
struct LvbHeader {
	elements: [HeaderElement; 7],
}

#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Debug)]
#[brw(little)]
struct LtbLayerEntry {
	name: [u8; 32],
	numbers: [u32; 24],
}

pub fn ltb_wip<R: BufRead + Seek>(reader: &mut R) -> Result<(), Box<dyn std::error::Error>> {
	let header = LtbHeader::read(reader)?;
	
	let element = header.elements[0];
	reader.seek(SeekFrom::Start(element.pointer))?;
	let mut writer = csv::Writer::from_writer(std::io::stdout());
	for _i in 0..element.value_b {
		let layer = LtbLayerEntry::read(reader)?;
		let mut row = Vec::<String>::new();
		let lossy_name = CStr::from_bytes_until_nul(&layer.name)?.to_string_lossy().into_owned();
		row.push(lossy_name);
		for number in layer.numbers {
			row.push(number.to_string());
		}
		writer.write_record(row)?;
	}
	writer.flush()?;
	Ok(())
}
