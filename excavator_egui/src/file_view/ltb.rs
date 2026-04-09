use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_backend::formats::ltb::{parse_ltb, ParsedLtb};
use excavator_backend::parse::ParseResult;

pub struct LtbFileView {
	parsed: ParseResult<ParsedLtb>,
	current_texture: Option<LtbViewTexture>,
}

impl ItemView for LtbFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut cursor = std::io::Cursor::new(bytes.as_slice());
		// blocks the main thread for now, until i figure out an ergonomic system for all the threading this app ought to do
		let parsed = parse_ltb(&mut cursor);
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
				for (i, data) in parsed.wflz_data_iter().enumerate() {
					match data {
						Ok(data) => if ui.button(format!("wflz data {}", i)).clicked() {
							let sqrt = (data.len() / 4).isqrt();
							let size = egui::Vec2::splat(sqrt as f32);
							let texture_name = format!("ltb thingy #{}", i);
							
							let egui_image = egui::ColorImage::from_rgba_unmultiplied([sqrt, sqrt], &data[..sqrt*sqrt*4]);
							let handle = ui.ctx().load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST);
							
							self.current_texture = Some(LtbViewTexture { handle, size });
						},
						Err(e) => { ui.label(format!("wflz data {} error: {}", i, e)); },
					}
				}
			},
			Err(e) => { ui.label(e.to_string()); },
		}
		None
	}
}

struct LtbViewTexture {
	handle: egui::TextureHandle,
	size: egui::Vec2,
}
