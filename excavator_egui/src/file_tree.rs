use async_executor::Task;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::EXECUTOR;

use excavator_backend::io::dir::DirContents;

enum TreeLoadMessage {
	Complete(std::io::Result<DirContents>),
}

pub struct FileTreeView {
	root_path: PathBuf,
	search_text: String,
	
	receiver: Receiver<TreeLoadMessage>,
	sender: Sender<TreeLoadMessage>,
	task: Task<()>,
	
	load_result: Option<std::io::Result<DirContents>>,
}

#[derive(Default)]
#[must_use]
pub struct FileTreeEffect {
	pub close_clicked: bool,
}

impl FileTreeView {
	pub fn new(root_path: PathBuf) -> Self {
		let (sender, receiver) = channel();
		let task = Self::start_load(root_path.clone(), sender.clone());
		Self {
			root_path,
			search_text: String::new(),
			
			receiver, sender, task,
			load_result: None,
		}
	}
	
	fn start_load(path: PathBuf, sender: Sender<TreeLoadMessage>) -> Task<()> {
		let load = DirContents::read_async(path);
		EXECUTOR.spawn(async move {
			let _ = sender.send(TreeLoadMessage::Complete(load.await));
		})
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) -> FileTreeEffect {
		self.apply_messages();
		
		let effect = self.fixed_ui(ui);
		egui::Frame::group(ui.style()).show(ui, |ui| {
			egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
				self.scrolling_ui(ui);
			});
		});
		
		effect
	}
	
	fn apply_messages(&mut self) {
		for message in self.receiver.try_iter() {
			match message {
				TreeLoadMessage::Complete(result) => {
					self.load_result = Some(result.map(|mut contents| {
						contents.sort_by_name();
						contents
					}));
				},
			}
		}
	}
		
	fn fixed_ui(&mut self, ui: &mut egui::Ui) -> FileTreeEffect {
		let mut effect = FileTreeEffect::default();
		
		let layout = egui::Layout::right_to_left(egui::Align::Min);
		ui.scope_builder(egui::UiBuilder::new().layout(layout), |ui| {
			if ui.button("Close").clicked() {
				effect.close_clicked = true;
			}
			if ui.button("Reload").clicked() {
				self.load_result = None;
				self.task = Self::start_load(self.root_path.clone(), self.sender.clone());
			}
			
			let search_box_size = egui::Vec2::new(ui.available_size().x, ui.min_size().y);
			let search_box = egui::TextEdit::singleline(&mut self.search_text)
				.hint_text("Search...");
			ui.add_sized(search_box_size, search_box);
		});
		
		effect
	}
	
	fn scrolling_ui(&mut self, ui: &mut egui::Ui) {
		match &self.load_result {
			None => {
				ui.spinner();
			},
			Some(Ok(dir_contents)) => {
				for name in dir_contents.name_iter() {
					ui.label(name);
				}
			},
			Some(Err(e)) => {
				let text = egui::RichText::new(e.to_string())
					.color(ui.visuals().error_fg_color)
					.monospace();
				ui.label(text);
			}
		}
	}
}
