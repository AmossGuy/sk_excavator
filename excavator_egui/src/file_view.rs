use egui::Ui;

use crate::file_read::ItemInfo;

pub fn show_file_view(ui: &mut Ui, files: Vec<ItemInfo>) {
	match files.len() {
		0 => ui.label("No files are selected."),
		1 => ui.label(format!("Selected file: {:?}", files[0])),
		2.. => ui.label("Multiple files are selected"),
	};
}
