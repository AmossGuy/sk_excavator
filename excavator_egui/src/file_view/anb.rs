use super::{FileView, FileViewEffect};
use std::any::TypeId;
use std::io::{BufRead, Seek};
use std::ops::DerefMut;

use hecs::{Entity, World};

use excavator_backend::formats::anb::{EditableStruct, Header};

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
}

impl AnbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut ecs_world = World::new();
		
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).unwrap();
		let root = excavator_backend::formats::anb::load_from_bytes(&bytes, &mut ecs_world);
		
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
	let entity_ref = world.entity(entity)
		.expect("entity should be spawned");
	let component_list = entity_ref.component_types().collect::<Vec<_>>();
	
	for type_id in component_list {
		if type_id == TypeId::of::<Header>() {
			struct_ui(ui, entity_ref.get::<&mut Header>().unwrap().deref_mut());
		} else {
			ui.label("look whatever i'm working on it");
		}
	}
}

fn struct_ui(ui: &mut egui::Ui, thing: &mut dyn EditableStruct) {
	ui.heading(thing.struct_name());
	
	egui::Grid::new("struct").show(ui, |ui| {
		for i in 0..thing.number_of_fields() {
			let name = thing.field_name(i).unwrap();
			ui.label(name);
			
			let value = thing.field_mut(i).unwrap();
			match lookup_value_widget(value) {
				None => { ui.label("(unknown type)"); },
				Some(value) => { value.value_widget(ui); },
			};
			
			ui.end_row();
		}
	});
}

fn lookup_value_widget<'a>(value: &'a mut dyn std::any::Any) -> Option<&'a mut dyn ValueWidget> {
	if let Some(number) = value.downcast_mut::<u32>() {
		Some(number)
	} else {
		None
	}
}

trait ValueWidget {
	fn value_widget(&mut self, ui: &mut egui::Ui);
}


impl ValueWidget for u32 {
	fn value_widget(&mut self, ui: &mut egui::Ui) {
		ui.add(egui::DragValue::new(self));
	}
}
