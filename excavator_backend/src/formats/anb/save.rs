// use super::{def_live as live, def_raw as raw};

// use std::marker::PhantomData;
// use zerocopy::{FromBytes, IntoBytes, KnownLayout, Immutable, LE, U32, U64};

/*
pub fn save_from_world(world: &World, root_entity: Entity) -> anyhow::Result<Vec<u8>> {
	let mut saver = Saver::new(world);
	
	let header_reser = saver.reserve::<raw::Header>();
	let header_component = world.get::<live::Header>(root_entity)
		.expect("root entity should exist and have header component");
	header_reser.write(&mut saver.output, save_header(&header_component));
	
	let node_entity = saver.world.get::<Children>(root_entity).unwrap()[0];
	save_node(&mut saver, node_entity, true)?;
	
	save_queued_blocks(&mut saver)?;
	
	Ok(saver.output)
}

fn save_node(saver: &mut Saver<'_>, node_entity: Entity, alt: bool) -> anyhow::Result<usize> {
	// Before going any further, we need to reserve the spot this node will be saved to.
	// However, we won't actually write it until later, when we have pointers to all of its children prepared.
	let node_reser = saver.reserve::<raw::NodeCommon>();
	let node_reser_location = node_reser.location;
	
	let node_kind = save_node_attachment(saver, node_entity)?;
	
	// Recursively save all of this node's children
	let (child_count, child_array_pointer) = if alt {
		save_children_nodes_alt(saver, node_entity)?
	} else {
		save_children_nodes(saver, node_entity)?
	};
	
	// Finally, write the parent, including the pointer to the children pointers.
	node_reser.write(&mut saver.output, save_node_common(node_kind, child_count as u32, child_array_pointer as u64));
	
	Ok(node_reser_location)
}

fn save_children_nodes(saver: &mut Saver<'_>, parent: Entity) -> anyhow::Result<(usize, usize)> {
	// step one: get iterator
	let children_iter = saver.world.get::<Children>(parent)
		.map(|c| &c[..]).unwrap_or(&[]).into_iter().copied();
	
	// step two: save each child, recursing for the children's children
	let mut child_pointers = Vec::<usize>::new();
	for child_entity in children_iter {
		let pointer = save_node(saver, child_entity, false)?;
		child_pointers.push(pointer);
	}
	
	// step three: write pointers to children
	let child_array_pointer = saver.output.len();
	for &pointer in child_pointers.iter() {
		saver.push(U64::<LE>::new(pointer as u64));
	}
	
	if child_pointers.is_empty() {
		Ok((0, 0))
	} else {
		Ok((child_pointers.len(), child_array_pointer))
	}
}

// the only difference this has from save_children_nodes is that it writes things in a different order, to imitate a quirk in the vanilla files. it just takes some finagling to do that
fn save_children_nodes_alt(saver: &mut Saver<'_>, parent: Entity) -> anyhow::Result<(usize, usize)> {
	// step one: get iterator
	let children_iter = saver.world.get::<Children>(parent)
		.map(|c| &c[..]).unwrap_or(&[]).into_iter().copied();
	
	// step two alt: save each child, but save their own children for later
	let mut child_things = Vec::<(Reservation<raw::NodeCommon>, Entity, u32)>::new();
	for child_entity in children_iter {
		// these two lines are basically the first half of save_node alt version
		let child_reser = saver.reserve::<raw::NodeCommon>();
		let child_kind = save_node_attachment(saver, child_entity)?;
		
		child_things.push((child_reser, child_entity, child_kind));
	}
	
	// step three: write pointers to children
	let child_array_pointer = saver.output.len();
	for (child_reser, _, _) in child_things.iter() {
		saver.push(U64::<LE>::new(child_reser.location as u64));
	}
	
	// extra step: do the recursion we saved for later (it's later)
	let child_things_len = child_things.len();
	for (child_reser, child_entity, child_kind) in child_things {
		let (w_child_count, w_child_array_pointer) = save_children_nodes(saver, child_entity)?;
		child_reser.write(&mut saver.output, save_node_common(child_kind, w_child_count as u32, w_child_array_pointer as u64));
	}
	
	if child_things_len == 0 {
		Ok((0, 0))
	} else {
		Ok((child_things_len, child_array_pointer))
	}
}

const PLACEHOLDER_POINTER: U64<LE> = U64::new(0xAA_AA_AA_AA_AA_AA_AA_AA);

fn save_header(this: &live::Header) -> raw::Header {
	raw::Header {
		magic: *b"YCSN",
		fixup: U32::new(this.fixup),
		version: U32::new(this.version),
		padding_a: U32::new(this.padding_a),
		padding_b: U32::new(this.padding_b),
		padding_c: U32::new(this.padding_c),
	}
}

fn save_node_common(node_kind: u32, child_count: u32, child_array_pointer: u64) -> raw::NodeCommon {
	raw::NodeCommon {
		kind: U32::new(node_kind),
		child_count: U32::new(child_count),
		child_array_pointer: U64::new(child_array_pointer),
	}
}

fn save_node_attachment(saver: &mut Saver<'_>, node_entity: Entity) -> anyhow::Result<u32> {
	let node_component = saver.world.get::<live::Node>(node_entity)
		.expect("entity should exist and have node component");
	match &*node_component {
		live::Node::Base => {},
		live::Node::Texture(node_live) => {
			let reser = saver.reserve::<raw::NodeTexture>();
			let node_raw = raw::NodeTexture {
				width: node_live.width.into(),
				height: node_live.height.into(),
				flags: node_live.flags.into(),
				padding: node_live.padding.into(),
				data_pointer: PLACEHOLDER_POINTER,
			};
			let data_block = node_live.data_block.clone();
			saver.deferred_blocks.push(DeferredBlock {
				data_block,
				node: DeferredBlockNode::Texture {
					node_raw, reser,
				},
			});
		},
		live::Node::Vertex(node_live) => {
			let reser = saver.reserve::<raw::NodeVertex>();
			let node_raw = raw::NodeVertex {
				vert_count: node_live.vert_count.into(),
				flags: node_live.flags.into(),
				data_pointer: PLACEHOLDER_POINTER,
			};
			let data_block = node_live.data_block.clone();
			saver.deferred_blocks.push(DeferredBlock {
				data_block,
				node: DeferredBlockNode::Vertex {
					node_raw, reser,
				},
			});
		},
		live::Node::Meta => {},
		live::Node::MetaScalar(node_live) => {
			saver.push(raw::NodeMetaScalar {
				unk_1: node_live.unk_1.into(),
				unk_2: node_live.unk_2.into(),
			});
		},
		live::Node::MetaPoint(node_live) => {
			saver.push(raw::NodeMetaPoint {
				x: node_live.x.into(),
				y: node_live.y.into(),
				z: node_live.z.into(),
				padding: node_live.padding.into(),
			});
		},
		live::Node::MetaAnchor(node_live) => {
			saver.push(raw::NodeMetaAnchor {
				x: node_live.x.into(),
				y: node_live.y.into(),
				z: node_live.z.into(),
				angle: node_live.angle.into(),
			});
		},
		live::Node::MetaRect(node_live) => {
			saver.push(raw::NodeMetaRect {
				center_x: node_live.center_x.into(),
				center_y: node_live.center_y.into(),
				center_z: node_live.center_z.into(),
				extents_x: node_live.extents_x.into(),
				extents_y: node_live.extents_y.into(),
				extents_z: node_live.extents_z.into(),
				angle: node_live.angle.into(),
				padding: node_live.padding.into(),
			});
		},
		live::Node::MetaString(node_live) => {
			let reser = saver.reserve::<raw::NodeMetaString>();
			let node_raw = raw::NodeMetaString {
				string_length: node_live.string_length.into(),
				padding: node_live.padding.into(),
				string_offset: PLACEHOLDER_POINTER,
			};
			let data_block = node_live.data_block.clone();
			saver.deferred_blocks.push(DeferredBlock {
				data_block,
				node: DeferredBlockNode::MetaString {
					node_raw, reser,
				},
			});
		},
		live::Node::MetaTable(node_live) => {
			let reser = saver.reserve::<raw::NodeMetaTable>();
			let node_raw = raw::NodeMetaTable {
				hashname_pointer: PLACEHOLDER_POINTER,
			};
			let data_block = node_live.data_block.clone();
			saver.deferred_blocks.push(DeferredBlock {
				data_block,
				node: DeferredBlockNode::MetaTable {
					node_raw, reser,
				},
			});
		},
		live::Node::Frame(node_live) => {
			saver.push(raw::NodeFrame {
				min_x: node_live.min_x.into(),
				max_x: node_live.max_x.into(),
				min_y: node_live.min_y.into(),
				max_y: node_live.max_y.into(),
			});
		},
		live::Node::SequenceFrame(node_live) => {
			saver.push(raw::NodeSequenceFrame {
				frame: node_live.frame.into(),
				delay: node_live.delay.into(),
			});
		},
		live::Node::Sequence(node_live) => {
			saver.push(raw::NodeSequence {
				hashname: node_live.hashname.into(),
				frame_count: node_live.frame_count.into(),
			});
		},
		live::Node::Animation(node_live) => {
			let reser = saver.reserve::<raw::NodeAnimation>();
			let node_raw = raw::NodeAnimation {
				sequence_count: node_live.sequence_count.into(),
				frame_count: node_live.frame_count.into(),
				single_texture: node_live.single_texture.into(),
				palette_index: node_live.palette_index.into(),
				hashname_pointer: PLACEHOLDER_POINTER,
			};
			let data_block = node_live.data_block.clone();
			saver.deferred_blocks.push(DeferredBlock {
				data_block,
				node: DeferredBlockNode::Animation {
					node_raw, reser,
				},
			});
		},
		live::Node::UnknownKind(kind) => {
			anyhow::bail!("unknown kind node ({})", kind);
		},
	}
	Ok(node_component.kind())
}

fn save_queued_blocks(saver: &mut Saver<'_>) -> anyhow::Result<()> {
	fn actual_block_save(saver: &mut Saver<'_>, entry: &DeferredBlock) -> Option<usize> {
		let data_block = entry.data_block.as_ref()?;
		let flags = data_block.flags;
		let data = *data_block.data.get();
		
		let datablock_pointer = saver.output.len();
		saver.push(raw::DataBlockHeader {
			data_size: (data.len() as u32).into(),
			flags: flags.into(),
		});
		saver.output.extend(data);
		saver.pad_to_alignment(8);
		
		Some(datablock_pointer)
	}
	
	for entry in std::mem::take(&mut saver.deferred_blocks) {
		let Some(datablock_pointer) = actual_block_save(saver, &entry) else {
			continue;
		};
		
		match entry.node {
			DeferredBlockNode::Texture { reser, mut node_raw, .. } => {
				node_raw.data_pointer = U64::new(datablock_pointer as u64);
				reser.write(&mut saver.output, node_raw);
			},
			DeferredBlockNode::Vertex { reser, mut node_raw, .. } => {
				node_raw.data_pointer = U64::new(datablock_pointer as u64);
				reser.write(&mut saver.output, node_raw);
			},
			DeferredBlockNode::MetaString { reser, mut node_raw, .. } => {
				node_raw.string_offset = U64::new(datablock_pointer as u64);
				reser.write(&mut saver.output, node_raw);
			},
			DeferredBlockNode::MetaTable { reser, mut node_raw, .. } => {
				node_raw.hashname_pointer = U64::new(datablock_pointer as u64);
				reser.write(&mut saver.output, node_raw);
			},
			DeferredBlockNode::Animation { reser, mut node_raw, .. } => {
				node_raw.hashname_pointer = U64::new(datablock_pointer as u64);
				reser.write(&mut saver.output, node_raw);
			},
		}
	}
	Ok(())
}

struct Saver<'world> {
	world: &'world World,
	output: Vec<u8>,
	deferred_blocks: Vec<DeferredBlock>,
}

impl<'world> Saver<'world> {
	fn new(world: &'world World) -> Self {
		Self {
			world,
			output: Vec::new(),
			deferred_blocks: Vec::new(),
		}
	}
	
	fn push<T: IntoBytes + Immutable>(&mut self, value: T) {
		self.output.extend(value.as_bytes());
	}
	
	fn reserve<T>(&mut self) -> Reservation<T> {
		Reservation::reserve(&mut self.output)
	}
	
	fn pad_to_alignment(&mut self, alignment: usize) {
		let new_len = self.output.len().next_multiple_of(alignment);
		self.output.resize(new_len, 0);
	}
}

#[must_use]
struct Reservation<T> {
	location: usize,
	phantom: PhantomData<fn(T)>,
}

impl<T> Reservation<T> {
	pub fn reserve(data: &mut Vec<u8>) -> Self {
		let location = data.len();
		data.extend(std::iter::repeat(0).take(std::mem::size_of::<T>()));
		Self { location, phantom: PhantomData }
	}
}

impl<T: FromBytes + IntoBytes + KnownLayout> Reservation<T> {
	pub fn write(self, data: &mut [u8], value: T) {
		let offset_data = data.get_mut(self.location..)
			.expect("reservation should be within the bounds of the data");
		let (destination, _) = T::mut_from_prefix(offset_data)
			.expect("reservation should be within the bounds of the data");
		*destination = value;
	}
}

struct DeferredBlock {
	data_block: Option<live::DataBlock>,
	node: DeferredBlockNode,
}

enum DeferredBlockNode {
	Texture {
		reser: Reservation<raw::NodeTexture>,
		node_raw: raw::NodeTexture,
	},
	Vertex {
		reser: Reservation<raw::NodeVertex>,
		node_raw: raw::NodeVertex,
	},
	MetaString {
		reser: Reservation<raw::NodeMetaString>,
		node_raw: raw::NodeMetaString,
	},
	MetaTable {
		reser: Reservation<raw::NodeMetaTable>,
		node_raw: raw::NodeMetaTable,
	},
	Animation {
		reser: Reservation<raw::NodeAnimation>,
		node_raw: raw::NodeAnimation,
	},
}
*/
