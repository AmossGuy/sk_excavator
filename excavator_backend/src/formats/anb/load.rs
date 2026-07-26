use super::definition::*;

use hecs::{Entity, World};
use hecs_hierarchy::HierarchyMut;
use zerocopy::{FromBytes, LE, U64};

pub fn load_from_bytes(bytes: &[u8], world: &mut World) -> anyhow::Result<Entity> {
	load_header(bytes, world)
}

fn load_header(bytes: &[u8], world: &mut World) -> anyhow::Result<Entity> {
	let header_component = parse_header(bytes)?;
	let header_entity = world.spawn((header_component,));
	
	let (root_component, root_child_offset, root_child_count) = parse_node_common(bytes, std::mem::size_of::<HeaderRaw>())?;
	let root_entity = world.attach_new::<(), _>(header_entity, (root_component,))?;
	
	load_node_list(bytes, world, root_entity, root_child_offset, root_child_count)?;
	
	Ok(header_entity)
}

fn load_node_list(bytes: &[u8], world: &mut World, parent: Entity, offset: u64, length: u32) -> anyhow::Result<()> {
	let offset_bytes = &bytes[(offset as usize)..];
	let (slice, _) = <[U64::<LE>]>::ref_from_prefix_with_elems(offset_bytes, length as usize)
		.map_err(|e| anyhow::anyhow!(e.to_string()))?;
	
	for pointer in slice {
		let pointer = pointer.get();
		let (node_component, children_offset, children_count) = parse_node_common(bytes, pointer as usize)?;
		let node_entity = world.attach_new::<(), _>(parent, (node_component,))?;
		
		load_node_list(bytes, world, node_entity, children_offset, children_count)?;
	}
	
	Ok(())
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<Header> {
	let (header_raw, _) = HeaderRaw::ref_from_prefix(bytes)
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok(Header {
		unknown_04: header_raw.unknown_04.get(),
		unknown_08: header_raw.unknown_08.get(),
		unknown_0C: header_raw.unknown_0C.get(),
		unknown_10: header_raw.unknown_10.get(),
		unknown_14: header_raw.unknown_14.get(),
	})
}

fn parse_node_common(bytes: &[u8], offset: usize) -> anyhow::Result<(NodeCommon, u64, u32)> {
	let (node_raw, _) = NodeCommonRaw::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	Ok((NodeCommon {
		kind: node_raw.kind.get(),
	}, node_raw.child_array_pointer.get(), node_raw.child_count.get()))
}
