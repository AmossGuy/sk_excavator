use super::{def_live as live, def_raw as raw};

use hecs::{Entity, World};
use hecs_hierarchy::HierarchyMut;
use zerocopy::{FromBytes, LE, U64};

pub fn load_from_bytes(bytes: &[u8], world: &mut World) -> anyhow::Result<Entity> {
	load_header(bytes, world)
}

fn load_header(bytes: &[u8], world: &mut World) -> anyhow::Result<Entity> {
	let header_component = parse_header(bytes)?;
	let header_entity = world.spawn((header_component,));
	
	let (root_component, root_child_offset, root_child_count) = parse_node_common(bytes, std::mem::size_of::<raw::Header>())?;
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

fn parse_header(bytes: &[u8]) -> anyhow::Result<live::Header> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes)
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok(live::Header {
		unknown_04: header_raw.unknown_04.get(),
		unknown_08: header_raw.unknown_08.get(),
		unknown_0C: header_raw.unknown_0C.get(),
		unknown_10: header_raw.unknown_10.get(),
		unknown_14: header_raw.unknown_14.get(),
	})
}

fn parse_node_common(bytes: &[u8], offset: usize) -> anyhow::Result<(live::Node, u64, u32)> {
	let (node_common_raw, followup) = raw::NodeCommon::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	let kind = node_common_raw.kind.get();
	
	let node = match kind {
		7 => {
			let (node_raw, _) = raw::NodeMetaRect::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaRect(live::NodeMetaRect {
				center_x: node_raw.center_x.get(),
				center_y: node_raw.center_y.get(),
				center_z: node_raw.center_z.get(),
				extents_x: node_raw.extents_x.get(),
				extents_y: node_raw.extents_y.get(),
				extents_z: node_raw.extents_z.get(),
				angle: node_raw.angle.get(),
				padding: node_raw.padding.get(),
			})
		},
		_ => live::Node::UnknownKind(kind),
	};
	
	Ok((node, node_common_raw.child_array_pointer.get(), node_common_raw.child_count.get()))
}
