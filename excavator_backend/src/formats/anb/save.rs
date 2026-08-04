use super::{def_live as live, def_raw as raw};

use hecs::{Entity, World};
use hecs_hierarchy::Hierarchy;
use std::marker::PhantomData;
use zerocopy::{FromBytes, IntoBytes, KnownLayout, LE, U32, U64};

pub fn save_from_world(world: &World, root_entity: Entity) -> anyhow::Result<Vec<u8>> {
	let mut data = Vec::<u8>::new();
	
	let header_reser = Reservation::<raw::Header>::reserve(&mut data);
	let header_component = world.get::<&live::Header>(root_entity)?;
	header_reser.write(&mut data, save_header(&header_component));
	
	let node_entity = world.get::<&hecs_hierarchy::Parent<()>>(root_entity)?.first_child(world)?;
	save_node(world, node_entity, &mut data, true)?;
	
	Ok(data)
}

fn save_node(world: &World, node_entity: Entity, output: &mut Vec<u8>, alt: bool) -> anyhow::Result<usize> {
	// Before going any further, we need to reserve the spot this node will be saved to.
	// However, we won't actually write it until later, when we have pointers to all of its children prepared.
	let node_reser = Reservation::<raw::NodeCommon>::reserve(output);
	let node_reser_location = node_reser.location;
	
	let node_kind = save_node_attachment(world, node_entity, output)?;
	
	// Recursively save all of this node's children
	let (child_count, child_array_pointer) = if alt {
		save_children_nodes_alt(world, node_entity, output)?
	} else {
		save_children_nodes(world, node_entity, output)?
	};
	
	// Finally, write the parent, including the pointer to the children pointers.
	node_reser.write(output, save_node_common(node_kind, child_count as u32, child_array_pointer as u64));
	
	Ok(node_reser_location)
}

fn save_children_nodes(world: &World, parent: Entity, output: &mut Vec<u8>) -> anyhow::Result<(usize, usize)> {
	// step one: get iterator
	let children_iter = world.children::<()>(parent);
	
	// step two: save each child, recursing for the children's children
	let mut child_pointers = Vec::<usize>::new();
	for child_entity in children_iter {
		let pointer = save_node(world, child_entity, output, false)?;
		child_pointers.push(pointer);
	}
	
	// step three: write pointers to children
	let child_array_pointer = output.len();
	for &pointer in child_pointers.iter() {
		output.extend(U64::<LE>::new(pointer as u64).to_bytes());
	}
	
	if child_pointers.is_empty() {
		Ok((0, 0))
	} else {
		Ok((child_pointers.len(), child_array_pointer))
	}
}

// the only difference this has from save_children_nodes is that it writes things in a different order, to imitate a quirk in the vanilla files. it just takes some finagling to do that
fn save_children_nodes_alt(world: &World, parent: Entity, output: &mut Vec<u8>) -> anyhow::Result<(usize, usize)> {
	// step one: get iterator
	let children_iter = world.children::<()>(parent);
	
	// step two alt: save each child, but save their own children for later
	let mut child_things = Vec::<(Reservation<raw::NodeCommon>, Entity, u32)>::new();
	for child_entity in children_iter {
		// these two lines are basically the first half of save_node alt version
		let child_reser = Reservation::<raw::NodeCommon>::reserve(output);
		let child_kind = save_node_attachment(world, child_entity, output)?;
		
		child_things.push((child_reser, child_entity, child_kind));
	}
	
	// step three: write pointers to children
	let child_array_pointer = output.len();
	for (child_reser, _, _) in child_things.iter() {
		output.extend(U64::<LE>::new(child_reser.location as u64).to_bytes());
	}
	
	// extra step: do the recursion we saved for later (it's later)
	let child_things_len = child_things.len();
	for (child_reser, child_entity, child_kind) in child_things {
		let (w_child_count, w_child_array_pointer) = save_children_nodes(world, child_entity, output)?;
		child_reser.write(output, save_node_common(child_kind, w_child_count as u32, w_child_array_pointer as u64));
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

fn save_node_attachment(world: &World, node_entity: Entity, output: &mut Vec<u8>) -> anyhow::Result<u32> {
	let node_component = world.get::<&live::Node>(node_entity)?;
	match &*node_component {
		live::Node::Base => {},
		live::Node::Texture(node_live) => {
			output.extend(raw::NodeTexture {
				width: node_live.width.into(),
				height: node_live.height.into(),
				flags: node_live.flags.into(),
				padding: node_live.padding.into(),
				data_pointer: PLACEHOLDER_POINTER,
			}.as_bytes());
		},
		live::Node::Vertex(node_live) => {
			output.extend(raw::NodeVertex {
				vert_count: node_live.vert_count.into(),
				flags: node_live.flags.into(),
				data_pointer: PLACEHOLDER_POINTER,
			}.as_bytes());
		},
		live::Node::Meta => {},
		live::Node::MetaScalar(node_live) => {
			output.extend(raw::NodeMetaScalar {
				unk_1: node_live.unk_1.into(),
				unk_2: node_live.unk_2.into(),
			}.as_bytes());
		},
		live::Node::MetaPoint(node_live) => {
			output.extend(raw::NodeMetaPoint {
				x: node_live.x.into(),
				y: node_live.y.into(),
				z: node_live.z.into(),
				padding: node_live.padding.into(),
			}.as_bytes());
		},
		live::Node::MetaAnchor(node_live) => {
			output.extend(raw::NodeMetaAnchor {
				x: node_live.x.into(),
				y: node_live.y.into(),
				z: node_live.z.into(),
				angle: node_live.angle.into(),
			}.as_bytes());
		},
		live::Node::MetaRect(node_live) => {
			output.extend(raw::NodeMetaRect {
				center_x: node_live.center_x.into(),
				center_y: node_live.center_y.into(),
				center_z: node_live.center_z.into(),
				extents_x: node_live.extents_x.into(),
				extents_y: node_live.extents_y.into(),
				extents_z: node_live.extents_z.into(),
				angle: node_live.angle.into(),
				padding: node_live.padding.into(),
			}.as_bytes());
		},
		live::Node::MetaString(node_live) => {
			output.extend(raw::NodeMetaString {
				string_length: node_live.string_length.into(),
				padding: node_live.padding.into(),
				string_offset: PLACEHOLDER_POINTER,
			}.as_bytes());
		},
		live::Node::MetaTable(_node_live) => {
			output.extend(raw::NodeMetaTable {
				hashname_pointer: PLACEHOLDER_POINTER,
			}.as_bytes());
		},
		live::Node::Frame(node_live) => {
			output.extend(raw::NodeFrame {
				min_x: node_live.min_x.into(),
				max_x: node_live.max_x.into(),
				min_y: node_live.min_y.into(),
				max_y: node_live.max_y.into(),
			}.as_bytes());
		},
		live::Node::SequenceFrame(node_live) => {
			output.extend(raw::NodeSequenceFrame {
				frame: node_live.frame.into(),
				delay: node_live.delay.into(),
			}.as_bytes());
		},
		live::Node::Sequence(node_live) => {
			output.extend(raw::NodeSequence {
				hashname: node_live.hashname.into(),
				frame_count: node_live.frame_count.into(),
			}.as_bytes());
		},
		live::Node::Animation(node_live) => {
			output.extend(raw::NodeAnimation {
				sequence_count: node_live.sequence_count.into(),
				frame_count: node_live.frame_count.into(),
				single_texture: node_live.single_texture.into(),
				palette_index: node_live.palette_index.into(),
				hashname_pointer: PLACEHOLDER_POINTER,
			}.as_bytes());
		},
		live::Node::UnknownKind(kind) => {
			anyhow::bail!("unknown kind node ({})", kind);
		},
	}
	Ok(node_component.kind())
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
		let (destination, _) = T::mut_from_prefix(&mut data[self.location..])
			.expect("reservation should be within the bounds of the data");
		*destination = value;
	}
}
