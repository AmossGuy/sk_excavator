use crate::file_tree::FileTreeView;
use crate::file_view::FileViewLoader;
use super::menu::MenuAction;
use super::menubar::{MenuBarAction, show_menu_bar_panel};
use super::settings::ExcavatorSettings;
use super::windows::WindowHolder;

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
			ExcavatorInner { settings, windows }
		);
		Self { excavator, file_tree, file_view }
	}
}

impl eframe::App for ExcavatorApp {
	fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		// depending on exactly how Ui::ui.show_viewport_deferred works, this might be bad?
		self.excavator.inner.write().windows.show_as_viewports(ui);
		
		show_menu_bar_panel(ui, &mut self.excavator);
		
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
		self.excavator.settings(|s| s.save(storage));
	}
}

impl ExcavatorApp {
	fn game_dir_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		if let Some(file_tree) = &mut self.file_tree {
			let effect = file_tree.ui(ui);
			if effect.close_clicked {
				MenuBarAction::CloseGameDir.execute(ui.ctx(), &mut self.excavator);
			}
			for pls in effect.pls_app {
				pls(ui.ctx(), &mut self.excavator);
			}
			if let Some(new_selection) = effect.new_selection {
				self.file_view = match new_selection.len() {
					1 => FileViewLoader::from_file_source(new_selection[0].clone(), ui.ctx()),
					_ => None,
				};
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
}
