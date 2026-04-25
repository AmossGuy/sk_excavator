use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_backend::formats::ltb::{parse_ltb, ParsedLtb, CHUNK_SIZE};
use excavator_backend::parse::ParseResult;

use std::borrow::Cow;
use std::sync::Arc;

pub struct LtbFileView {
	parsed: ParseResult<ParsedLtb>,
	tab: Tab,
	
	current_texture: Option<LtbViewTexture>,
	current_tilemap_layer: Option<usize>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Tab {
	Images,
	TilemapList,
	TilemapDisplay
}

impl ItemView for LtbFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut cursor = std::io::Cursor::new(bytes.as_slice());
		// blocks the main thread for now, until i figure out an ergonomic system for all the threading this app ought to do
		let parsed = parse_ltb(&mut cursor);
		Self { parsed, tab: Tab::Images, current_texture: None, current_tilemap_layer: None }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		egui::TopBottomPanel::top("ltb tabs").show_inside(ui, |ui| ui.horizontal(|ui| {
			for (label, value) in [
				("Images", Tab::Images),
				("Tilemap list", Tab::TilemapList),
				("Tilemap display", Tab::TilemapDisplay),
			] {
				if ui.selectable_label(self.tab == value, label).clicked() {
					self.tab = value;
				}
			}
		}));
		
		egui::CentralPanel::default().show_inside(ui, |ui| {
			match self.tab {
				Tab::Images => self.images_ui(ui),
				Tab::TilemapList => self.tilemap_ui(ui),
				Tab::TilemapDisplay => self.tilemap_display(ui),
			};
		});
		
		None
	}
}

impl LtbFileView {
	fn images_ui(&mut self, ui: &mut egui::Ui) {
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
								
								let bytes_per_pixel = match ltb_image.paletted() { false => 4, true => 1 };
								let whatever = size_usize[0] * size_usize [1] * bytes_per_pixel;
								let sliceoed: Cow<'_, [u8]> = match data.get(..whatever) {
									Some(sli) => Cow::from(sli),
									None => {
										let mut vec = Vec::from(data.clone());
										vec.resize(whatever, 0);
										Cow::from(vec)
									},
								};
								
								let egui_image = match ltb_image.paletted() {
									false => egui::ColorImage::from_rgba_unmultiplied(size_usize, &sliceoed),
									true => egui::ColorImage::from_gray(size_usize, &sliceoed),
								};
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
	}
	
	fn tilemap_ui(&mut self, ui: &mut egui::Ui) {
		let Ok(ref parsed) = self.parsed else { return; };
		for (i, layer_string) in parsed.debug_layers() {
			ui.horizontal(|ui| {
				ui.label(layer_string);
				if ui.button("show").clicked() {
					self.current_tilemap_layer = Some(i);
					self.tab = Tab::TilemapDisplay;
				}
			});
		}
	}
	
	fn tilemap_display(&mut self, ui: &mut egui::Ui) {
		let Ok(ref ltb) = self.parsed else { return; };
		let Some(layer_index) = self.current_tilemap_layer else {
			ui.label("select layer in the tilemap list tab to view it here");
			return;
		};
		
		let grid_size = ltb.chunk_grid_size(layer_index);
		ui.label(format!("grid rendering test ({} by {})", grid_size[0], grid_size[1]));
		
		egui::ScrollArea::both().show(ui, |ui| {
			self.tilemap_display_scrollarea(ui, layer_index);
			ui.allocate_space(ui.available_size());
		});
	}
	
	fn tilemap_display_scrollarea(&mut self, ui: &mut egui::Ui, layer_index: usize) {
		let Ok(ref ltb) = self.parsed else { return; };
		
		let grid_size = ltb.chunk_grid_size(layer_index);
		let cell_size = egui::Vec2::new(50.0, 50.0);
		let tile_size = cell_size / CHUNK_SIZE as f32;
		let color = ui.visuals().text_color();
		
		let top_left = ui.cursor().min;
		let grid_area = egui::Vec2::new(grid_size[0] as f32 * cell_size.x, grid_size[1] as f32 * cell_size.y);
		let (_, painter) = ui.allocate_painter(grid_area, egui::Sense::empty());
		
		let mut chunk_iter = ltb.iterate_chunk_offsets(layer_index);
		for column_n in 0..grid_size[1] {
			for row_n in 0..grid_size[0] {
				let chunk_offset = chunk_iter.next().unwrap();
				
				if chunk_offset != 0 {
					let cell_top_left = top_left + egui::Vec2::new(cell_size.x * row_n as f32, cell_size.y * column_n as f32);
					
					let chunk_data = ltb.get_chunk_data(chunk_offset as usize);
					let mut tile_iter = chunk_data.iter();
					for column_n in 0..CHUNK_SIZE {
						for row_n in 0..CHUNK_SIZE {
							let tile = tile_iter.next().unwrap();
							if *tile != 0 {
								let tile_top_left = cell_top_left + egui::Vec2::new(tile_size.x * row_n as f32, tile_size.y * column_n as f32);
								let rect = egui::Rect::from_min_size(tile_top_left, tile_size);
								painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
							}
						}
					}
				}
			}
		}
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
