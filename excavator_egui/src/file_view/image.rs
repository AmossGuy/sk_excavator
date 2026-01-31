use egui::Ui;
use image::{DynamicImage, EncodableLayout};
use image::codecs::png::PngDecoder;
use std::io::Cursor;

// use crate::file_read::ItemInfo;

pub struct ImageFileView {
	texture_handle: egui::TextureHandle,
	image_size: [usize; 2],
}

impl ImageFileView {
	pub fn load(data: &[u8], ctx: &egui::Context, texture_name: String) -> anyhow::Result<Self> {
		let mut cursor = Cursor::new(data);
		let decoder = PngDecoder::new(&mut cursor)?;
		let decoded_image = DynamicImage::from_decoder(decoder)?;
		let image_size: [usize; 2] = [
			decoded_image.width().try_into()?,
			decoded_image.height().try_into()?,
		];
		
		let egui_image = match &decoded_image {
			DynamicImage::ImageRgb8(image) => {
				egui::ColorImage::from_rgb(image_size, image.as_bytes())
			},
			DynamicImage::ImageRgba8(image) => {
				egui::ColorImage::from_rgba_unmultiplied(image_size, image.as_bytes())
			},
			other => {
				let image = other.to_rgba8();
				egui::ColorImage::from_rgba_unmultiplied(image_size, image.as_bytes())
			},
		};
		
		let texture_handle = ctx.load_texture(texture_name, egui_image, egui::TextureOptions::NEAREST);
		Ok(Self { texture_handle, image_size })
	}
	
	pub fn view_ui(&mut self, ui: &mut Ui) {
		let texture = egui::load::SizedTexture {
			id: self.texture_handle.id(),
			size: egui::Vec2::new(self.image_size[0] as f32, self.image_size[1] as f32),
		};
		ui.add(egui::Image::new(texture).fit_to_exact_size(ui.available_size()));
	}
}
