#![forbid(unsafe_code)]

mod file_read;
mod file_tree;
mod file_view;
mod file_write;
mod plugins;

use std::path::PathBuf;
use std::sync::Arc;

use crate::file_read::{ItemInfo, BytesLoadResult, ListingLoadResult};
use crate::file_tree::FileTree;
use crate::file_view::FileViewSwitcher;
use crate::file_write::FileExtractor;
use crate::plugins::{MessageQueue, ThreadSpawner};

fn main() -> eframe::Result {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		"Shovel Knight Excavator",
		native_options,
		Box::new(|cc| {
			if let Some(storage) = cc.storage && let Some(app) = eframe::get_value::<ExcavatorApp>(storage, eframe::APP_KEY) {
				Ok(Box::new(app))
			}	else {
				Ok(Box::new(ExcavatorApp::default()))
			}
		}),
	)
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ExcavatorApp {
	file_tree_root: Option<PathBuf>,
	is_hex_editor_on: bool,
	
	#[serde(skip)]
	file_tree: FileTree,
	#[serde(skip)]
	file_view: FileViewSwitcher,
	#[serde(skip)]
	extractor: FileExtractor,
}

impl eframe::App for ExcavatorApp {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
	
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let messages = ctx.plugin_or_default::<MessageQueue>();
		let threads = ctx.plugin_or_default::<ThreadSpawner>();
		messages.lock().send_multiple(threads.lock().take_messages());
		messages.lock().apply_all(self, ctx);
		
		if let Some(ref file_tree_root) = self.file_tree_root {
			self.file_tree.set_root_from_path_if_different(file_tree_root.clone());
		}
		
		self.extractor.run(ctx);
		
		let mut file_view_refresh_needed = false;
		
		egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Select directory...").clicked() {
						let dialog = rfd::FileDialog::new()
							.set_parent(&frame)
							.set_title("Select directory");
						threads.lock().spawn(ctx.clone(), move |_| {
							let maybe_path = dialog.pick_folder();
							maybe_path.map(|path| { ExcavatorMessage::UpdateGameDir { path } })
						});
					}
					if ui.button("Quit").clicked() {
						ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
				
				ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui| {
					let is_hex_editor_on_prev = self.is_hex_editor_on;
					ui.checkbox(&mut self.is_hex_editor_on, "Hex editor");
					if is_hex_editor_on_prev != self.is_hex_editor_on {
						file_view_refresh_needed = true;
					}
				});
			});
		});
		
		egui::SidePanel::left("file tree").show(ctx, |ui| {
			egui::ScrollArea::both().show(ui, |ui| {
				let selection_update = self.file_tree.add_view(ui);
				ui.take_available_space();
				
				if let Some(selection_update) = selection_update {
					self.file_view.switch(&selection_update, self.is_hex_editor_on, &ui.ctx());
				}
			})
		});
		
		if file_view_refresh_needed {
			self.file_view.switch_same(self.is_hex_editor_on, ctx);
		}
		
		egui::CentralPanel::default().show(ctx, |ui| {
			if let Some(message) = self.file_view.add_view(ui) {
				messages.lock().send(message);
			}
		});
	}
}

pub enum ExcavatorMessage {
	UpdateGameDir {
		path: PathBuf,
	},
	ExtractItem {
		item: ItemInfo,
		dest: PathBuf,
	},
	ListingLoadDone {
		item: ItemInfo,
		result: Arc<ListingLoadResult>,
	},
	BytesLoadDone {
		path: PathBuf,
		result: Arc<BytesLoadResult>,
	}
}

impl ExcavatorMessage {
	fn apply(self, app: &mut ExcavatorApp, ctx: &egui::Context) {
		match self {
			Self::UpdateGameDir { path } => {
				app.file_tree_root = Some(path);
			},
			Self::ExtractItem { item, dest } => {
				app.extractor.submit(item, dest);
			},
			Self::ListingLoadDone { item, result } => {
				app.file_tree.update_from_load(item, result);
			},
			Self::BytesLoadDone { path, result } => {
				app.file_view.update_from_load(&path, Arc::clone(&result), ctx);
				app.extractor.when_a_load_finishes(ctx, &path, result);
			},
		}
	}
}
