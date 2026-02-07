use image::{DynamicImage, EncodableLayout};
use image::codecs::png::PngDecoder;
use std::io::Cursor;

use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

pub struct ImageFileView {
	// bytes: FileBytes,
	texture: Result<ImageViewTexture, String>,
}

impl ItemView for ImageFileView {
	fn new(bytes: FileBytes, ctx: &egui::Context) -> Self {
		let name = format!("{:?}", bytes.source_item());
		let texture = ImageViewTexture::load(bytes.as_slice(), ctx, name)
			.map_err(|e| e.to_string());
		Self { /* bytes, */ texture }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		match &self.texture {
			Ok(texture) => {
				let texture = egui::load::SizedTexture {
					id: texture.handle.id(),
					size: texture.size,
				};
				ui.add(egui::Image::new(texture).fit_to_exact_size(ui.available_size()));
				None
			},
			Err(message) => {
				ui.label(format!("Error creating texture:\n{}", message));
				None
			},
		}
	}
}

struct ImageViewTexture {
	handle: egui::TextureHandle,
	size: egui::Vec2,
}

impl ImageViewTexture {
	pub fn load(data: &[u8], ctx: &egui::Context, texture_name: String) -> anyhow::Result<Self> {
		let mut cursor = Cursor::new(data);
		let decoder = PngDecoder::new(&mut cursor)?;
		let decoded_image = DynamicImage::from_decoder(decoder)?;
		
		let egui_image_size: [usize; 2] = [
			decoded_image.width().try_into()?,
			decoded_image.height().try_into()?,
		];
		let egui_image = match &decoded_image {
			DynamicImage::ImageRgb8(image) => {
				egui::ColorImage::from_rgb(egui_image_size, image.as_bytes())
			},
			DynamicImage::ImageRgba8(image) => {
				egui::ColorImage::from_rgba_unmultiplied(egui_image_size, image.as_bytes())
			},
			other => {
				let image = other.to_rgba8();
				egui::ColorImage::from_rgba_unmultiplied(egui_image_size, image.as_bytes())
			},
		};
		
		Ok(Self {
			handle: ctx.load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST),
			size: egui::Vec2::new(decoded_image.width() as f32, decoded_image.height() as f32),
		})
	}
}
