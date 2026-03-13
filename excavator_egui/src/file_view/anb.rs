use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_formats::anb::AnbHeader;
use excavator_formats::util_binary::{ParserStruct, ParserStructError};
use excavator_formats::wflz::WflzDecompressor;

pub struct AnbFileView {
	#[expect(dead_code)] // todo
	bytes: FileBytes,
	texture: Result<SizedTextureHandle, ParserStructError>,
}

impl ItemView for AnbFileView {
	fn new(bytes: FileBytes, ctx: &egui::Context) -> Self where Self: Sized {
		let file = bytes.as_slice();
		
		let texture = (|| {
			let anb_header = ParserStruct::<AnbHeader>::new(file, 0).retrieve()?;
			let wflz_header_offset = anb_header.data_pointer.get() as usize + std::mem::size_of::<u32>() * 6; // there's the AnbDataStart struct but this is lazy hacked code
			
			let data = WflzDecompressor::new(file, wflz_header_offset)?.decompress_all()?;
			
			Ok(data_to_texture(data, format!("{:?} - frame 0", bytes.source_item()), ctx))
		})();
		
		Self { bytes, texture }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		match &self.texture {
			Ok(texture) => {
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
