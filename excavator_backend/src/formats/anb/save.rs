use super::definition::*;
use hecs::{Entity, World};
use std::marker::PhantomData;
use zerocopy::{FromBytes, IntoBytes, KnownLayout, LE, U32, U64};

pub fn save_from_world(world: &World, entity: Entity) -> Vec<u8> {
	let mut data = Vec::new();
	
	let header = Reservation::<HeaderRaw>::reserve(&mut data);
	let placeholder = Reservation::<Placeholder>::reserve(&mut data);
	let second_pointer = Reservation::<U64<LE>>::reserve(&mut data);
	
	header.write(&mut data, save_header(&world.entity(entity).unwrap().get::<&Header>().unwrap(), &second_pointer));
	second_pointer.write(&mut data, U64::new(placeholder.location as u64));
	placeholder.write(&mut data, Placeholder::new());
	
	data
}

fn save_header(this: &Header, root_node: &Reservation<U64<LE>>) -> HeaderRaw {
	HeaderRaw {
		magic: *b"YCSN",
		unknown_04: U32::new(this.unknown_04),
		unknown_08: U32::new(this.unknown_08),
		unknown_0C: U32::new(this.unknown_0C),
		unknown_10: U32::new(this.unknown_10),
		unknown_14: U32::new(this.unknown_14),
		unknown_18: U32::new(this.unknown_18),
		unknown_1C: U32::new(this.unknown_1C),
		root_node_pointer: U64::new(root_node.pointer_64()),
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
	
	pub fn pointer_64(&self) -> u64 {
		self.location as u64
	}
}

impl<T: FromBytes + IntoBytes + KnownLayout> Reservation<T> {
	pub fn write(self, data: &mut [u8], value: T) {
		*T::mut_from_prefix(&mut data[self.location..]).unwrap().0 = value;
	}
}
