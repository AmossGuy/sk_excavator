use crate::core::app::{ExcavatorApp, TaskToAppMessage};
use rfd::FileDialog;

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
