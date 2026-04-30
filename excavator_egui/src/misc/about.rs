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
			ui.label("about");
		});
	}
}
