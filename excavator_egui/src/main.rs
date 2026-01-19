#![forbid(unsafe_code)]

mod file_read;
mod file_tree;

use std::convert::Infallible;
use std::path::PathBuf;

use file_tree::FileTree;

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
	choose_dir_bind: Option<egui_async::Bind<Option<PathBuf>, Infallible>>,
	#[serde(skip)]
	file_tree: FileTree,
}

impl eframe::App for ExcavatorApp {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
	
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
		
		if let Some(bind) = &mut self.choose_dir_bind {
			let dialog = rfd::AsyncFileDialog::new().set_parent(&frame);
			if let Some(result) = bind.read_or_request(|| async {
				let handle = dialog.pick_folder().await;
				Ok(handle.map(|h| h.path().to_owned()))
			}) {
				if let Ok(Some(path)) = result {
					self.file_tree_root = path.clone();
				}
				self.choose_dir_bind = None;
			}
		}
		
		self.file_tree.set_root_from_path_if_different(self.file_tree_root.clone());
		
		egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Select directory...").clicked() {
						// If there's already a file dialog open, overwriting choose_dir_bind leaves it open, but the program will no longer react if something is chosen with it.
						// TODO: Either make it able to handle multiple file dialogs, or just show an error popup.
						if self.choose_dir_bind.is_none() {
							self.choose_dir_bind = Some(egui_async::Bind::new(true));
						}
					}
					if ui.button("Quit").clicked() {
						ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
			});
		});
		
		egui::SidePanel::left("file tree").show(ctx, |ui| {
			egui::ScrollArea::both().show(ui, |ui| {
				self.file_tree.add_view(ui);
				ui.take_available_space();
			})
		});
		
		egui::CentralPanel::default().show(ctx, |_ui| {
		});
	}
}
