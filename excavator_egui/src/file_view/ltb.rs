use super::{FileView, FileViewEffect};

use excavator_backend::formats::ltb::{parse_ltb, ParsedLtb, CHUNK_SIZE};

use std::borrow::Cow;
use std::io::{BufRead, Seek};
use std::sync::Arc;

pub struct LtbFileView {
	parsed: anyhow::Result<Arc<ParsedLtb>>,
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

impl LtbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> Self {
		let parsed = parse_ltb(&mut reader).map(|x| Arc::new(x));
		Self { parsed, tab: Tab::Images, current_texture: None, current_tilemap_layer: None }
	}
}

impl FileView for LtbFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		egui::Panel::top("ltb tabs").show(ui, |ui| ui.horizontal(|ui| {
			for (label, value) in [
				("Images", Tab::Images),
				("Tilemap list", Tab::TilemapList),
				("Tilemap display", Tab::TilemapDisplay),
			] {
				if ui.selectable_label(self.tab == value, label).clicked() {
					self.tab = value;
				}
			}
			
			if ui.button("Dump everything").clicked() {
				if let Ok(parsed) = &self.parsed {
					crate::misc::file_dialog::show_ltb_dump_dialog(Arc::clone(&parsed));
				}
			}
		}));
		
		egui::CentralPanel::default().show(ui, |ui| {
			match self.tab {
				Tab::Images => self.images_ui(ui),
				Tab::TilemapList => self.tilemap_ui(ui),
				Tab::TilemapDisplay => self.tilemap_display(ui),
			};
		});
		
		FileViewEffect::default()
	}
}

impl LtbFileView {
	fn images_ui(&mut self, ui: &mut egui::Ui) {
		egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
			self.images_ui_inner(ui);
		});
	}
		
	fn images_ui_inner(&mut self, ui: &mut egui::Ui) {
		if let Some(ref texture) = self.current_texture {
			let e_texture = egui::load::SizedTexture {
				id: texture.handle.id(),
				size: texture.size,
			};
			ui.add(egui::Image::new(e_texture).fit_to_exact_size(texture.size));
		}
		
		match &self.parsed {
			Ok(parsed) => { egui::ScrollArea::vertical().show(ui, |ui| {
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
								
								self.current_texture = Some(LtbViewTexture { handle, size: size_vec });
							} }
							
							if ui.button("(export)").clicked() {
								if let Ok(data) = ltb_image.decompress() {
									let size = ltb_image.size();
									crate::misc::file_dialog::show_wflz_export_dialog(Vec::from(data), size, ui.ctx());
								}
							}
							
							ui.label(format!("({})", ltb_image.meta_debug()));
						},
						Err(e) => { ui.label(format!("image data {} error: {:?}", i, e)); },
					}
				}
			}); },
			Err(e) => { ui.label(format!("error: {:?}", e)); },
		}
	}
	
	fn tilemap_ui(&mut self, ui: &mut egui::Ui) {
		let Ok(ref parsed) = self.parsed else { return; };
		egui::ScrollArea::both().show(ui, |ui| { for (i, layer_string) in parsed.debug_layers() {
			ui.horizontal(|ui| {
				ui.label(layer_string);
				if ui.button("show").clicked() {
					self.current_tilemap_layer = Some(i);
					self.tab = Tab::TilemapDisplay;
				}
			});
		}});
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
}
