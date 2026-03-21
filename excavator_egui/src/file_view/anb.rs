use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_formats::anb::{AnbHeader, AnbDataStart, get_the_stupid_sprite_size};
use excavator_formats::util_binary::{ParserStruct, ParserStructError};
use excavator_formats::wflz::WflzDecompressor;

pub struct AnbFileView {
	bytes: FileBytes,
	texture: Result<SizedTextureHandle, anyhow::Error>,
	data_compressed_size: usize,
	data_decompressed_size: usize,
}

impl ItemView for AnbFileView {
	fn new(bytes: FileBytes, ctx: &egui::Context) -> Self where Self: Sized {
		let file = bytes.as_slice();
		
		let mut data_compressed_size = 0;
		let mut data_decompressed_size = 0;
		
		let texture = (|| {
			let anb_header = ParserStruct::<AnbHeader>::new(file, 0).retrieve()?;
			let wflz_header_offset = anb_header.data_pointer.get() as usize + std::mem::size_of::<u32>() * 6; // there's the AnbDataStart struct but this is lazy hacked code
			
			let decompressed = WflzDecompressor::new(file, wflz_header_offset)?.decompress_all()?;
			data_compressed_size = decompressed.compressed_size;
			data_decompressed_size = decompressed.data.len();
			
			let sprite_size = get_the_stupid_sprite_size(file)?;
			let sprite_size_usize = [sprite_size[0] as usize, sprite_size[1] as usize];
			
			if sprite_size_usize[0] * sprite_size_usize[1] * 4 != decompressed.data.len() {
				anyhow::bail!(
					"wrong sprite size info: w{} * h{} * 4 == {} != {}",
					sprite_size_usize[0], sprite_size_usize[1],
					sprite_size_usize[0] * sprite_size_usize[1] * 4,
					decompressed.data.len(),
				);
			}
			
			Ok(data_to_texture(decompressed.data, sprite_size_usize, format!("{:?} - frame 0", bytes.source_item()), ctx))
		})();
		
		Self { bytes, texture, data_compressed_size, data_decompressed_size }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		match &self.texture {
			Ok(texture) => {
				let _: Result<_, ParserStructError> = (|| {
					let file = self.bytes.as_slice();
					
					let anb_header = ParserStruct::<AnbHeader>::new(file, 0).retrieve()?;
					let data_start = ParserStruct::<AnbDataStart>::new(file, anb_header.data_pointer.get() as usize).retrieve()?;
					
					egui::Grid::new("stats grid").show(ui, |ui| {
						ui.label(format!("Compressed size from metadata: {}", data_start.wflz.compressed_size));
						ui.label(format!("Compressed size from parser: {}", self.data_compressed_size));
						ui.end_row();
						
						ui.label(format!("Decompressed size from metadata: {}", data_start.wflz.decompressed_size));
						ui.label(format!("Decompressed size from parser: {}", self.data_decompressed_size));
						ui.end_row();
					});
					
					Ok(())
				})();
				
				ui.label("guys i don't think this is interpreting the data correctly ngl");
				
				let texture = texture.as_texture();
				ui.add(egui::Image::new(texture).fit_to_exact_size(ui.available_size()));
				None
			},
			Err(e) => {
				ui.label(format!("Error creating texture:\n{:?}", e));
				None
			},
		}
	}
}

struct SizedTextureHandle {
	handle: egui::TextureHandle,
	size: egui::Vec2,
}

impl SizedTextureHandle {
	fn as_texture(&self) -> egui::load::SizedTexture {
		egui::load::SizedTexture {
			id: self.handle.id(),
			size: self.size,
		}
	}
}

fn data_to_texture(data: Box<[u8]>, size: [usize; 2], texture_name: String, ctx: &egui::Context) -> SizedTextureHandle {
	let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, &data);
	let handle = ctx.load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST);
	
	SizedTextureHandle {
		handle,
		size: egui::Vec2::new(size[0] as f32, size[1] as f32),
	}
}
