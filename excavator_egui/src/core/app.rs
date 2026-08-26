use crate::file_view::FileViewLoader;
use super::menubar::show_menu_bar_panel;
use super::settings::ExcavatorSettings;
use super::windows::WindowHolder;

use std::{path::{Path, PathBuf}, sync::Arc};

pub struct ExcavatorApp {
	excavator: ExcavatorContext,
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
		let file_view = None;
		
		let excavator = ExcavatorContext::new(
			ExcavatorInner { settings, windows, new_file_path: None }
		);
		Self { excavator, file_view }
	}
}

impl eframe::App for ExcavatorApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		// depending on exactly how Ui::ui.show_viewport_deferred works, this might be bad?
		self.excavator.inner.write().windows.show_as_viewports(ui, &self.excavator);
		
		show_menu_bar_panel(ui, &self.excavator);
		
		if let Some(path) = self.excavator.inner.write().new_file_path.take() {
			self.file_view = FileViewLoader::from_path(path, ui.ctx());
		}
		
		egui::CentralPanel::default().show(ui, |ui| {
			if let Some(file_view) = &mut self.file_view {
				file_view.ui(ui);
			} else {
				ui.label("no file open");
			}
		});
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.excavator.settings(|s| s.save(storage));
	}
}

struct ExcavatorInner {
	settings: ExcavatorSettings,
	windows: WindowHolder,
	
	// temporary solution:
	new_file_path: Option<PathBuf>,
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
	
	pub fn open_file_dialog(&self) {
		let mut dialog = rfd::FileDialog::new();
		dialog = dialog.set_title("Open File — Excavator");
		
		if let Some(path) = self.settings(|s| s.open_dialog_dir.clone()) {
			dialog = dialog.set_directory(path);
		}
		
		let excavator = self.clone();
		std::thread::spawn(move || {
			if let Some(path) = dialog.pick_file() {
				let parent = path.parent().map(|p| p.to_path_buf());
				excavator.settings_mut(|s| s.open_dialog_dir = parent);
				
				excavator.open_file(path);
			}
		});
	}
	
	pub fn open_file<P: AsRef<Path>>(&self, path: P) {
		let path = path.as_ref().to_path_buf();
		self.settings_mut(|s| s.add_recent_file(path.clone()));
		self.inner.write().new_file_path = Some(path);
	}
}
