use crate::file_tree::FileTreeView;
use crate::file_view::FileViewLoader;
use super::menubar::{MenuBarAction, show_menu_bar_panel};
use super::settings::ExcavatorSettings;
use super::windows::WindowHolder;

use std::sync::mpsc;

pub struct ExcavatorApp {
	settings: ExcavatorSettings,
	pub windows: WindowHolder,
	file_tree: Option<FileTreeView>,
	file_view: Option<FileViewLoader>,
	
	receiver: mpsc::Receiver<TaskToAppMessage>,
	sender: mpsc::Sender<TaskToAppMessage>,
}

pub enum TaskToAppMessage {
	SetRootPath(std::path::PathBuf),
}

impl ExcavatorApp {
	pub fn main() -> eframe::Result {
		eframe::run_native(
			"SkExcavator",
			eframe::NativeOptions::default(),
			Box::new(|cc| {
				#[cfg(debug_assertions)]
				// workaround for it going off spuriously when clicking on stuff in file tree
				cc.egui_ctx.global_style_mut(|s| s.debug.warn_if_rect_changes_id = false);
				
				Ok(Box::new(Self::new(cc)))
			}),
		)
	}
	
	fn new(cc: &eframe::CreationContext) -> Self {
		let storage = cc.storage.expect("CreationContext should have storage");
		
		let settings = ExcavatorSettings::load(storage);
		let windows = WindowHolder::new();
		let file_tree = Option::map(settings.game_root_path.clone(), FileTreeView::new);
		let file_view = None;
		
		let (sender, receiver) = mpsc::channel();
		
		Self { settings, windows, file_tree, file_view, receiver, sender }
	}
	
	pub fn sender(&self) -> &mpsc::Sender<TaskToAppMessage> {
		&self.sender
	}
	
	pub fn set_game_root_path(&mut self, path: Option<std::path::PathBuf>) {
		self.settings.game_root_path = path;
		self.file_tree = Option::map(self.settings.game_root_path.clone(), FileTreeView::new);
	}
}

impl eframe::App for ExcavatorApp {
	fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// textbook example of this aliasing problem
		// wouldn't need to collect if compiler understood the other stuff doesn't use receiver
		for message in self.receiver.try_iter().collect::<Vec<_>>() {
			match message {
				TaskToAppMessage::SetRootPath(path) => {
					self.set_game_root_path(Some(path));
				},
			}
		}
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		self.windows.show_as_viewports(ui);
		
		show_menu_bar_panel(ui, self, frame);
		
		if let Some(file_view) = &mut self.file_view {
			egui::Panel::right("file view").resizable(true).show_inside(ui, |ui| {
				file_view.ui(ui);
			});
		}
		
		egui::CentralPanel::default().show_inside(ui, |ui| {
			self.game_dir_ui(ui, frame);
		});
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.settings.save(storage);
	}
}

impl ExcavatorApp {
	fn game_dir_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		if let Some(file_tree) = &mut self.file_tree {
			let effect = file_tree.ui(ui);
			if effect.close_clicked {
				MenuBarAction::CloseGameDir.apply(self, ui.ctx(), frame);
			}
			for pls in effect.pls_app {
				pls(self, ui.ctx(), frame);
			}
			if let Some(new_selection) = effect.new_selection {
				self.file_view = match new_selection.len() {
					1 => FileViewLoader::from_file_source(new_selection[0].clone(), ui.ctx()),
					_ => None,
				};
			}
		} else {
			if ui.button("Select game path...").clicked() {
				MenuBarAction::SelectGameDir.apply(self, ui.ctx(), frame);
			}
		}
	}
}
