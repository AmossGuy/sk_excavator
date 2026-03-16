use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_formats::anb::{AnbHeader, AnbDataStart};
use excavator_formats::util_binary::{ParserStruct, ParserStructError};
use excavator_formats::wflz::WflzDecompressor;

pub struct AnbFileView {
	bytes: FileBytes,
	texture: Result<SizedTextureHandle, ParserStructError>,
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
			
			Ok(data_to_texture(decompressed.data, format!("{:?} - frame 0", bytes.source_item()), ctx))
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

fn data_to_texture(data: Box<[u8]>, texture_name: String, ctx: &egui::Context) -> SizedTextureHandle {
	// obvious placeholder test stuff
	let size_guess = (data.len() / 4).isqrt();
	
	let egui_image = egui::ColorImage::from_rgba_unmultiplied([size_guess; 2], &data[..(size_guess as usize).pow(2) * 4]);
	let handle = ctx.load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST);
	
	SizedTextureHandle {
		handle,
		size: egui::Vec2::splat(size_guess as f32),
	}
}
