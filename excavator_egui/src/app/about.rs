use crate::app::context::ExcavatorContext;
use crate::app::windows::Window;

#[derive(Default)]
pub struct AboutWindow {}

impl AboutWindow {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Window for AboutWindow {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &ExcavatorContext) {
		egui::CentralPanel::default().show(ui, |ui| {
			ui.heading("Shovel Knight Excavator");
			ui.label("by AmossGuy");
			
			ui.separator();
			
			ui.label("Excavator is an in-development tool for modding Shovel Knight. Please note that this an EARLY BUILD; plenty of crucial functionality is not yet implemented.");
			
			let layout = egui::Layout::bottom_up(egui::Align::Center);
			ui.scope_builder(egui::UiBuilder::new().layout(layout), |ui| {
				ui.hyperlink_to("github repository", "https://github.com/AmossGuy/sk_excavator");
				ui.separator();
				ui.take_available_height();
			});
		});
	}
	
	fn initial_size(&self) -> egui::Vec2 {
		egui::Vec2::new(500.0, 300.0)
	}
}
