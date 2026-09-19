use crate::file_view::FileView;
use super::menubar::{show_menu_bar_panel, test_menu_bar_shortcuts, ShortcutStorage};
use super::settings::ExcavatorSettings;
use super::windows::{Window, WindowHolder};

use std::{path::PathBuf, sync::{Arc, mpsc}};

pub struct ExcavatorApp {
	excavator: ExcavatorContext,
	windows: WindowHolder,
	receiver: mpsc::Receiver<AppChannelMessage>,
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
		
		let exc_inner = ExcavatorInner {
			settings: ExcavatorSettings::load(storage),
			shortcuts: ShortcutStorage::new(),
			file_view: None,
		};
		
		let (sender, receiver) = mpsc::channel();
		let excavator = ExcavatorContext {
			inner: Arc::new(egui::mutex::RwLock::new(exc_inner)),
			app_sender: sender,
			needs_parent_repaint: egui::mutex::Mutex::new(false),
		};
		
		let windows = WindowHolder::new("global window holder");
		
		Self { excavator, windows, receiver }
	}
}

impl eframe::App for ExcavatorApp {
	fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
		for message in self.receiver.try_iter() {
			match message {
				AppChannelMessage::AddWindow(window) => {
					self.windows.add(window);
				},
			}
		}
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		self.windows.show_all(ui, &self.excavator);
		
		show_menu_bar_panel(ui, &self.excavator);
		test_menu_bar_shortcuts(ui.ctx(), &self.excavator);
		
		if let Some(file_view) = self.excavator.get_file_view() {
			file_view.write().ui(ui, &self.excavator);
		}
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.excavator.settings(|s| s.save(storage));
	}
}

struct ExcavatorInner {
	settings: ExcavatorSettings,
	shortcuts: ShortcutStorage,
	
	// I'll move this elsewhere sometime soon, I think.
	file_view: Option<Arc<egui::mutex::RwLock<Box<dyn FileView>>>>,
}

enum AppChannelMessage {
	AddWindow(Box<dyn Window>),
}

#[derive(Clone)]
pub struct ExcavatorContext {
	inner: Arc<egui::mutex::RwLock<ExcavatorInner>>,
	app_sender: mpsc::Sender<AppChannelMessage>,
	needs_parent_repaint: egui::mutex::Mutex<bool>,
}

impl ExcavatorContext {
	pub fn settings<R>(&self, reader: impl FnOnce(&ExcavatorSettings) -> R) -> R {
		reader(&self.inner.read().settings)
	}
	
	pub fn settings_mut<R>(&self, writer: impl FnOnce(&mut ExcavatorSettings) -> R) -> R {
		writer(&mut self.inner.write().settings)
	}
	
	pub fn shortcuts<R>(&self, reader: impl FnOnce(&ShortcutStorage) -> R) -> R {
		reader(&self.inner.read().shortcuts)
	}
	
	pub fn file_view<R>(&self, reader: impl FnOnce(&dyn FileView) -> R) -> Option<R> {
		match self.inner.read().file_view {
			Some(ref view) => Some(reader(view.read().as_ref())),
			None => None,
		}
	}
	
	pub fn file_view_mut<R>(&self, writer: impl FnOnce(&mut dyn FileView) -> R) -> Option<R> {
		match self.inner.read().file_view {
			Some(ref view) => Some(writer(view.write().as_mut())),
			None => None,
		}
	}
	
	pub fn repaint_parent_if_needed(&self, ctx: &egui::Context) {
		let needed = {
			let mut lock = self.needs_parent_repaint.lock();
			std::mem::replace(&mut *lock, false)
		};
		
		if needed {
			ctx.request_repaint_of(ctx.parent_viewport_id());
		}
	}
	
	pub fn set_needs_parent_repaint(&self) {
		*self.needs_parent_repaint.lock() = true;
	}
	
	pub fn add_window(&self, window: impl Window) {
		self.add_window_boxed(Box::new(window));
	}
	
	pub fn add_window_boxed(&self, window: Box<dyn Window>) {
		let message = AppChannelMessage::AddWindow(window);
		let _ = self.app_sender.send(message);
		self.set_needs_parent_repaint();
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
	
	pub fn open_file(&self, path: PathBuf) {
		self.settings_mut(|s| s.add_recent_file(path.clone()));
		crate::app::load::spawn_load_thread(path, self);
	}
	
	pub fn set_file_view(&self, view: Box<dyn FileView>) {
		self.inner.write().file_view = Some(Arc::new(egui::mutex::RwLock::new(view)));
	}
	
	pub fn get_file_view(&self) -> Option<Arc<egui::mutex::RwLock<Box<dyn FileView>>>> {
		self.inner.read().file_view.clone()
	}
}
