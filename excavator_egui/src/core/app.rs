use super::menubar::show_menu_bar_panel;
use super::message::apply_messages;
use super::settings::ExcavatorSettings;

pub struct ExcavatorApp {
	settings: ExcavatorSettings,
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
		let settings = ExcavatorSettings::load(storage);
		Self { settings }
	}
}

impl eframe::App for ExcavatorApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		apply_messages(ui.ctx(), self);
		
		show_menu_bar_panel(ui);
		egui::Panel::bottom("status bar").show_inside(ui, |_ui| {
		});
		
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.label("(awesomesauce)");
			ui.label(format!("game_root_path: {:?}", self.settings.game_root_path));
		});
	}
	
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		self.settings.save(storage);
	}
}
