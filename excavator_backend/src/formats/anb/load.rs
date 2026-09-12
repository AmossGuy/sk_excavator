use crate::formats::common::{ArcBytes, pointer_slice};
use {super as live, super::raw as raw};

use std::collections::VecDeque;
use thunderdome::Arena;
use zerocopy::FromBytes;

pub fn load_from_bytes(bytes: &ArcBytes) -> anyhow::Result<live::Anb> {
	let (mut header, root_node_cont) = parse_header(bytes)?;
	
	let mut node_arena = Arena::new();
	
	let (root_data, root_children_cont) = root_node_cont.parse_node(bytes)?;
	let root_node = live::Node { parent: None, children: Vec::new(), data: root_data };
	let root_node_id = live::NodeId(node_arena.insert(root_node));
	
	// Can't do this until now, when the root node's id is determined
	header.root_node = Some(root_node_id);
	
	let mut children_get_queue = VecDeque::from([(root_children_cont, root_node_id)]);
	while let Some((children_cont, parent_id)) = children_get_queue.pop_front() {
		for child_cont in children_cont.children(bytes.get())? {
			let (child_data, child_children_cont) = child_cont.parse_node(bytes)?;
			let child_node = live::Node { parent: Some(parent_id), children: Vec::new(), data: child_data };
			let child_node_id = live::NodeId(node_arena.insert(child_node));
			
			let parent_mut = node_arena.get_mut(parent_id.0).expect("parent node should exist");
			parent_mut.children.push(child_node_id);
			
			if child_children_cont.count != 0 {
				children_get_queue.push_back((child_children_cont, child_node_id));
			}
		}
	}
	
	Ok(live::Anb::from_parts(header, node_arena))
}

fn parse_header(bytes: &ArcBytes) -> anyhow::Result<(live::Header, NodeContinuation)> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes.get())
		.map_err(|e| e.map_src(<[_]>::to_vec))?;
	let followup_offset = std::mem::size_of::<raw::Header>() as u64;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok((live::Header {
		root_node: None,
		
		fixup: header_raw.fixup.get(),
		version: header_raw.version.get(),
		padding_a: header_raw.padding_a.get(),
		padding_b: header_raw.padding_b.get(),
		padding_c: header_raw.padding_c.get(),
	}, NodeContinuation {
		offset: followup_offset,
	}))
}

struct NodeContinuation {
	offset: u64,
}

impl NodeContinuation {
	fn parse_node(&self, bytes: &ArcBytes) -> anyhow::Result<(live::NodeData, ChildrenContinuation)> {
		// I just didn't want to deal with the noise changing the indent level would add to the diff
		// ...Although, also, it makes sense to put such a large function somewhere out of the way
		parse_node(bytes, self.offset)
	}
}

struct ChildrenContinuation {
	offset: u64,
	count: u32,
}

impl ChildrenContinuation {
	fn children<'a>(&self, bytes: &'a [u8]) -> anyhow::Result<impl Iterator<Item = NodeContinuation> + 'a> {
		let pointers = pointer_slice(bytes, self.offset, self.count)?;
		Ok(pointers.into_iter().map(|pointer| {
			NodeContinuation { offset: pointer.get() }
		}))
	}
}

fn parse_node(bytes: &ArcBytes, offset: u64) -> anyhow::Result<(live::NodeData, ChildrenContinuation)> {
	let offset_u = offset as usize;
	let offset_bytes = bytes.get().get(offset_u..)
		.ok_or_else(|| anyhow::anyhow!("node out of bounds"))?;
	let (node_common_raw, followup) = raw::NodeCommon::ref_from_prefix(offset_bytes)
		.map_err(|e| e.map_src(<[_]>::to_vec))?;
	let kind = node_common_raw.kind.get();
	
	let node = match kind {
		raw::NODE_KIND_BASE => live::NodeData::Base,
		raw::NODE_KIND_TEXTURE => {
			let (node_raw, _) = raw::NodeTexture::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			let data_block = parse_data_block(bytes, node_raw.data_pointer.get() as usize)?;
			
			live::NodeData::Texture(live::NodeTexture {
				width: node_raw.width.get(),
				height: node_raw.height.get(),
				flags: node_raw.flags.get(),
				padding: node_raw.padding.get(),
				data_block,
			})
		},
		raw::NODE_KIND_VERTEX => {
			let (node_raw, _) = raw::NodeVertex::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			let data_block = parse_data_block(bytes, node_raw.data_pointer.get() as usize)?;
			
			live::NodeData::Vertex(live::NodeVertex {
				vert_count: node_raw.vert_count.get(),
				flags: node_raw.flags.get(),
				data_block,
			})
		}
		raw::NODE_KIND_META => live::NodeData::Meta,
		raw::NODE_KIND_META_SCALAR => {
			let (node_raw, _) = raw::NodeMetaScalar::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::MetaScalar(live::NodeMetaScalar {
				unk_1: node_raw.unk_1.get(),
				unk_2: node_raw.unk_2.get(),
			})
		},
		raw::NODE_KIND_META_POINT => {
			let (node_raw, _) = raw::NodeMetaPoint::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::MetaPoint(live::NodeMetaPoint {
				x: node_raw.x.get(),
				y: node_raw.y.get(),
				z: node_raw.z.get(),
				padding: node_raw.padding.get(),
			})
		},
		raw::NODE_KIND_META_ANCHOR => {
			let (node_raw, _) = raw::NodeMetaAnchor::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::MetaAnchor(live::NodeMetaAnchor {
				x: node_raw.x.get(),
				y: node_raw.y.get(),
				z: node_raw.z.get(),
				angle: node_raw.angle.get(),
			})
		},
		raw::NODE_KIND_META_RECT => {
			let (node_raw, _) = raw::NodeMetaRect::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::MetaRect(live::NodeMetaRect {
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
		raw::NODE_KIND_META_STRING => {
			let (node_raw, _) = raw::NodeMetaString::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			let data_block = parse_data_block(bytes, node_raw.string_offset.get() as usize)?;
			
			live::NodeData::MetaString(live::NodeMetaString {
				string_length: node_raw.string_length.get(),
				padding: node_raw.padding.get(),
				data_block,
			})
		},
		raw::NODE_KIND_META_TABLE => {
			let (node_raw, _) = raw::NodeMetaTable::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			let data_block = parse_data_block(bytes, node_raw.hashname_pointer.get() as usize)?;
			
			live::NodeData::MetaTable(live::NodeMetaTable {
				data_block
			})
		},
		raw::NODE_KIND_FRAME => {
			let (node_raw, _) = raw::NodeFrame::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::Frame(live::NodeFrame {
				min_x: node_raw.min_x.get(),
				max_x: node_raw.max_x.get(),
				min_y: node_raw.min_y.get(),
				max_y: node_raw.max_y.get(),
			})
		},
		raw::NODE_KIND_SEQUENCE_FRAME => {
			let (node_raw, _) = raw::NodeSequenceFrame::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::SequenceFrame(live::NodeSequenceFrame {
				frame: node_raw.frame.get(),
				delay: node_raw.delay.get(),
			})
		},
		raw::NODE_KIND_SEQUENCE => {
			let (node_raw, _) = raw::NodeSequence::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			live::NodeData::Sequence(live::NodeSequence {
				hashname: node_raw.hashname.get(),
				frame_count: node_raw.frame_count.get(),
			})
		},
		raw::NODE_KIND_ANIMATION => {
			let (node_raw, _) = raw::NodeAnimation::ref_from_prefix(followup)
				.map_err(|e| e.map_src(<[_]>::to_vec))?;
			let data_block = parse_data_block(bytes, node_raw.hashname_pointer.get() as usize)?;
			
			live::NodeData::Animation(live::NodeAnimation {
				sequence_count: node_raw.sequence_count.get(),
				frame_count: node_raw.frame_count.get(),
				single_texture: node_raw.single_texture.get(),
				palette_index: node_raw.palette_index.get(),
				data_block,
			})
		},
		other => anyhow::bail!("unknown node kind: {}", other),
	};
	
	let cont = ChildrenContinuation {
		offset: node_common_raw.child_array_pointer.get(),
		count: node_common_raw.child_count.get(),
	};
	Ok((node, cont))
}

fn parse_data_block(bytes: &ArcBytes, offset: usize) -> anyhow::Result<Option<live::DataBlock>> {
	if offset == 0 {
		return Ok(None);
	}
	
	let temp_yoke: yoke::Yoke<(u32, &[u8]), _> = bytes.try_map_project_cloned(|slice, _| -> anyhow::Result<_> {
		let slice = slice.get(offset..)
			.ok_or_else(|| anyhow::anyhow!("data block out of range"))?;
		let (header, followup) = raw::DataBlockHeader::ref_from_prefix(&slice)
			.map_err(|e| e.map_src(<[_]>::to_vec))?;
		let flags = header.flags.get();
		let data_size = header.data_size.get() as usize;
		let data = followup.get(..data_size)
			.ok_or_else(|| anyhow::anyhow!("data block data goes past end"))?;
		Ok((flags, data))
	})?;
	
	let flags: u32 = temp_yoke.get().0;
	let data_yoke: ArcBytes = temp_yoke.map_project(|(_, data), _| data);
	
	Ok(Some(live::DataBlock { flags, data: data_yoke }))
}

pub fn parse_vertex_data_block(
	data_block: &live::DataBlock, vert_count: u32,
) -> anyhow::Result<Vec<live::VertexEntry>> {
		let vert_count_u = vert_count as usize;
		let raw_entry_slice = <[raw::VertexEntry]>::ref_from_bytes_with_elems(data_block.data.get(), vert_count_u)
			.map_err(|e| e.map_src(<[_]>::to_vec))?;
		Ok(raw_entry_slice.iter().map(|entry_raw| live::VertexEntry {
			position_x: entry_raw.position_x.get(),
			position_y: entry_raw.position_y.get(),
			texture_x: entry_raw.texture_x.get(),
			texture_y: entry_raw.texture_y.get(),
			width: entry_raw.width.get(),
			height: entry_raw.height.get(),
		}).collect())
	}
