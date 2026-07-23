use super::definition::*;
use super::super::TreeMarker;
use hecs::{Entity, World};
use hecs_hierarchy::HierarchyMut;
use zerocopy::{FromBytes, LE, U64};

pub fn load_from_bytes(bytes: &[u8], world: &mut World) -> Entity {
	load_header(bytes, world)
}

fn load_header(bytes: &[u8], world: &mut World) -> Entity {
	// early state of work on this... unwrapping okay for the moment
	let (header_component, node_offset_x2) = parse_header(bytes).unwrap();
	let header_entity = world.spawn((header_component,));
	
	load_node_list(bytes, world, header_entity, node_offset_x2, 1);
	
	header_entity
}

fn load_node_list(bytes: &[u8], world: &mut World, parent: Entity, offset: u64, length: u32) {
	let offset_bytes = &bytes[(offset as usize)..];
	let slice = <[U64::<LE>]>::ref_from_prefix_with_elems(offset_bytes, length as usize).unwrap().0;
	
	for pointer in slice {
		let pointer = pointer.get();
		let (node_component, children_offset, children_count) = parse_node_common(bytes, pointer as usize).unwrap();
		let node_entity = world.attach_new::<TreeMarker, _>(parent, (node_component,)).unwrap();
		
		load_node_list(bytes, world, node_entity, children_offset, children_count);
	}
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<(Header, u64)> {
	let (header_raw, _) = HeaderRaw::ref_from_prefix(bytes)
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok((Header {
		unknown_04: header_raw.unknown_04.get(),
		unknown_08: header_raw.unknown_08.get(),
		unknown_0C: header_raw.unknown_0C.get(),
		unknown_10: header_raw.unknown_10.get(),
		unknown_14: header_raw.unknown_14.get(),
		unknown_18: header_raw.unknown_18.get(),
		unknown_1C: header_raw.unknown_1C.get(),
	}, header_raw.root_node_pointer.get()))
}

fn parse_node_common(bytes: &[u8], offset: usize) -> anyhow::Result<(NodeCommon, u64, u32)> {
	let (node_raw, _) = NodeCommonRaw::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	Ok((NodeCommon {
		kind: node_raw.kind.get(),
	}, node_raw.child_array_pointer.get(), node_raw.child_count.get()))
}
