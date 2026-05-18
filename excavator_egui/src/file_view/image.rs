use image::{DynamicImage, EncodableLayout};
use image::codecs::png::PngDecoder;
use std::io::{BufRead, Seek};

use super::{FileView, FileViewEffect};

pub struct ImageFileView {
	texture: Result<ImageViewTexture, String>,
}

impl ImageFileView {
	pub fn load(reader: impl BufRead + Seek, ctx: &egui::Context) -> Self {
		let name = "ImageFileView texture".to_string();
		let texture = ImageViewTexture::load(reader, ctx, name)
			.map_err(|e| e.to_string());
		Self { texture }
	}
}

impl FileView for ImageFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		match &self.texture {
			Ok(texture) => {
				let texture = egui::load::SizedTexture {
					id: texture.handle.id(),
					size: texture.size,
				};
				ui.add(egui::Image::new(texture).fit_to_exact_size(ui.available_size()));
			},
			Err(message) => {
				ui.label(format!("Error creating texture:\n{}", message));
			},
		}
		
		FileViewEffect::default()
	}
}

struct ImageViewTexture {
	handle: egui::TextureHandle,
	size: egui::Vec2,
}

impl ImageViewTexture {
	pub fn load(mut reader: impl BufRead + Seek, ctx: &egui::Context, texture_name: String) -> anyhow::Result<Self> {
		let decoder = PngDecoder::new(&mut reader)?;
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
