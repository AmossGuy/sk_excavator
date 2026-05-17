use crate::core::app::{ExcavatorApp, TaskToAppMessage};
use rfd::FileDialog;

use bstr::{BString, ByteSlice};
use std::path::PathBuf;
use excavator_backend::formats::pak::do_single_pak_extract;

pub fn show_game_path_dialog(app: &mut ExcavatorApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
	let dialog = FileDialog::new()
		.set_title("Select game path")
		.set_parent(frame);
	
	let sender = app.sender().clone();
	let ctx = ctx.clone();
	
	std::thread::spawn(move || {
		if let Some(path) = dialog.pick_folder() {
			if sender.send(TaskToAppMessage::SetRootPath(path)).is_ok() {
				ctx.request_repaint();
			}
		}
	});
}

pub fn show_file_extract_dialog(outer_path: PathBuf, inner_path: BString, ctx: &egui::Context, frame: &mut eframe::Frame) {
	let dialog = FileDialog::new()
		.set_title("Extract from archive")
		.set_file_name(inner_path.to_str_lossy())
		.set_parent(frame);
	
	let ctx = ctx.clone();
	
	std::thread::spawn(move || {
		if let Some(save_path) = dialog.save_file() {
			if do_single_pak_extract(outer_path, inner_path, save_path).is_ok() {
				ctx.request_repaint();
			}
		}
	});
}
