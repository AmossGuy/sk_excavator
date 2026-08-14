use super::{FileView, FileViewEffect};
use crate::file_view::common::tree::{entity_tree_ui, EntityTreeCallbacks};

use std::io::{BufRead, Seek};

use bevy_ecs::{entity::Entity, world::World};

pub struct PakFileView {
	ecs_world: World,
	root: Entity,
}

impl PakFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> anyhow::Result<Self> {
		use {std::sync::Arc, yoke::Yoke};
		
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes)?;
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(bytes), |vec| &vec[..]);
		
		let mut ecs_world = World::new();
		let root = excavator_backend::formats::pak::load_from_bytes(&yoke_bytes, &mut ecs_world)?;
		
		Ok(Self { ecs_world, root })
	}
}

impl FileView for PakFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root, &TREE_CALLBACKS);
		});
		
		FileViewEffect::default()
	}
}

const TREE_CALLBACKS: EntityTreeCallbacks = EntityTreeCallbacks::new();
