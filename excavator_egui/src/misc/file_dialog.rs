use crate::core::app::ExcavatorContext;
use rfd::FileDialog;

use bstr::{BString, ByteSlice};
use std::path::PathBuf;
use excavator_backend::formats::pak_old::do_single_pak_extract;

pub fn show_game_path_dialog(ctx: &egui::Context, excavator: &ExcavatorContext) {
	let dialog = FileDialog::new()
		.set_title("Select game path");
	
	let excavator = excavator.clone();
	let ctx = ctx.clone();
	
	std::thread::spawn(move || {
		if let Some(path) = dialog.pick_folder() {
			excavator.settings_mut(|s| s.game_root_path = Some(path));
			ctx.request_repaint();
		}
	});
}

pub fn show_file_extract_dialog(outer_path: PathBuf, inner_path: BString, ctx: &egui::Context) {
	let dialog = FileDialog::new()
		.set_title("Extract from archive")
		.set_file_name(inner_path.to_str_lossy());
	
	let ctx = ctx.clone();
	
	std::thread::spawn(move || {
		if let Some(save_path) = dialog.save_file() {
			if do_single_pak_extract(outer_path, inner_path, save_path).is_ok() {
				ctx.request_repaint();
			}
		}
	});
}

pub fn show_wflz_export_dialog(data: Vec<u8>, size: [u32; 2], _ctx: &egui::Context) { // who cares about the frame, i'm hustling!!!
	let dialog = FileDialog::new()
		.set_title("Export WFLZ image as PNG")
		.add_filter("PNG image", &["png"]);
	
	// let ctx = ctx.clone();
	
	std::thread::spawn(move || {
		if let Some(save_path) = dialog.save_file() {
			let _ = (|| -> anyhow::Result<()> {
				use image::ImageEncoder;
				
				let writer = std::fs::File::create(save_path)?;
				let encoder = image::codecs::png::PngEncoder::new(writer);
				
				encoder.write_image(&data, size[0], size[1], image::ExtendedColorType::Rgba8)?;
				Ok(())
			})();
		}
	});
}

pub fn show_ltb_dump_dialog(ltb: std::sync::Arc<excavator_backend::formats::ltb::ParsedLtb>) {
	let dialog = FileDialog::new()
		.set_title("Select empty folder for .ltb dump");
	
	std::thread::spawn(move || {
		if let Some(save_path) = dialog.pick_folder() {
			let _ = ltb.dump_everything(save_path);
		}
	});
}
