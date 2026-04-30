use crate::EXECUTOR;
use crate::core::message::{Message, send_message};
use rfd::AsyncFileDialog;

pub fn show_game_path_dialog(ctx: &egui::Context) {
	let dialog = AsyncFileDialog::new()
		.set_title("Select game path")
		.pick_folder();
	
	let lifeline = ctx.clone();
	EXECUTOR.spawn(async move {
		if let Some(handle) = dialog.await {
			send_message(&lifeline, Message::SetGamePath(handle.path().to_path_buf()));
		}
	}).detach();
}
