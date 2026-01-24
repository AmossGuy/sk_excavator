use egui::Ui;

use crate::file_read::{FileLoader, ItemInfo};

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
	},
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
		self.state = SwitcherState::Single { item: selection.clone() };
	}
	
	pub fn add_view(&mut self, ui: &mut Ui, loader: &mut FileLoader) {
		match &mut self.state {
			SwitcherState::Blank => { ui.label("No files are selected."); },
			SwitcherState::Multi => { ui.label("Multiple files are selected."); },
			SwitcherState::Single { item } => {
				if !item.is_file() {
					ui.label("The selected item isn't a file.");
					return;
				}
				
				match item.extension() {
					Some(b"pak") => { ui.label("Archive selected; please select one of the files inside the archive."); },
					Some(b"stl") => {
						if let Some(result) = loader.read_or_request(item) {
							match result {
								Ok(data) => {
									// I still need to write the code for reading only part of an st file...
									ui.label(format!("size: {} (test)", data.len()));
								},
								Err(error) => { ui.label(format!("Error: {}", error)); },
							};
						} else {
							ui.spinner();
						}
					},
					_ => { ui.label("Unknown or unimplemented file type."); },
				}
			},
		};
	}
}
