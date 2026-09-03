use crate::formats::common::ArcBytes;
use super::{def_live as live, def_raw as raw};

use std::iter;
use thunderdome::Arena;
use undoredo::Recorder;
use zerocopy::{FromBytes, LE, U64};

pub fn load_from_bytes(bytes: &ArcBytes) -> anyhow::Result<live::Pak> {
	let (header, file_list_cont) = parse_header(bytes)?;
	
	let mut files = Arena::new();
	for file_cont in file_list_cont.iter_pointers(bytes.get())? {
		files.insert(file_cont.parse_file(bytes)?);
	}
	
	Ok(live::Pak {
		header: Recorder::new([header]),
		files: Recorder::new(files),
	})
}

fn parse_header(bytes: &ArcBytes) -> anyhow::Result<(live::Header, FileListContinuation)> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes.get())
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"\0\0\0\0" {
		anyhow::bail!("wrong magic");
	}
	
	Ok((live::Header {
		// no fields?
	}, FileListContinuation {
		file_count: header_raw.file_count.get(),
		data_array_pointer: header_raw.data_array_pointer.get(),
		name_array_pointer: header_raw.name_array_pointer.get(),
	}))
}

struct FileListContinuation {
	file_count: u32,
	data_array_pointer: u64,
	name_array_pointer: u64,
}

impl FileListContinuation {
	fn iter_pointers<'a>(&self, bytes: &'a [u8]) -> anyhow::Result<impl Iterator<Item = FileContinuation> + 'a> {
		let data_pointers = pointer_slice(bytes, self.data_array_pointer, self.file_count)?;
		let name_pointers = pointer_slice(bytes, self.name_array_pointer, self.file_count)?;
		
		Ok(iter::zip(data_pointers, name_pointers).map(|(data_pointer, name_pointer)| {
			let (data_pointer, name_pointer) = (data_pointer.get(), name_pointer.get());
			FileContinuation { data_pointer, name_pointer }
		}))
	}
}

fn pointer_slice<'a>(bytes: &'a [u8], start_offset: u64, count: u32) -> anyhow::Result<&'a [U64<LE>]> {
	let (start_offset_u, count_u) = (start_offset as usize, count as usize);
	let sliced_bytes = bytes.get(start_offset_u..)
		.ok_or_else(|| anyhow::anyhow!("pointer list out of bounds"))?;
	
	let (pointers, _) = <[U64<LE>]>::ref_from_prefix_with_elems(sliced_bytes, count_u)
		.map_err(|e| e.map_src(<[_]>::to_vec))?;
	Ok(pointers)
}

struct FileContinuation {
	data_pointer: u64,
	name_pointer: u64,
}

impl FileContinuation {
	fn parse_file(&self, bytes: &ArcBytes) -> anyhow::Result<live::File> {
		let name_bytes = null_terminated_bytes(bytes, self.name_pointer)?;
		let (header_raw, file_bytes) = self.get_file_data(bytes)?;
		
		Ok(live::File {
			filename: name_bytes,
			time: header_raw.time.get(),
			filename_hash: header_raw.filename_hash.get(),
			flags: header_raw.flags.get(),
			specials: header_raw.specials.get(),
			padding: header_raw.padding.get(),
			data: file_bytes,
		})
	}
	
	fn get_file_data<'a>(&self, bytes: &'a ArcBytes) -> anyhow::Result<(&'a raw::FileHeader, ArcBytes)> {
		let (header, after_header) = raw::FileHeader::ref_from_prefix(bytes.get())
			.map_err(|e| e.map_src(<[_]>::to_vec))?;
		
		let file_size = header.size.get() as usize;
		let data_slice = after_header.get(..file_size)
			.ok_or_else(|| anyhow::anyhow!("file goes past end"))?;
		
		let data_yoke = bytes.map_project_cloned(|slice, _| {
			let range = slice.subslice_range(data_slice).unwrap();
			&slice[range]
		});
		
		Ok((header, data_yoke))
	}
}

fn null_terminated_bytes(bytes: &ArcBytes, offset: u64) -> anyhow::Result<ArcBytes> {
	let offset_u = offset as usize;
	
	bytes.try_map_project_cloned(|slice, _| {
		let name_slice = &slice.get(offset_u..)
			.ok_or_else(|| anyhow::anyhow!("out of bounds"))?;
		// TODO: this would be a good place to use slice::split_once when it's stabilized
		let name_length = name_slice.iter().position(|&byte| byte == 0)
			.ok_or_else(|| anyhow::anyhow!("no null terminator"))?;
		Ok(&name_slice[..name_length])
	})
}
