use eframe::egui;
use std::convert::Infallible;
use std::path::PathBuf;

fn main() -> eframe::Result {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		"Shovel Knight Excavator",
		native_options,
		Box::new(|cc| Ok(Box::new(ExcavatorApp::new(cc)))),
	)
}

struct ExcavatorApp {
	choose_dir_bind: Option<egui_async::Bind<Option<PathBuf>, Infallible>>,
	root_path: Option<PathBuf>,
}

impl ExcavatorApp {
	fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		Self {
			choose_dir_bind: None,
			root_path: None,
		}
	}
}

impl eframe::App for ExcavatorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
		
		if let Some(bind) = &mut self.choose_dir_bind {
			if let Some(result) = bind.read_or_request(|| async {
				let handle = rfd::AsyncFileDialog::new().pick_folder().await;
				Ok(handle.map(|h| h.path().to_owned()))
			}) {
				if let Ok(Some(path)) = result {
					self.root_path = Some(path.clone());
				}
				self.choose_dir_bind = None;
			}
		}
		
		egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Select directory...").clicked() {
						self.choose_dir_bind = Some(egui_async::Bind::new(false));
					}
					if ui.button("Quit").clicked() {
						ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
			});
		});
		
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label(format!("{:?}", self.root_path));
		});
	}
}
