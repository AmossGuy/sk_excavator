#![forbid(unsafe_code)]

mod file_read;
mod file_tree;
mod file_view;
mod plugins;

// use std::convert::Infallible;
use std::path::PathBuf;

use crate::file_read::FileLoader;
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
	file_loader: FileLoader,
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
						messages.lock().send(ExcavatorMessage::LaunchFileDialog {
							dialog: rfd::FileDialog::new().set_parent(&frame),
							kind: FileDialogKind::PickFolder,
							after: |path| { ExcavatorMessage::UpdateGameDir { path } },
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
				let selection_update = self.file_tree.add_view(ui);
				ui.take_available_space();
				
				if let Some(selection_update) = selection_update {
					self.file_view.switch(&selection_update);
				}
			})
		});
		
		egui::CentralPanel::default().show(ctx, |ui| {
			self.file_view.add_view(ui, &mut self.file_loader);
		});
	}
}

#[derive(Clone)]
pub enum ExcavatorMessage {
	LaunchFileDialog {
		dialog: rfd::FileDialog,
		kind: FileDialogKind,
		after: fn(PathBuf) -> ExcavatorMessage,
	},
	UpdateGameDir {
		path: PathBuf,
	},
}

#[derive(Copy, Clone)]
pub enum FileDialogKind {
	PickFile,
	PickFolder,
	SaveFile,
}

impl ExcavatorMessage {
	fn apply(self, app: &mut ExcavatorApp, ctx: &egui::Context) {
		match self {
			Self::LaunchFileDialog { dialog, kind, after } => {
				let spawner = ctx.plugin_or_default::<ThreadSpawner>();
				spawner.lock().spawn(ctx, move || {
					let maybe_path = match kind {
						FileDialogKind::PickFile => dialog.pick_file(),
						FileDialogKind::PickFolder => dialog.pick_folder(),
						FileDialogKind::SaveFile => dialog.save_file(),
					};
					maybe_path.map(after)
				});
			},
			Self::UpdateGameDir { path } => {
				println!("UpdateGameDir: {}", path.display());
				todo!();
			},
		}
	}
}
