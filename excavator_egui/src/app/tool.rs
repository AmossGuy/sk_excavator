use crate::app::context::ExcavatorContext;
use crate::app::windows::Window;

pub struct HashToolWindow {
	hash: u32,
	string: Option<String>,
	update_needed: bool,
}

impl HashToolWindow {
	pub fn new() -> Self {
		Self {
			hash: 0,
			string: None,
			update_needed: true,
		}
	}
}

impl Window for HashToolWindow {
	fn ui(&mut self, ui: &mut egui::Ui, excavator: &ExcavatorContext) {
		egui::CentralPanel::default().show(ui, |ui| {
			if self.update_needed {
				self.string = excavator.unhash(self.hash).map(String::from_utf8_lossy_owned);
				self.update_needed = false;
			}
			
			ui.horizontal(|ui| {
				ui.label("Hash input:");
				let response = ui.add(egui::DragValue::new(&mut self.hash));
				if response.changed() {
					self.update_needed = true;
				}
			});
			
			ui.horizontal(|ui| {
				if let Some(string) = &self.string {
					ui.label("String output:");
					ui.label(string);
				} else {
					ui.label("No string found for this hash");
				}
			});
		});
	}
	
	fn build_viewport(&self, builder: &mut egui::ViewportBuilder) {
		builder.inner_size = Some(egui::Vec2::new(200.0, 200.0));
		builder.title = Some("Hash Tool — Excavator".into());
	}
}
