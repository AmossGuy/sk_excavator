use super::{FileView, FileViewEffect};
use std::io::{BufRead, Seek};

use bevy_ecs::{entity::Entity, reflect::AppTypeRegistry, world::World};
use bevy_reflect::{PartialReflect, ReflectRef, TypeRegistry, structs::Struct};

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
}

impl AnbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut ecs_world = bevy_ecs::world::World::new();
		ecs_world.insert_resource(AppTypeRegistry::new_with_derived_types());
		
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).unwrap();
		let root = excavator_backend::formats::anb::load_from_bytes(&bytes, &mut ecs_world.commands());
		ecs_world.flush();
		
		Self { ecs_world, root }
	}
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		entity_ui(ui, &self.ecs_world, self.root);
		FileViewEffect::default()
	}
}

fn entity_ui(ui: &mut egui::Ui, world: &World, entity: Entity) {
	let entity_ref = world.get_entity(entity)
		.expect("entity should be spawned");
	let component_list = entity_ref.archetype().components();
	
	for &component_id in component_list {
		if let Some(info) = world.components().get_info(component_id) && let Some(type_id) = info.type_id() {
			let reflect = world.get_reflect(entity, type_id)
				.expect("entity should have the components of its archetype");
			let registry = world.get_resource::<AppTypeRegistry>()
				.expect("type registry should exist")
				.read();
			
			match reflect.reflect_ref() {
				ReflectRef::Struct(thing) => { struct_ui(ui, thing, &registry); },
				_ => {},
			}
		}
	}
}

fn struct_ui(ui: &mut egui::Ui, thing: &dyn Struct, registry: &TypeRegistry) {
	ui.heading(thing.reflect_type_ident().unwrap_or("(anonymous type)"));
	
	egui::Grid::new("struct").show(ui, |ui| {
		for (name, value) in thing.iter_fields() {
			ui.label(name);
			match lookup_value_widget(value, registry) {
				None => { ui.label("(unknown type)"); },
				Some(value) => { value.value_widget(ui); },
			};
			ui.end_row();
		}
	});
}

fn lookup_value_widget<'a>(value: &'a dyn PartialReflect, _registery: &TypeRegistry) -> Option<&'a dyn ValueWidget> {
	if let Some(number) = value.try_downcast_ref::<u32>() {
		Some(number)
	} else {
		None
	}
}

// Reflecting traits on primitives is slightly too annoying for me to bother at the moment
/*
fn lookup_value_widget<'a>(value: &'a dyn PartialReflect, registery: &TypeRegistry) -> Option<&'a dyn ValueWidget> {
	let type_data = registery.get_type_data::<ReflectValueWidget>(value.type_id())?;
	value.try_as_reflect().and_then(|v| type_data.get(v))
}
*/

// #[reflect_trait]
trait ValueWidget {
	fn value_widget(&self, ui: &mut egui::Ui);
}


impl ValueWidget for u32 {
	fn value_widget(&self, ui: &mut egui::Ui) {
		let mut fake = self.clone();
		ui.add(egui::DragValue::new(&mut fake));
	}
}
