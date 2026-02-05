#![forbid(unsafe_code)]

mod file_read;
mod file_tree;
mod file_view;
mod plugins;

// use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

use crate::file_read::{ItemInfo, ItemLoader};
use crate::file_tree::FileTree;
use crate::file_view::FileViewSwitcher;
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
	file_tree_root: PathBuf,
	
	#[serde(skip)]
	file_tree: FileTree,
	#[serde(skip)]
	file_view: FileViewSwitcher,
	#[serde(skip)]
	item_loader: ItemLoader,
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
		
		self.file_tree.set_root_from_path_if_different(self.file_tree_root.clone());
		
		egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Select directory...").clicked() {
						let dialog = rfd::FileDialog::new().set_parent(&frame);
						threads.lock().spawn(ctx.clone(), move |_| {
							let maybe_path = dialog.pick_folder();
							maybe_path.map(|path| { ExcavatorMessage::UpdateGameDir { path } })
						});
					}
					if ui.button("Quit").clicked() {
						ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
			});
		});
		
		egui::SidePanel::left("file tree").show(ctx, |ui| {
			egui::ScrollArea::both().show(ui, |ui| {
				let selection_update = self.file_tree.add_view(ui, &self.item_loader);
				ui.take_available_space();
				
				if let Some(selection_update) = selection_update {
					self.file_view.switch(&selection_update);
				}
			})
		});
		
		egui::CentralPanel::default().show(ctx, |ui| {
			self.file_view.add_view(ui, &mut self.item_loader);
		});
	}
}

#[derive(Clone, Debug)]
pub enum ExcavatorMessage {
	UpdateGameDir {
		path: PathBuf,
	},
	ItemLoadDone {
		item: ItemInfo,
		result: Arc<crate::file_read::LoadResult>,
	},
}

impl ExcavatorMessage {
	fn apply(self, app: &mut ExcavatorApp, _ctx: &egui::Context) {
		match self {
			Self::UpdateGameDir { path } => {
				app.file_tree_root = path;
			},
			Self::ItemLoadDone { item, result } => {
				app.file_tree.update_from_load(item, result);
			},
		}
	}
}
