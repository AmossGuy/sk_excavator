use excavator_backend::file_tree::{self, FileTreeBackend, TreeNode};
use excavator_backend::io::LoadState;
use excavator_backend::request_thread::Waker;

use egui_ltreeview::{NodeBuilder, TreeViewBuilder};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Eq, Hash, PartialEq)]
enum NodeId {
	Node(file_tree::UniqueId),
	Aux,
}

pub struct FileTreeView {
	root_path: PathBuf,
	// search_text: String,
	
	backend: FileTreeBackend<RepaintWaker>,
}

#[derive(Default)]
#[must_use]
pub struct FileTreeEffect {
	pub close_clicked: bool,
}

#[derive(Clone)]
struct RepaintWaker {
	ctx: Option<egui::Context>,
}

impl RepaintWaker {
	pub fn dummy() -> Self {
		Self { ctx: None }
	}
	
	pub fn new(ctx: &egui::Context) -> Self {
		Self { ctx: Some(ctx.clone()) }
	}
}

impl Waker for RepaintWaker {
	fn wake(&self) {
		if let Some(ctx) = &self.ctx {
			ctx.request_repaint();
		}
	}
}

impl FileTreeView {
	pub fn new(root_path: PathBuf) -> Self {
		let backend = FileTreeBackend::new(root_path.clone(), RepaintWaker::dummy());
		Self {
			root_path,
			// search_text: String::new(),
			backend,
		}
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) -> FileTreeEffect {
		self.backend.replace_waker(RepaintWaker::new(ui.ctx()));
		
		let effect = self.fixed_ui(ui);
		
		ui.separator();
		
		egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
			self.scrolling_ui(ui);
		});
		
		effect
	}
		
	fn fixed_ui(&mut self, ui: &mut egui::Ui) -> FileTreeEffect {
		let mut effect = FileTreeEffect::default();
		
		// let layout = egui::Layout::right_to_left(egui::Align::Min);
		let layout = egui::Layout::left_to_right(egui::Align::Min);
		ui.scope_builder(egui::UiBuilder::new().layout(layout), |ui| {
			if ui.button("Close").clicked() {
				effect.close_clicked = true;
			}
			if ui.button("Reload").clicked() {
				*self = Self::new(self.root_path.clone());
			}
			
			// search is not implemented, because i had a horrible time trying to
			/*
			let search_box_size = egui::Vec2::new(ui.available_size().x, ui.min_size().y);
			let search_box = egui::TextEdit::singleline(&mut self.search_text)
				.hint_text("Search...");
			ui.add_sized(search_box_size, search_box);
			*/
		});
		
		effect
	}
	
	fn scrolling_ui(&mut self, ui: &mut egui::Ui) {
		let tree = egui_ltreeview::TreeView::new(ui.id().with("TreeView"))
			.allow_drag_and_drop(false);
		
		tree.show(ui, |builder| {
			let root_state = self.backend.tree_root();
			
			if let Some(root_node) = self.render_load_state(builder, &root_state) {
				self.render_node(builder, &root_node);
			}
		});
	}
	
	#[must_use]
	fn render_load_state<'a, 'b, T>(&mut self, builder: &mut TreeViewBuilder<'a, NodeId>, state: &'b LoadState<T>) -> Option<&'b T> {
		match &*state {
			LoadState::Unloaded => {
				builder.leaf(NodeId::Aux, "(Not loaded)");
				None
			},
			LoadState::Loading => {
				builder.node(NodeBuilder::leaf(NodeId::Aux).label_ui(|ui| { ui.spinner(); }));
				None
			},
			LoadState::Loaded(loaded) => {
				Some(&loaded)
			},
			LoadState::Failed(e) => {
				builder.node(NodeBuilder::leaf(NodeId::Aux).label_ui(|ui| {
					let text = egui::RichText::new(e.to_string())
						.color(ui.visuals().error_fg_color)
						.monospace();
					ui.label(text);
				}));
				None
			},
		}
	}
		
	fn render_node<'a>(&mut self, builder: &mut TreeViewBuilder<'a, NodeId>, node: &Arc<Mutex<TreeNode>>) {
		let node_lock = node.lock().unwrap();
		let is_expandable = node_lock.is_expandable();
		let node_id = NodeId::Node(node_lock.unique_id());
		let display_name = node_lock.display_name();
		let children_state = node_lock.children();
		drop(node_lock);
		
		let node_builder_start = if is_expandable { NodeBuilder::dir } else { NodeBuilder::leaf };
		let node_builder = node_builder_start(node_id).label(display_name).default_open(false);
		
		let is_expanded = builder.node(node_builder);
		
		if is_expandable && is_expanded {
			self.backend.start_load_if_unloaded(node);
			
			if let Some(children) = self.render_load_state(builder, &children_state) {
				for child in children.iter() {
					self.render_node(builder, child);
				}
			}
		}
		
		if is_expandable {
			builder.close_dir();
		}
	}
}
