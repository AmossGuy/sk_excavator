use crate::file_view::common::editable::edit_editable_data;
use crate::file_view::common::tree::{EntityTreeCallbacks, ShowInTree, TreeFileView};

use std::sync::Arc;
use yoke::Yoke;

use bevy_ecs::{
	entity::Entity, system::Commands,
	world::{EntityRef, World},
};

use excavator_backend::formats::pak::def_live as pak;
use excavator_backend::formats::common::undo::undoable_replace_component;

pub struct PakFileView {
	ecs_world: World,
	root: Entity,
}

impl PakFileView {
	pub fn parse(file_contents: Vec<u8>) -> anyhow::Result<Self> {
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
		
		let mut ecs_world = World::new();
		
		ecs_world.register_required_components::<pak::Header, ShowInTree>();
		ecs_world.register_required_components::<pak::FileMetadata, ShowInTree>();
		// EntityRef::components panics if component not registered
		ecs_world.register_component::<pak::FileMetadata>();
		
		let root = excavator_backend::formats::pak::load_from_bytes(&yoke_bytes, &mut ecs_world)?;
		
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
