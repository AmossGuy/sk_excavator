use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use crate::file_view::common::tree::{entity_tree_ui, EntityTreeCallbacks, ShowInTree};

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

impl FileView for PakFileView {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &ExcavatorContext) {
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root, &TREE_CALLBACKS);
		});
	}
}

const TREE_CALLBACKS: EntityTreeCallbacks = EntityTreeCallbacks::new()
	.entity_ui(entity_ui);

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
