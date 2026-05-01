use std::path::PathBuf;
use crate::core::message::{Message, send_message};
use crate::core::menubar::MenuBarAction;

pub struct FileTreeView {
	root_path: PathBuf,
	search_text: String,
}

impl FileTreeView {
	pub fn new(root_path: PathBuf) -> Self {
		Self {
			root_path,
			search_text: String::new(),
		}
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) {
		self.fixed_ui(ui);
		egui::Frame::group(ui.style()).show(ui, |ui| {
			egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
				self.scrolling_ui(ui);
			});
		});
	}
		
	fn fixed_ui(&mut self, ui: &mut egui::Ui) {
		let layout = egui::Layout::right_to_left(egui::Align::Min);
		ui.scope_builder(egui::UiBuilder::new().layout(layout), |ui| {
			if ui.button("Close").clicked() {
				send_message(ui.ctx(), Message::MenuBarAction(MenuBarAction::CloseGameDir));
			}
			if ui.button("Reload").clicked() {
				// TODO: It's not implemented enough for there to be anything to reload, lol
			}
			
			let search_box_size = egui::Vec2::new(ui.available_size().x, ui.min_size().y);
			let search_box = egui::TextEdit::singleline(&mut self.search_text)
				.hint_text("Search...");
			ui.add_sized(search_box_size, search_box);
		});
	}
	
	fn scrolling_ui(&mut self, ui: &mut egui::Ui) {
		let test_contents = (0..=1000).into_iter().map(|x| x.to_string());
		for entry in test_contents {
			ui.label(entry);
		}
	}
}
