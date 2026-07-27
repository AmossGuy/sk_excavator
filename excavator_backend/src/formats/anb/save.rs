use super::{def_live as live, def_raw as raw};

use hecs::{Entity, World};
use hecs_hierarchy::Hierarchy;
use std::marker::PhantomData;
use zerocopy::{FromBytes, IntoBytes, KnownLayout, LE, U32, U64};

pub fn save_from_world(world: &World, root_entity: Entity) -> Vec<u8> {
	let mut data = Vec::<u8>::new();
	
	let header_reser = Reservation::<raw::Header>::reserve(&mut data);
	let header_component = world.get::<&live::Header>(root_entity).unwrap();
	header_reser.write(&mut data, save_header(&header_component));
	
	let node_entity = world.get::<&hecs_hierarchy::Parent<()>>(root_entity).unwrap().first_child(world).unwrap();
	save_node_recursively(world, node_entity, &mut data);
	
	data
}

fn save_node_recursively(world: &World, node_entity: Entity, output: &mut Vec<u8>) -> usize {
	// Before going any further, we need to reserve the spot this node will be saved to.
	// However, we won't actually write it until later, when we have pointers to all of its children prepared.
	let node_reser = Reservation::<raw::NodeCommon>::reserve(output);
	let node_reser_location = node_reser.location;
	
	// make a placeholder for the node's actual data
	let node_component = world.get::<&live::Node>(node_entity).unwrap();
	output.extend(std::iter::repeat(0xAA).take(kind_data_bytes(node_component.kind())));
	
	let (child_count, child_array_pointer) = save_children_nodes(world, node_entity, output);
	
	// Finally, write the parent, including the pointer to the children pointers.
	node_reser.write(output, save_node_common(&node_component, child_count, child_array_pointer));
	
	node_reser_location
}

fn save_children_nodes(world: &World, parent: Entity, output: &mut Vec<u8>) -> (usize, usize) {
	// step one: get iterator
	let children_iter = world.children::<()>(parent);
	
	// step two: save each child, recursing for the children's children
	let mut child_pointers = Vec::<usize>::new();
	for child in children_iter {
		let pointer = save_node_recursively(world, child, output);
		child_pointers.push(pointer);
	}
	
	// step three: write pointers to children
	let child_array_pointer = output.len();
	for pointer in child_pointers.iter().copied() {
		output.extend(U64::<LE>::new(pointer as u64).to_bytes());
	}
	
	if child_pointers.is_empty() {
		(0, 0)
	} else {
		(child_pointers.len(), child_array_pointer)
	}
}

fn kind_data_bytes(kind: u32) -> usize {
	match kind {
		0 => 0,
		1 => 24,
		2 => 16,
		3 => 0,
		4 => 8,
		5 => 16,
		6 => 16,
		7 => 32,
		8 => 16,
		9 => 8,
		10 => 16,
		11 => 8,
		12 => 8,
		13 => 24,
		_ => 0,
	}
}

fn save_header(this: &live::Header) -> raw::Header {
	raw::Header {
		magic: *b"YCSN",
		unknown_04: U32::new(this.unknown_04),
		unknown_08: U32::new(this.unknown_08),
		unknown_0C: U32::new(this.unknown_0C),
		unknown_10: U32::new(this.unknown_10),
		unknown_14: U32::new(this.unknown_14),
	}
}

fn save_node_common(this: &live::Node, child_count: usize, child_array_pointer: usize) -> raw::NodeCommon {
	raw::NodeCommon {
		kind: U32::new(this.kind()),
		child_count: U32::new(child_count as u32),
		child_array_pointer: U64::new(child_array_pointer as u64),
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
		*T::mut_from_prefix(&mut data[self.location..]).unwrap().0 = value;
	}
}
