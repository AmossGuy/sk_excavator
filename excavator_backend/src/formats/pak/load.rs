use crate::formats::common::ArcBytes;
use super::{def_live as live, def_raw as raw};

use undoredo::Recorder;
use zerocopy::{FromBytes, LE, U64};

pub fn load_from_bytes(bytes: &ArcBytes) -> anyhow::Result<live::Pak> {
	let (header, _continuation) = parse_header(bytes)?;
	Ok(live::Pak {
		header: Recorder::new([header]),
	})
}

/*
fn load_header(bytes: &ArcBytes, world: &mut World) -> anyhow::Result<Entity> {
	let (header_component, pointers) = parse_header(bytes)?;
	let header_entity = world.spawn(header_component);
	
	let header_id = header_entity.id();
	let mut spawner = ChildSpawner::new(world, header_id);
	load_file_list(bytes, &mut spawner, pointers)?;
	
	Ok(header_id)
}

fn load_file_list(bytes: &ArcBytes, spawner: &mut ChildSpawner<'_>, pointers: FileListPointers) -> anyhow::Result<()> {
	let offset_a_bytes = bytes.get().get(pointers.data_array_pointer as usize..)
		.ok_or_else(|| anyhow::anyhow!("node list out of bounds"))?;
	let (slice_a, _) = <[U64::<LE>]>::ref_from_prefix_with_elems(offset_a_bytes, pointers.file_count as usize)
		.map_err(|e| anyhow::anyhow!(e.to_string()))?;
	
	let offset_b_bytes = bytes.get().get(pointers.name_array_pointer as usize..)
		.ok_or_else(|| anyhow::anyhow!("node list out of bounds"))?;
	let (slice_b, _) = <[U64::<LE>]>::ref_from_prefix_with_elems(offset_b_bytes, pointers.file_count as usize)
		.map_err(|e| anyhow::anyhow!(e.to_string()))?;
	
	for i in 0..(pointers.file_count as usize) {
		let metadata_pointer = slice_a[i].get();
		let name_pointer = slice_b[i].get();
		
		let metadata_slice = &bytes.get()[(metadata_pointer as usize)..];
		let (metadata_raw, _) = raw::FileHeader::ref_from_prefix(metadata_slice)
			.map_err(|e| anyhow::anyhow!(e.to_string()))?;
		let metadata_component = live::FileMetadata {
			time: metadata_raw.time.get(),
			filename_hash: metadata_raw.filename_hash.get(),
			flags: metadata_raw.flags.get(),
			specials: metadata_raw.specials.get(),
			padding: metadata_raw.padding.get(),
		};
		
		let name_bytes: ArcBytes = bytes.clone().try_map_project(|slice, _| -> anyhow::Result<_> {
			let name_slice = &slice[(name_pointer as usize)..];
			let name_length = name_slice.iter().position(|&byte| byte == 0)
				.ok_or_else(|| anyhow::anyhow!("no null terminator"))?;
			Ok(&name_slice[..name_length])
		})?;
		let name_component = live::FileName { name: name_bytes };
		
		spawner.spawn((metadata_component, name_component));
	}
	
	Ok(())
}
*/

#[derive(Copy, Clone)]
struct FileListPointers {
	file_count: u32,
	data_array_pointer: u64,
	name_array_pointer: u64,
}

fn parse_header(bytes: &ArcBytes) -> anyhow::Result<(live::Header, FileListPointers)> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes.get())
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"\0\0\0\0" {
		anyhow::bail!("wrong magic");
	}
	
	Ok((live::Header {
	}, FileListPointers {
		file_count: header_raw.file_count.get(),
		data_array_pointer: header_raw.data_array_pointer.get(),
		name_array_pointer: header_raw.name_array_pointer.get(),
	}))
}
