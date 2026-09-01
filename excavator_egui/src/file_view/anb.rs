use crate::file_view::FileView;
use crate::file_view::common::tree::TreeFileView;
use excavator_backend::formats::anb::{def_live as anb, load_from_bytes};
// use excavator_backend::formats::wflz;

use std::sync::Arc;
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(TreeFileView::new(anb))
}

/*
pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
}

impl AnbFileView {
	pub fn parse(file_contents: Vec<u8>) -> anyhow::Result<Self> {
		let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
		let root = excavator_backend::formats::anb::load_from_bytes(&yoke_bytes)?;
		Ok(Self { ecs_world, root })
	}
}

impl TreeFileView for AnbFileView {
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
*/
