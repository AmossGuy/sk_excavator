use crate::file_tree::FileTreeView;
use super::menubar::{MenuBarAction, show_menu_bar_panel};
use super::message::{apply_messages, Message, send_message, show_status_bar_panel};
use super::settings::ExcavatorSettings;
use super::windows::WindowHolder;

pub struct ExcavatorApp {
	settings: ExcavatorSettings,
	pub windows: WindowHolder,
	file_tree: Option<FileTreeView>,
}

impl ExcavatorApp {
	pub fn main() -> eframe::Result {
		eframe::run_native(
			"SkExcavator",
			eframe::NativeOptions::default(),
			Box::new(|cc| {
				Ok(Box::new(Self::new(cc)))
			}),
		)
	}
	
	fn new(cc: &eframe::CreationContext) -> Self {
		let storage = cc.storage.expect("CreationContext should have storage");
		
		let settings = ExcavatorSettings::load(storage);
		let windows = WindowHolder::new();
		let file_tree = Option::map(settings.game_root_path.clone(), FileTreeView::new);
		
		Self { settings, windows, file_tree }
	}
	
	pub fn set_game_root_path(&mut self, path: Option<std::path::PathBuf>) {
		self.settings.game_root_path = path;
		self.file_tree = Option::map(self.settings.game_root_path.clone(), FileTreeView::new);
	}
}

impl eframe::App for ExcavatorApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		apply_messages(ui.ctx(), self);
		
		self.windows.show_as_viewports(ui);
		
		show_menu_bar_panel(ui);
		show_status_bar_panel(ui);
		
		egui::CentralPanel::default().show_inside(ui, |ui| {
			if let Some(file_tree) = &mut self.file_tree {
				file_tree.ui(ui);
			} else {
				if ui.button("Select game path...").clicked() {
					send_message(ui.ctx(), Message::MenuBarAction(MenuBarAction::SelectGameDir));
				}
			}
		});
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.settings.save(storage);
	}
}
