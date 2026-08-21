use bevy_ecs::{prelude::*, component::Mutable};
use std::{mem, ops::DerefMut};

#[derive(Resource, Default)]
pub struct UndoResource {
	commands: undo_2::Commands<Box<dyn UndoEntry>>,
}

pub trait UndoEntry: Send + Sync + 'static {
	fn undo(&self, world: &mut World);
	fn redo(&self, world: &mut World);
}

pub fn undoable_replace_component<T>(entity: Entity, new: T) -> impl Command where
	T: Component<Mutability = Mutable> + Clone,
{
	move |world: &mut World| {
		let u = UndoableReplaceComponent::build(world.entity_mut(entity).into(), new);
		world.get_resource_or_init::<UndoResource>().commands.push(Box::new(u));
	}
}

struct UndoableReplaceComponent<T> {
	entity: Entity,
	old: T,
	new: T,
}

impl<T> UndoableReplaceComponent<T>  where
	T: Component<Mutability = Mutable> + Clone,
{
	fn build(mut entity_mut: EntityMut<'_>, new: T) -> Self {
		let mut component_mut = entity_mut.get_components_mut::<&mut T>().unwrap();
		let old = mem::replace(component_mut.deref_mut(), new.clone());
		Self {
			entity: entity_mut.id(),
			old, new,
		}
	}
}

impl<T> UndoEntry for UndoableReplaceComponent<T> where
	T: Component<Mutability = Mutable> + Clone,
{
	fn undo(&self, world: &mut World) {
		let mut entity_mut = world.entity_mut(self.entity);
		let mut component_mut = entity_mut.get_components_mut::<&mut T>().unwrap();
		*component_mut = self.old.clone();
	}
	
	fn redo(&self, world: &mut World) {
		let mut entity_mut = world.entity_mut(self.entity);
		let mut component_mut = entity_mut.get_components_mut::<&mut T>().unwrap();
		*component_mut = self.new.clone();
	}
}
