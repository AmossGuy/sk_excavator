use crate::core::windows::Window;

#[derive(Default)]
pub struct AboutWindow {}

impl AboutWindow {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Window for AboutWindow {
	fn ui(&mut self, ui: &mut egui::Ui) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Shovel Knight Excavator");
			ui.label("by AmossGuy");
			
			ui.separator();
			
			ui.label("Excavator is an in-development tool for modding Shovel Knight: Treasure Trove. You're here a bit early; the basic, necessary functionality is still being implemented. But welcome, nonetheless!");
			
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
