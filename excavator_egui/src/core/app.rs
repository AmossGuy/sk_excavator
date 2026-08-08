use crate::file_tree::FileTreeView;
use crate::file_view::FileViewLoader;
use super::menu::MenuAction;
use super::menubar::{MenuBarAction, show_menu_bar_panel};
use super::settings::ExcavatorSettings;
use super::windows::WindowHolder;

use excavator_backend::io::file::FileSource;

use std::sync::Arc;

pub struct ExcavatorApp {
	excavator: ExcavatorContext,
	
	file_tree: Option<FileTreeView>,
	file_view: Option<FileViewLoader>,
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
		
		let excavator = ExcavatorContext::new(
			ExcavatorInner { settings, windows, file_to_open: None }
		);
		Self { excavator, file_tree, file_view }
	}
}

impl eframe::App for ExcavatorApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		// depending on exactly how Ui::ui.show_viewport_deferred works, this might be bad?
		self.excavator.inner.write().windows.show_as_viewports(ui, &self.excavator);
		
		show_menu_bar_panel(ui, &mut self.excavator);
		
		if let Some(file_view) = &mut self.file_view {
			egui::Panel::right("file view").resizable(true).show(ui, |ui| {
				file_view.ui(ui);
			});
		}
		
		match self.excavator.settings(|s| s.game_root_path.clone()) {
			None => { self.file_tree = None; },
			Some(new_root_path) => {
				let file_tree = self.file_tree.get_or_insert_with(|| FileTreeView::new(new_root_path.clone()));
				if new_root_path != file_tree.root_path {
					file_tree.root_path = new_root_path;
				}
			},
		}
		
		egui::CentralPanel::default().show(ui, |ui| {
			self.game_dir_ui(ui);
		});
		
		if let Some(file_to_open) = self.excavator.take_file_to_open() {
			self.file_view = FileViewLoader::from_file_source(file_to_open.clone(), ui.ctx());
			if self.file_view.is_some() {
				self.excavator.add_recent_file(file_to_open);
			}
		}
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.excavator.settings(|s| s.save(storage));
	}
}

impl ExcavatorApp {
	fn game_dir_ui(&mut self, ui: &mut egui::Ui) {
		if let Some(file_tree) = &mut self.file_tree {
			let effect = file_tree.ui(ui);
			if effect.close_clicked {
				MenuBarAction::CloseGameDir.execute(ui.ctx(), &mut self.excavator);
			}
			for pls in effect.pls_app {
				pls(ui.ctx(), &mut self.excavator);
			}
			if let Some(new_selection) = effect.new_selection {
				match new_selection.len() {
					1 => {
						self.excavator.open_file(new_selection[0].clone());
					},
					_ => { 
						self.file_view = None
						
					},
				}
			}
		} else {
			if ui.button("Select game path...").clicked() {
				MenuBarAction::SelectGameDir.execute(ui.ctx(), &mut self.excavator);
			}
		}
	}
}

struct ExcavatorInner {
	settings: ExcavatorSettings,
	windows: WindowHolder,
	
	file_to_open: Option<FileSource>,
}

#[derive(Clone)]
pub struct ExcavatorContext {
	inner: Arc<egui::mutex::RwLock<ExcavatorInner>>,
}

impl ExcavatorContext {
	fn new(inner: ExcavatorInner) -> Self {
		Self {
			inner: Arc::new(egui::mutex::RwLock::new(inner)),
		}
	}
	
	pub fn settings<R>(&self, reader: impl FnOnce(&ExcavatorSettings) -> R) -> R {
		reader(&self.inner.read().settings)
	}
	
	pub fn settings_mut<R>(&self, writer: impl FnOnce(&mut ExcavatorSettings) -> R) -> R {
		writer(&mut self.inner.write().settings)
	}
	
	pub fn add_window(&self, window: impl super::windows::Window) {
		self.inner.write().windows.add(window);
	}
	
	pub fn open_file(&self, file_source: FileSource) {
		self.inner.write().file_to_open = Some(file_source);
	}
	
	fn add_recent_file(&self, file_source: FileSource) {
		self.settings_mut(|s| {
			if let Some(index) = s.recent_files.iter().position(|item| *item == file_source) {
				s.recent_files.remove(index);
			}
			
			s.recent_files.push_back(file_source);
			while s.recent_files.len() > usize::from(s.max_recent_files) {
				s.recent_files.pop_front();
			}
		});
	}
	
	pub fn clear_recent_files(&self) {
		self.settings_mut(|s| s.recent_files.clear());
	}
	
	fn take_file_to_open(&self) -> Option<FileSource> {
		self.inner.write().file_to_open.take()
	}
}
