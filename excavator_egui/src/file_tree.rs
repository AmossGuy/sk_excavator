use excavator_backend::file_tree::FileTreeBackend;
use excavator_backend::io::{dir::DirItem, LoadState};
use excavator_backend::request_thread::Waker;
use std::path::PathBuf;
use std::rc::Rc;

use egui_ltreeview::{NodeBuilder, TreeViewBuilder};

// Those type signatures are pretty bad even in abstract, but with this shorthand they're at least readable.
type Build<'a> = TreeViewBuilder<'a, NodeId>;
type Back = FileTreeBackend<RepaintWaker>;

#[derive(Clone, Eq, Hash, PartialEq)]
enum NodeId {
	Dir(PathBuf),
	Aux,
}

pub struct FileTreeView {
	root_path: PathBuf,
	search_text: String,
	
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
			search_text: String::new(),
			backend,
		}
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) -> FileTreeEffect {
		self.backend.replace_waker(RepaintWaker::new(ui.ctx()));
		self.backend.update_loading();
		
		let effect = self.fixed_ui(ui);
		egui::Frame::group(ui.style()).show(ui, |ui| {
			egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
				self.scrolling_ui(ui);
			});
		});
		
		effect
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
		let tree = egui_ltreeview::TreeView::new(ui.id().with("TreeView"))
			.allow_drag_and_drop(false);
		
		tree.show(ui, |builder| {
			let root = Rc::clone(self.backend.request_root());
			let backend = &mut self.backend;
			Self::render_load_state(builder, backend, root, Self::render_dir_item);
		});
	}
	
	fn render_load_state<'a, T>(builder: &mut Build<'a>, backend: &mut Back, state: Rc<LoadState<T>>, render_loaded: impl FnOnce(&mut Build<'a>, &mut Back, &T)) {
		match &*state {
			LoadState::Loading => {
				builder.node(NodeBuilder::leaf(NodeId::Aux).label_ui(|ui| { ui.spinner(); }));
			},
			LoadState::Loaded(loaded) => {
				render_loaded(builder, backend, &loaded);
			},
			LoadState::Failed(e) => {
				builder.node(NodeBuilder::leaf(NodeId::Aux).label_ui(|ui| {
					let text = egui::RichText::new(e.to_string())
						.color(ui.visuals().error_fg_color)
						.monospace();
					ui.label(text);
				}));
			},
		}
	}
	
	fn render_dir_item(builder: &mut Build<'_>, backend: &mut Back, item: &DirItem) {
		let is_dir = item.is_dir();
		let path = item.file_path().to_path_buf();
		
		let node_builder_start = if is_dir { NodeBuilder::dir } else { NodeBuilder::leaf };
		let node_builder = node_builder_start(NodeId::Dir(path.clone())).label(item.display_name());
		
		let is_expanded = builder.node(node_builder);
		
		if is_dir && is_expanded {
			let contents = Rc::clone(backend.request_dir(path));
			Self::render_load_state(builder, backend, contents, |bu, ba, contents| {
				for child in contents.iter() {
					Self::render_dir_item(bu, ba, child);
				}
			});
		}
		
		if is_dir {
			builder.close_dir();
		}
	}
}
