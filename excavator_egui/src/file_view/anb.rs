use super::{FileView, FileViewEffect};
use std::io::{BufRead, Seek};
use std::ops::DerefMut;

use hecs::{Entity, World};
use hecs_hierarchy::Hierarchy;

use excavator_backend::formats::{
	DropdownRenderer, EditableData, EditableDataRenderer,
	anb::{Header, Node},
};

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
}

impl AnbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> anyhow::Result<Self> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes)?;
		
		let mut ecs_world = World::new();
		let root = excavator_backend::formats::anb::load_from_bytes(&bytes, &mut ecs_world)?;
		
		Ok(Self { ecs_world, root })
	}
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		if ui.button("Save as (WIP)").clicked() {
			// another case of using blocking thingy because i'm lazy
			if let Some(path) = rfd::FileDialog::new().save_file() {
				let data = excavator_backend::formats::anb::save_from_world(&self.ecs_world, self.root);
				std::fs::write(path, data).unwrap();
			}
		}
		
		ui.separator();
		
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root);
		});
		
		FileViewEffect::default()
	}
}

fn entity_tree_ui(ui: &mut egui::Ui, world: &mut World, root: Entity) {
	egui::containers::Frame::group(ui.style()).show(ui, |ui| {
		entity_ui(ui, world, root);
	});
	
	let indent_frame = egui::containers::Frame::NONE.outer_margin(egui::Margin {
		left: 20,
		..egui::Margin::ZERO
	});
	
	// I think I might need to futz with my hecs_hierarchy fork more to avoid this collect
	let children = world.children::<()>(root).collect::<Vec<_>>();
	indent_frame.show(ui, |ui| {
		for (i, child) in children.into_iter().enumerate() {
			ui.push_id(i, |ui| {
				entity_tree_ui(ui, world, child);
			});
		}
	});
}

fn entity_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
	let entity_ref = world.entity(entity)
		.expect("entity should be spawned");
	
	if let Some(mut header_component) = entity_ref.get::<&mut Header>() {
		struct_ui(ui, header_component.deref_mut());
	}
	if let Some(mut node_component) = entity_ref.get::<&mut Node>() {
		struct_ui(ui, node_component.deref_mut());
	}
}

fn struct_ui(ui: &mut egui::Ui, thing: &mut impl EditableData) {
	let struct_name = thing.struct_name();
	ui.heading(struct_name);
	
	egui::Grid::new(struct_name).show(ui, |ui| {
		thing.display(EguiDataRenderer { ui });
	});
}

struct EguiDataRenderer<'a> {
	ui: &'a mut egui::Ui,
}

impl EditableDataRenderer for EguiDataRenderer<'_> {
	type Dropdown<'a> = EguiDropdownRenderer<'a>;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(Self::Dropdown<'_>)) {
		let ui = &mut self.ui;
		ui.label(name);
		egui::ComboBox::from_id_salt(name)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				contents(EguiDropdownRenderer { ui })
			});
		ui.end_row();
	}
	
	fn field_f32(&mut self, name: &str, value: &mut f32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u32(&mut self, name: &str, value: &mut u32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
}

struct EguiDropdownRenderer<'a> {
	ui: &'a mut egui::Ui,
}

impl DropdownRenderer for EguiDropdownRenderer<'_> {
	fn choice(&mut self, name: &str, selected: bool) -> bool {
		let ui = &mut self.ui;
		ui.selectable_label(selected, name).clicked()
	}
}
