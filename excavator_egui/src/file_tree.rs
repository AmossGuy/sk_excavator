use async_executor::Task;
use bstr::BString;
use std::collections::{hash_map, HashMap};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::EXECUTOR;

use excavator_backend::io::dir::{DirContents, DirItem};

enum TreeLoadMessage {
	RootComplete { result: std::io::Result<DirItem> },
	DirComplete { path: PathBuf, result: std::io::Result<DirContents> },
}

pub struct FileTreeView {
	root_path: PathBuf,
	search_text: String,
	
	receiver: Receiver<TreeLoadMessage>,
	sender: Sender<TreeLoadMessage>,
	
	root: Option<std::io::Result<DirItem>>,
	#[allow(dead_code)] // we want to keep it around just for cancel-on-drop
	root_task: Task<()>,
	dirs: HashMap<PathBuf, std::io::Result<DirContents>>,
	dir_tasks: HashMap<PathBuf, Task<()>>,
}

#[derive(Default)]
#[must_use]
pub struct FileTreeEffect {
	pub close_clicked: bool,
}

impl FileTreeView {
	pub fn new(root_path: PathBuf) -> Self {
		let (sender, receiver) = channel();
		let root_task = Self::start_root_load(root_path.clone(), sender.clone());
		
		Self {
			root_path,
			search_text: String::new(),
			
			receiver,
			sender,
			
			root: None,
			root_task,
			dirs: HashMap::new(),
			dir_tasks: HashMap::new(),
		}
	}
	
	fn start_root_load(path: PathBuf, sender: Sender<TreeLoadMessage>) -> Task<()> {
		let load = DirItem::read_single_async(path);
		EXECUTOR.spawn(async move {
			let _ = sender.send(TreeLoadMessage::RootComplete { result: load.await });
		})
	}
	
	fn start_dir_load(path: PathBuf, sender: Sender<TreeLoadMessage>) -> Task<()> {
		let load = DirContents::read_async(path.clone());
		EXECUTOR.spawn(async move {
			let _ = sender.send(TreeLoadMessage::DirComplete { path, result: load.await });
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
				TreeLoadMessage::RootComplete { result } => {
					self.root = Some(result);
				},
				TreeLoadMessage::DirComplete { path, mut result } => {
					if let Ok(ref mut dir_contents) = result {
						dir_contents.sort_by_name();
					}
					self.dirs.insert(path, result);
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
				*self = Self::new(self.root_path.clone());
			}
			
			let search_box_size = egui::Vec2::new(ui.available_size().x, ui.min_size().y);
			let search_box = egui::TextEdit::singleline(&mut self.search_text)
				.hint_text("Search...");
			ui.add_sized(search_box_size, search_box);
		});
		
		effect
	}
	
	fn scrolling_ui(&mut self, ui: &mut egui::Ui) {
		match &self.root {
			None => {
				ui.spinner();
			},
			Some(Ok(item)) => {
				let tree = egui_ltreeview::TreeView::new(ui.id().with("TreeView"))
					.allow_drag_and_drop(false);
				
				tree.show(ui, |builder| {
					Self::render_dir_item(builder, item, &self.dirs, &mut self.dir_tasks, &self.sender);
				});
			},
			Some(Err(e)) => {
				let text = egui::RichText::new(e.to_string())
					.color(ui.visuals().error_fg_color)
					.monospace();
				ui.label(text);
			}
		}
	}
	
	fn render_dir_item(builder: &mut egui_ltreeview::TreeViewBuilder<BString>, item: &DirItem, dirs: &HashMap<PathBuf, std::io::Result<DirContents>>, dir_tasks: &mut HashMap<PathBuf, Task<()>>, sender: &Sender<TreeLoadMessage>) {
		use egui_ltreeview::NodeBuilder;
		
		let node_id: BString = [
			b"DIR:".as_slice(),
			item.source_path().as_os_str().as_encoded_bytes(),
		].into_iter().collect();
		
		let is_dir = item.is_dir();
		
		let node_builder_start = if is_dir { NodeBuilder::dir } else { NodeBuilder::leaf };
		let node_builder = node_builder_start(node_id.clone()).label(item.display_name());
		
		let is_expanded = builder.node(node_builder);
		
		// got lazy here, started copy-pasting
		if is_dir && is_expanded {
			match dirs.get(item.source_path()) {
				None => {
					let auxillary_id: BString = [
						b"AUX:".as_slice(),
						node_id.as_slice(),
					].into_iter().collect();
					builder.node(NodeBuilder::leaf(auxillary_id).label_ui(|ui| { ui.spinner(); }));
					
					match dir_tasks.entry(item.source_path().to_path_buf()) {
						hash_map::Entry::Vacant(vacant) => {
							vacant.insert(Self::start_dir_load(item.source_path().to_path_buf(), sender.clone()));
						},
						_ => {},
					}
				},
				Some(Ok(contents)) => {
					for item in contents.iter() {
						Self::render_dir_item(builder, item, dirs, dir_tasks, sender);
					}
				},
				Some(Err(e)) => {
					let auxillary_id: BString = [
						b"AUX:".as_slice(),
						node_id.as_slice(),
					].into_iter().collect();
					builder.node(NodeBuilder::leaf(auxillary_id).label_ui(|ui| {
						let text = egui::RichText::new(e.to_string())
							.color(ui.visuals().error_fg_color)
							.monospace();
						ui.label(text);
					}));
				},
			}
		}
		
		if is_dir {
			builder.close_dir();
		}
	}
}
