use crate::file_view::FileView;
use super::menubar::{show_menu_bar_panel, test_menu_bar_shortcuts, MenuEnv, ShortcutStorage};
use super::settings::ExcavatorSettings;
use super::windows::{Window, WindowHolder};

use excavator_backend::hash::Unhasher;

use std::{path::PathBuf, sync::{Arc, mpsc}};

pub struct ExcavatorApp {
	excavator: ExcavatorContext,
	windows: WindowHolder,
	file_view: Box<dyn FileView>,
	error_popups: Vec<anyhow::Error>,
	receiver: mpsc::Receiver<AppMessage>,
}

struct PlaceholderFileView;

impl FileView for PlaceholderFileView {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &ExcavatorContext) {
		egui::CentralPanel::default().show(ui, |_| {});
	}
}

impl ExcavatorApp {
	pub fn main() -> eframe::Result {
		let mut options = eframe::NativeOptions::default();
		options.viewport.title = Some("Shovel Knight Excavator".into());
		
		eframe::run_native(
			"SkExcavator",
			options,
			Box::new(|cc| {
				Ok(Box::new(Self::new(cc)))
			}),
		)
	}
	
	fn new(cc: &eframe::CreationContext) -> Self {
		let storage = cc.storage.expect("CreationContext should have storage");
		
		use anyhow::Context;
		let unhasher_result = Self::unhasher_load().context("failed to load unhasher data");
		let (unhasher, unhasher_error) = match  {
			Ok(good) => (good, None),
			Err(e) => (Unhasher::empty(), Some(e)),
		};
		
		let exc_inner = ExcavatorInner {
			settings: ExcavatorSettings::load(storage),
			shortcuts: ShortcutStorage::new(),
			unhasher,
		};
		
		let (sender, receiver) = mpsc::channel();
		let excavator = ExcavatorContext {
			inner: Arc::new(egui::mutex::RwLock::new(exc_inner)),
			app_sender: sender,
			needs_parent_repaint: egui::mutex::Mutex::new(false),
		};
		
		if let Some(e) = unhasher_error {
			excavator.display_error(e);
		}
		
		let windows = WindowHolder::new("global window holder");
		let file_view = Box::new(PlaceholderFileView);
		let error_popups = Vec::new();
		
		Self { excavator, windows, file_view, error_popups, receiver }
	}
	
	fn unhasher_load() -> std::io::Result<Unhasher> {
		// "data folder location" system pending
		let path = PathBuf::from("excavator_data/unhash_string_list.txt");
		let file = std::io::BufReader::new(std::fs::File::open(path)?);
		Unhasher::load(file)
	}
	
	fn update_title(&self, ctx: &egui::Context) {
		let window_title: String = match self.file_view.tab_title() {
			None => "Shovel Knight Excavator".into(),
			Some(tab_title) => format!("{} — Excavator", tab_title),
		};
		
		ctx.send_viewport_cmd(egui::ViewportCommand::Title(window_title));
	}
	
		fn error_popup(&mut self, ui: &mut egui::Ui) {
		let response = egui::Modal::new(ui.id().with("error modal")).show(ui, |ui| {
			match self.error_popups.len() {
				1 => { ui.label("An error occurred:"); },
				other => { ui.label(format!("{other} errors occurred:")); },
			}
			
			egui::ScrollArea::vertical().show(ui, |ui| {
				let error_fg_color = ui.visuals().error_fg_color;
				for error in &mut *self.error_popups {
					ui.colored_label(error_fg_color, format!("{:#}", error));
				}
			});
			
			egui::Sides::new().show(ui, |_| {}, |ui| {
				if ui.button("OK").clicked() {
					ui.close();
				}
			});
		});
		
		if response.should_close() {
			self.error_popups.clear();
		}
	}
}

impl eframe::App for ExcavatorApp {
	fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		use AppMessage::*;
		
		for message in self.receiver.try_iter() {
			match message {
				AddWindow(window) => { self.windows.add(window); },
				SetFileView(view) => { self.file_view = view; self.update_title(ctx); },
				RequestQuit => { ctx.send_viewport_cmd(egui::ViewportCommand::Close); },
				DisplayError(error) => { self.error_popups.push(error.into()); },
			}
		}
	}
	
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		self.windows.show_all(ui, &self.excavator);
		
		let mut menu_env = MenuEnv::new(&self.excavator, &mut *self.file_view);
		show_menu_bar_panel(ui, &mut menu_env);
		test_menu_bar_shortcuts(ui.ctx(), &mut menu_env);
		
		self.file_view.ui(ui, &self.excavator);
		
		if !self.error_popups.is_empty() {
			self.error_popup(ui)
		}
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.excavator.settings(|s| s.save(storage));
	}
}

struct ExcavatorInner {
	settings: ExcavatorSettings,
	shortcuts: ShortcutStorage,
	unhasher: Unhasher,
}

enum AppMessage {
	AddWindow(Box<dyn Window>),
	SetFileView(Box<dyn FileView>),
	RequestQuit,
	DisplayError(anyhow::Error),
}

#[derive(Clone)]
pub struct ExcavatorContext {
	inner: Arc<egui::mutex::RwLock<ExcavatorInner>>,
	app_sender: mpsc::Sender<AppMessage>,
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
	
	fn app_message(&self, message: AppMessage) {
		let _ = self.app_sender.send(message);
		self.set_needs_parent_repaint();
	}
	
	pub fn add_window(&self, window: impl Window) {
		self.add_window_boxed(Box::new(window));
	}
	
	pub fn add_window_boxed(&self, window: Box<dyn Window>) {
		self.app_message(AppMessage::AddWindow(window));
	}
	
	pub fn display_error(&self, error: impl Into<anyhow::Error>) {
		self.app_message(AppMessage::DisplayError(error.into()));
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
		self.app_message(AppMessage::SetFileView(view));
	}
	
	pub fn request_app_quit(&self) {
		self.app_message(AppMessage::RequestQuit);
	}
	
	pub fn unhash(&self, hash: u32) -> Option<Vec<u8>> {
		self.inner.read().unhasher.unhash(hash)
			.map(|x| x.to_vec())
	}
}
