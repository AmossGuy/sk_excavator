use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use crate::file_view::common::tree::{entity_tree_ui, EntityTreeCallbacks};

use std::sync::Arc;
use yoke::Yoke;

use bevy_ecs::{
	component::Component, entity::Entity, system::Commands,
	world::{EntityRef, World},
};

use excavator_backend::formats::anb::def_live as anb;
use excavator_backend::formats::common::undo::{UndoEntry, UndoResource, undoable_replace_component};
// use excavator_backend::formats::wflz;

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
	
	save_message: String,
}

impl AnbFileView {
	pub fn parse(file_contents: Vec<u8>) -> anyhow::Result<Self> {
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
		
		let mut ecs_world = World::new();
		let root = excavator_backend::formats::anb::load_from_bytes(&yoke_bytes, &mut ecs_world)?;
		ecs_world.init_resource::<UndoResource>();
		
		let save_message = String::new();
		
		Ok(Self { ecs_world, root, save_message })
	}
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &ExcavatorContext) {
		if ui.button("Save as (WIP)").clicked() {
			// another case of using blocking thingy because i'm lazy
			if let Some(path) = rfd::FileDialog::new().save_file() {
				match excavator_backend::formats::anb::save_from_world(&self.ecs_world, self.root) {
					Err(e) => { self.save_message = format!("error while serializing: {}", e); },
					Ok(data) => match std::fs::write(path, data) {
						Err(e) => { self.save_message = format!("error while writing file: {}", e); },
						Ok(()) => { self.save_message = "Success".to_string(); },
					},
				}
			}
		}
		if ui.button("Undo").clicked() {
			self.ecs_world.resource_scope::<UndoResource, _>(|world, mut undo| {
				for action in undo.commands.undo() {
					interpret_action(world, action);
				}
			});
		}
		if ui.button("Redo").clicked() {
			self.ecs_world.resource_scope::<UndoResource, _>(|world, mut undo| {
				for action in undo.commands.redo() {
					interpret_action(world, action);
				}
			});
		}
		ui.label(&self.save_message);
		
		ui.separator();
		
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root, &TREE_CALLBACKS);
		});
	}
}

const TREE_CALLBACKS: EntityTreeCallbacks = EntityTreeCallbacks::new()
	.entity_ui(entity_ui);

fn entity_ui(ui: &mut egui::Ui, entity: EntityRef<'_>, commands: &mut Commands) {
	match entity.components::<(Option<&anb::Header>, Option<&anb::Node>)>() {
		(Some(header), None) => {
			egui::Grid::new("header fields").show(ui, |ui| {
				if let Some(edited_header) = edit_editable_data(ui, header) {
					commands.queue(undoable_replace_component(entity.id(), edited_header));
				}
			});
		},
		(None, Some(node)) => {
			egui::Grid::new("node fields").show(ui, |ui| {
				if let Some(edited_node) = edit_editable_data(ui, node) {
					commands.queue(undoable_replace_component(entity.id(), edited_node));
				}
			});
		},
		_ => {
			let error_fg_color = ui.visuals().error_fg_color;
			ui.colored_label(error_fg_color, "Entity has an unexpected component setup");
		},
	}
}

fn interpret_action(world: &mut World, action: (undo_2::Action, &Box<dyn UndoEntry>)) {
	match action.0 {
		undo_2::Action::Do => action.1.redo(world),
		undo_2::Action::Undo => action.1.undo(world),
	}
}

/*
fn load_texture(size: [usize; 2], wflz_data: &[u8], ctx: &egui::Context) -> LoadedTexture {
	let decompressed_data = wflz::decompress(&mut std::io::Cursor::new(wflz_data)).unwrap();
	let image = egui::ColorImage::from_rgba_unmultiplied(size, &decompressed_data);
	let handle = ctx.load_texture("anb texture", image, egui::TextureOptions::NEAREST);
	LoadedTexture { handle }
}
*/

#[derive(Component)]
struct LoadedTexture {
	handle: egui::TextureHandle,
}

impl From<&LoadedTexture> for egui::load::SizedTexture {
	fn from(value: &LoadedTexture) -> Self {
		Self::from_handle(&value.handle)
	}
}
