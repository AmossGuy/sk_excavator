use crate::file_view::{FileView, FileViewEffect};
use crate::file_view::common::editable::struct_edit_ui;
use crate::file_view::common::tree::{entity_tree_ui, EntityTreeCallbacks};

use std::io::{BufRead, Seek};

use bevy_ecs::{
	component::Component, entity::Entity, system::Commands,
	world::{EntityRef, World},
};

use excavator_backend::formats::anb::def_live as anb;
use excavator_backend::formats::wflz;

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
	
	save_message: String,
}

impl AnbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> anyhow::Result<Self> {
		use {std::sync::Arc, yoke::Yoke};
		
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes)?;
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(bytes), |vec| &vec[..]);
		
		let mut ecs_world = World::new();
		let root = excavator_backend::formats::anb::load_from_bytes(&yoke_bytes, &mut ecs_world)?;
		
		let save_message = String::new();
		
		Ok(Self { ecs_world, root, save_message })
	}
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
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
		ui.label(&self.save_message);
		
		ui.separator();
		
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root, &TREE_CALLBACKS);
		});
		
		FileViewEffect::default()
	}
}

const TREE_CALLBACKS: EntityTreeCallbacks = EntityTreeCallbacks::new()
	.entity_ui(entity_ui);

fn entity_ui(ui: &mut egui::Ui, entity: EntityRef<'_>, commands: &mut Commands) {
	match entity.components::<(Option<&anb::Header>, Option<&anb::Node>)>() {
		(Some(header), None) => {
			egui::Grid::new("header edit").show(ui, |ui| {
				struct_edit_ui(ui, header);
			});
		},
		(None, Some(node)) => {
			ui.label("todo");
		},
		_ => {
			let error_fg_color = ui.visuals().error_fg_color;
			ui.colored_label(error_fg_color, "Entity has an unexpected component setup");
		},
	}
}

fn load_texture(size: [usize; 2], wflz_data: &[u8], ctx: &egui::Context) -> LoadedTexture {
	let decompressed_data = wflz::decompress(&mut std::io::Cursor::new(wflz_data)).unwrap();
	let image = egui::ColorImage::from_rgba_unmultiplied(size, &decompressed_data);
	let handle = ctx.load_texture("anb texture", image, egui::TextureOptions::NEAREST);
	LoadedTexture { handle }
}

#[derive(Component)]
struct LoadedTexture {
	handle: egui::TextureHandle,
}

impl From<&LoadedTexture> for egui::load::SizedTexture {
	fn from(value: &LoadedTexture) -> Self {
		Self::from_handle(&value.handle)
	}
}
