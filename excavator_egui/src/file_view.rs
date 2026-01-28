mod st;

use egui::Ui;

use crate::file_read::{FileLoader, ItemInfo};
use crate::file_view::st::StFileView;

#[derive(Default)]
pub struct FileViewSwitcher {
	state: SwitcherState,
}

#[derive(Default)]
enum SwitcherState {
	#[default]
	Blank,
	Multi,
	Single {
		item: ItemInfo,
		view: SingleView,
	},
}

enum SingleView {
	Unknown,
	Pak,
	St(StFileView),
}

impl FileViewSwitcher {
	pub fn switch(&mut self, selection: &Vec<ItemInfo>) {
		match selection.len() {
			0 => self.state = SwitcherState::Blank,
			1 => self.switch_single(&selection[0]),
			2.. => self.state = SwitcherState::Multi,
		};
	}
	
	fn switch_single(&mut self, selection: &ItemInfo) {
		let view = match selection.extension() {
			Some(b"pak") => SingleView::Pak,
			Some(b"stb" | b"stl" | b"stm") => SingleView::St(StFileView::default()),
			_ => SingleView::Unknown,
		};
		self.state = SwitcherState::Single { item: selection.clone(), view };
	}
	
	pub fn add_view(&mut self, ui: &mut Ui, loader: &mut FileLoader) {
		match &mut self.state {
			SwitcherState::Blank => { ui.label("No files are selected."); },
			SwitcherState::Multi => { ui.label("Multiple files are selected."); },
			SwitcherState::Single { item, view } => {
				if !item.is_file() {
					ui.label("The selected item isn't a file.");
					return;
				}
				
				match view {
					SingleView::Pak => { ui.label("Archive selected; please select one of the files inside the archive."); },
					SingleView::St(st_view) => {
						if let Some(result) = loader.read_or_request(item) {
							match result {
								Ok(data) => {
									let is_stl = item.extension() == Some(b"stl");
									st_view.view_ui(ui, data, is_stl);
								},
								Err(error) => { ui.label(format!("Error: {}", error)); },
							};
						} else {
							ui.spinner();
						}
					},
					SingleView::Unknown => { ui.label("Unknown or unimplemented file type."); },
				}
			},
		};
	}
}
