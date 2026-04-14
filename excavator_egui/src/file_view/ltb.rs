use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_backend::formats::ltb::{parse_ltb, ParsedLtb};
use excavator_backend::parse::ParseResult;

use std::borrow::Cow;
use std::sync::Arc;

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
			let e_texture = egui::load::SizedTexture {
				id: texture.handle.id(),
				size: texture.size,
			};
			let r = ui.add(egui::Image::new(e_texture).fit_to_exact_size(ui.available_size() * egui::Vec2::new(1.0, 0.3)));
			r.interact(egui::Sense::click()).context_menu(|ui| wflz_context_menu(ui, &texture));
		}
		
		match &self.parsed {
			Ok(parsed) => {
				for (i, ltb_image) in parsed.images().enumerate() {
					match ltb_image {
						Ok(ltb_image) => {
							if ui.button(format!("image data {}", i)).clicked() { 'click: {
								let Ok(data) = ltb_image.decompress() else {
									self.current_texture = None;
									break 'click;
								};
								
								let size = ltb_image.size();
								let size_usize = [size[0] as usize, size[1] as usize];
								let size_vec = egui::Vec2::new(size[0] as f32, size[1] as f32);
								let texture_name = format!("ltb thingy #{}", i);
								
								// im going mad!
								let whatever = size_usize[0] * size_usize [1] * 4;
								// dat's why da names are bad...
								let sliceoed: Cow<'_, [u8]> = match data.get(..whatever) {
									Some(sli) => Cow::from(sli),
									None => {
										let mut vec = Vec::from(data.clone());
										vec.resize(whatever, 0);
										Cow::from(vec)
									},
								};
								
								let egui_image = egui::ColorImage::from_rgba_unmultiplied(size_usize, &sliceoed);
								let handle = ui.ctx().load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST);
								
								let raw = Arc::from(data);
								self.current_texture = Some(LtbViewTexture { handle, size: size_vec, raw });
							} }
							
							ui.label(format!("({})", ltb_image.meta_debug()));
						},
						Err(e) => { ui.label(format!("image data {} error: {}", i, e)); },
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
	raw: Arc<[u8]>,
}

fn wflz_context_menu(ui: &mut egui::Ui, texture: &LtbViewTexture) {
	if ui.button("Export raw decompressed data").clicked() {
		let threads = ui.ctx().plugin_or_default::<crate::plugins::ThreadSpawner>();
		
		let dialog = rfd::FileDialog::new()
			.set_title("Export raw decompressed data");
		
		let raw = Arc::clone(&texture.raw);
		threads.lock().spawn(ui.ctx().clone(), move |_| {
			let outcome = dialog.save_file();
			if let Some(path) = outcome {
				let _ = std::fs::write(&path, raw);
			}
			None
		});
	}
}
