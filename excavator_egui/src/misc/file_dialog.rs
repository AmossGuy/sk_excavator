use crate::EXECUTOR;
use crate::core::app::{ExcavatorApp, TaskToAppMessage};
use rfd::AsyncFileDialog;

pub fn show_game_path_dialog(app: &mut ExcavatorApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
	let dialog = AsyncFileDialog::new()
		.set_title("Select game path")
		.set_parent(frame)
		.pick_folder();
	
	let sender = app.sender().clone();
	let ctx = ctx.clone();
	EXECUTOR.spawn(async move {
		if let Some(handle) = dialog.await {
			let path = handle.path().to_path_buf();
			let _ = sender.send(TaskToAppMessage::SetRootPath(path));
			ctx.request_repaint();
		}
	}).detach();
}
