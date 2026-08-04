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
	
	let (root_component, root_child_offset, root_child_count) = parse_node(bytes, std::mem::size_of::<raw::Header>())?;
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
		let (node_component, children_offset, children_count) = parse_node(bytes, pointer as usize)?;
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
		fixup: header_raw.fixup.get(),
		version: header_raw.version.get(),
		padding_a: header_raw.padding_a.get(),
		padding_b: header_raw.padding_b.get(),
		padding_c: header_raw.padding_c.get(),
	})
}

fn parse_node(bytes: &[u8], offset: usize) -> anyhow::Result<(live::Node, u64, u32)> {
	let (node_common_raw, followup) = raw::NodeCommon::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	let kind = node_common_raw.kind.get();
	
	let node = match kind {
		0 => live::Node::Base,
		1 => {
			let (node_raw, _) = raw::NodeTexture::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::Texture(live::NodeTexture {
				width: node_raw.width.get(),
				height: node_raw.height.get(),
				flags: node_raw.flags.get(),
				padding: node_raw.padding.get(),
			})
		},
		2 => {
			let (node_raw, _) = raw::NodeVertex::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			let extra_data = parse_data_block(bytes, node_raw.data_pointer.get() as usize)?.to_vec();
			live::Node::Vertex(live::NodeVertex {
				vert_count: node_raw.vert_count.get(),
				flags: node_raw.flags.get(),
				extra_data,
			})
		}
		3 => live::Node::Meta,
		4 => {
			let (node_raw, _) = raw::NodeMetaScalar::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaScalar(live::NodeMetaScalar {
				unk_1: node_raw.unk_1.get(),
				unk_2: node_raw.unk_2.get(),
			})
		},
		5 => {
			let (node_raw, _) = raw::NodeMetaPoint::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaPoint(live::NodeMetaPoint {
				x: node_raw.x.get(),
				y: node_raw.y.get(),
				z: node_raw.z.get(),
				padding: node_raw.padding.get(),
			})
		},
		6 => {
			let (node_raw, _) = raw::NodeMetaAnchor::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaAnchor(live::NodeMetaAnchor {
				x: node_raw.x.get(),
				y: node_raw.y.get(),
				z: node_raw.z.get(),
				angle: node_raw.angle.get(),
			})
		},
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
		8 => {
			let (node_raw, _) = raw::NodeMetaString::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaString(live::NodeMetaString {
				string_length: node_raw.string_length.get(),
				padding: node_raw.padding.get(),
			})
		},
		9 => {
			let (_node_raw, _) = raw::NodeMetaTable::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::MetaTable(live::NodeMetaTable {
			})
		},
		10 => {
			let (node_raw, _) = raw::NodeFrame::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::Frame(live::NodeFrame {
				min_x: node_raw.min_x.get(),
				max_x: node_raw.max_x.get(),
				min_y: node_raw.min_y.get(),
				max_y: node_raw.max_y.get(),
			})
		},
		11 => {
			let (node_raw, _) = raw::NodeSequenceFrame::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::SequenceFrame(live::NodeSequenceFrame {
				frame: node_raw.frame.get(),
				delay: node_raw.delay.get(),
			})
		},
		12 => {
			let (node_raw, _) = raw::NodeSequence::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::Sequence(live::NodeSequence {
				hashname: node_raw.hashname.get(),
				frame_count: node_raw.frame_count.get(),
			})
		},
		13 => {
			let (node_raw, _) = raw::NodeAnimation::ref_from_prefix(followup)
				.map_err(|e| anyhow::anyhow!("{}", e))?;
			live::Node::Animation(live::NodeAnimation {
				sequence_count: node_raw.sequence_count.get(),
				frame_count: node_raw.frame_count.get(),
				single_texture: node_raw.single_texture.get(),
				palette_index: node_raw.palette_index.get(),
			})
		},
		_ => live::Node::UnknownKind(kind),
	};
	
	Ok((node, node_common_raw.child_array_pointer.get(), node_common_raw.child_count.get()))
}

fn parse_data_block(bytes: &[u8], offset: usize) -> anyhow::Result<&[u8]> {
	let (header, followup) = raw::DataBlockHeader::ref_from_prefix(&bytes[offset..])
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	if header.magic != [0xFF, 0xFF, 0xFF, 0x00] {
		anyhow::bail!("wrong data block magic");
	}
	let data_size = header.data_size.get() as usize;
	Ok(&followup[..data_size])
}
