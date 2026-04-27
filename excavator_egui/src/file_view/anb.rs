use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_backend::formats::anb::{decompress_wflz, parse_anb, ParsedAnb, ParsedAnbNode, ParsedData};

pub struct AnbFileView {
	parsed: anyhow::Result<ParsedAnb>,
	current_texture: Option<AnbViewTexture>,
}

impl ItemView for AnbFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut cursor = std::io::Cursor::new(bytes.as_slice());
		// blocks the main thread for now, until i figure out an ergonomic system for all the threading this app ought to do
		let parsed = parse_anb(&mut cursor);
		Self { parsed, current_texture: None }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		if let Some(ref texture) = self.current_texture {
			let texture = egui::load::SizedTexture {
				id: texture.handle.id(),
				size: texture.size,
			};
			ui.add(egui::Image::new(texture).fit_to_exact_size(ui.available_size() * egui::Vec2::new(1.0, 0.3)));
		}
		
		match &self.parsed {
			Ok(parsed) => {
				egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
					node_ui(ui, "root", parsed.root(), &mut self.current_texture);
				});
			},
			Err(e) => { ui.label(format!("failed to load anb: {}", e)); },
		};
		
		None
	}
}

fn node_ui(ui: &mut egui::Ui, index: impl std::hash::Hash + std::fmt::Display, node: &anyhow::Result<ParsedAnbNode>, current_texture: &mut Option<AnbViewTexture>) {
	match node {
		Ok(node) => {
			egui::CollapsingHeader::new(format!("{} (kind: {})", index, node.kind()))
				.id_salt(index)
				.default_open(true)
				.show(ui, |ui| {
					match node.data() {
						ParsedData::FrameWflz { metadata, data } => {
							if ui.button("show image").clicked() { 'click: {
								let Ok(data) = decompress_wflz(&mut std::io::Cursor::new(data)) else {
									break 'click;
								};
								
								let egui_image_size = [metadata.image_width as usize, metadata.image_height as usize];
								let egui_image = egui::ColorImage::from_rgba_unmultiplied(egui_image_size, &data);
								let texture_name = "let's hope giving this a fixed name is good enough to work for now lol";
								*current_texture = Some(AnbViewTexture {
									handle: ui.ctx().load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST),
									size: egui::Vec2::new(metadata.image_width as f32, metadata.image_height as f32),
								});
							} }
						},
						_ => {},
					}
					
					for (i, child) in node.children().enumerate() {
						node_ui(ui, i, child, current_texture);
					}
				});
		},
		Err(e) => { ui.label(format!("error reading node: {}", e)); },
	}
}

struct AnbViewTexture {
	handle: egui::TextureHandle,
	size: egui::Vec2,
}
