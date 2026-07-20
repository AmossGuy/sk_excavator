use super::{FileView, FileViewEffect};
use std::io::{BufRead, Seek};

use bevy_ecs::{entity::Entity, reflect::AppTypeRegistry, world::World};
use bevy_reflect::{PartialReflect, ReflectMut, structs::Struct};

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
		entity_ui(ui, &mut self.ecs_world, self.root);
		
		if ui.button("Save as (WIP)").clicked() {
			// another case of using blocking thingy because i'm lazy
			if let Some(path) = rfd::FileDialog::new().save_file() {
				let data = excavator_backend::formats::anb::save_from_world(&self.ecs_world, self.root);
				std::fs::write(path, data).unwrap();
			}
		}
		
		FileViewEffect::default()
	}
}

fn entity_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
	let component_list = {
		let entity_ref = world.get_entity(entity)
			.expect("entity should be spawned");
		entity_ref.archetype().components().to_vec()
	};
	
	for component_id in component_list {
		if let Some(info) = world.components().get_info(component_id) && let Some(type_id) = info.type_id() {
			let mut reflect = world.get_reflect_mut(entity, type_id)
				.expect("entity should have the components of its archetype");
			
			match reflect.reflect_mut() {
				ReflectMut::Struct(thing) => { struct_ui(ui, thing); },
				_ => {},
			}
		}
	}
}

fn struct_ui(ui: &mut egui::Ui, thing: &mut dyn Struct) {
	ui.heading(thing.reflect_type_ident().unwrap_or("(anonymous type)"));
	
	egui::Grid::new("struct").show(ui, |ui| {
		for i in 0..thing.field_len() {
			let name = thing.name_at(i).unwrap();
			ui.label(name);
			
			let value = thing.field_at_mut(i).unwrap();
			match lookup_value_widget(value) {
				None => { ui.label("(unknown type)"); },
				Some(value) => { value.value_widget(ui); },
			};
			
			ui.end_row();
		}
	});
}

fn lookup_value_widget<'a>(value: &'a mut dyn PartialReflect) -> Option<&'a mut dyn ValueWidget> {
	if let Some(number) = value.try_downcast_mut::<u32>() {
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
	fn value_widget(&mut self, ui: &mut egui::Ui);
}


impl ValueWidget for u32 {
	fn value_widget(&mut self, ui: &mut egui::Ui) {
		ui.add(egui::DragValue::new(self));
	}
}
