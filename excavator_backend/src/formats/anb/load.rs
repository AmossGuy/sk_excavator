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
	
	let node_offset_x2 = node_offset_x2 as usize;
	let node_offset = U64::<LE>::ref_from_prefix(&bytes[node_offset_x2..]).unwrap().0.get() as usize;
	
	let node_component = parse_node_common(bytes, node_offset as usize).unwrap();
	let _node_entity = world.attach_new::<TreeMarker, _>(header_entity, (node_component,));
	
	header_entity
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

fn parse_node_common(bytes: &[u8], offset: usize) -> anyhow::Result<NodeCommon> {
	let (node_raw, _) = NodeCommonRaw::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	Ok(NodeCommon {
		kind: node_raw.kind.get(),
	})
}
