use crate::file_view::FileView;
use crate::file_view::common::tree::TreeFileView;
use excavator_backend::formats::pak::{def_live as pak, load_from_bytes};

use std::sync::Arc;
use yoke::Yoke;

pub fn parse_pak(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let pak = load_from_bytes(&yoke_bytes)?;
	Ok(TreeFileView::new(pak))
}

/*
pub struct PakFileView {
	ecs_world: World,
	root: Entity,
}

impl PakFileView {
	pub fn parse(file_contents: Vec<u8>) -> anyhow::Result<Self> {
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
		let root = excavator_backend::formats::pak::load_from_bytes(&yoke_bytes)?;
		Ok(Self { ecs_world, root })
	}
}

impl TreeFileView for PakFileView {
	fn ecs_world(&self) -> &World {
		&self.ecs_world
	}
	
	fn ecs_world_mut(&mut self) -> &mut World {
		&mut self.ecs_world
	}
	
	fn root_id(&self) -> Entity {
		self.root
	}
	
	fn tree_callbacks(&self) -> EntityTreeCallbacks {
		EntityTreeCallbacks {
			entity_ui: entity_ui,
		}
	}
}

fn entity_ui(ui: &mut egui::Ui, entity: EntityRef<'_>, commands: &mut Commands) {
	match entity.components::<(Option<&pak::Header>, Option<&pak::FileMetadata>, Option<&pak::FileName>)>() {
		(Some(header), None, None) => {
			egui::Grid::new("header fields").show(ui, |ui| {
				if let Some(edited_header) = edit_editable_data(ui, header) {
					commands.queue(undoable_replace_component(entity.id(), edited_header));
				}
			});
		},
		(None, Some(metadata), Some(name)) => {
			egui::Grid::new("file fields").show(ui, |ui| {
				ui.label("name");
				ui.label(String::from_utf8_lossy(name.name.get())); // needs to be made editable
				ui.end_row();
				
				if let Some(edited_metadata) = edit_editable_data(ui, metadata) {
					commands.queue(undoable_replace_component(entity.id(), edited_metadata));
				}
			});
		},
		_ => {
			let error_fg_color = ui.visuals().error_fg_color;
			ui.colored_label(error_fg_color, "Entity has an unexpected component setup");
		},
	}
}
*/
