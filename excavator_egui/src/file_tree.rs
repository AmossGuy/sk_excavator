use egui::Ui;
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder};
use lexical_sort::natural_lexical_cmp;

use std::path::Path;
use std::sync::Arc;

use crate::file_read::{FsItemKind, ItemInfo, ItemLoader, LoadedData, LoadResult};

#[derive(Default)]
pub struct FileTree {
	root: Option<TreeNode>,
}

struct TreeNode {
	source: ItemInfo,
	children: TreeChildren,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum ExpandHandler {
	Directory,
	PakArchive,
}

// TODO: Isn't this just duplicating the functionality of `egui_async::StateWithData`?
// What if TreeNode's children field just held the Bind?
enum TreeChildren {
	Unloaded,
	Loading,
	Loaded(Box<[TreeNode]>),
	Failed(Box<str>),
}

impl FileTree {
	pub fn set_root_from_path(&mut self, path: impl AsRef<Path>) {
		self.root = Some(TreeNode::new(ItemInfo::Fs {
			path: path.as_ref().to_owned(),
			kind: FsItemKind::Directory,
		}));
	}
	
	pub fn set_root_from_path_if_different(&mut self, path: impl AsRef<Path>) {
		let is_same_path = match &self.root {
			Some(root) => match &root.source {
				ItemInfo::Fs { path: location_path, .. } => location_path == path.as_ref(),
				_ => false,
			},
			None => false,
		};
		if !is_same_path {
			self.set_root_from_path(path);
		}
	}
	
	pub fn update_from_load(&mut self, item: ItemInfo, load_result: Arc<LoadResult>) {
		if let Some(root) = &mut self.root {
			root.update_from_load(&item, &load_result);
		}
	}
	
	pub fn add_view(&mut self, ui: &mut Ui, loader: &ItemLoader) -> Option<Vec<ItemInfo>> {
		if let Some(root) = &mut self.root {
			let view = TreeView::new(ui.make_persistent_id("file tree"))
				.fallback_context_menu(Self::context_menu);
			
			let ctx2 = ui.ctx().clone();
			let (_, actions) = view.show(ui, |builder| {
				root.build(builder, loader, &ctx2, true);
			});
			
			let mut selection_update = None;
			for action in actions {
				match action {
					egui_ltreeview::Action::SetSelected(s) => {
						selection_update = Some(s.into_iter().map(|x| x.0).collect());
					},
					_ => {},
				}
			}
			selection_update
		} else {
			ui.label("No directory has been opened.");
			None
		}
	}
	
	fn context_menu(ui: &mut Ui, nodes: &Vec<(ItemInfo, bool)>) {
		let text_size = egui::TextStyle::Body
			.resolve(ui.style())
			.size;
		ui.set_width(text_size * 15.); // UGH. stupid
		
		let node = match nodes.len() {
			0 => { ui.label("Nothing selected"); return; },
			1 => &nodes[0],
			2.. => { ui.label("Multiselect actions not yet implemented"); return; },
		};
		
		let item = &node.0;
		match item {
			ItemInfo::Fs { path, kind: _ } => {
				if ui.button("Copy path").clicked() {
					ui.ctx().copy_text(path.to_string_lossy().into_owned());
				}
			},
			ItemInfo::Pak { inner_path: _, outer_path: _ } => {
				/*
				if ui.button("Extract from archive").clicked() {
				}
				*/
			},
		};
	}
}

impl TreeNode {
	fn new(source: ItemInfo) -> Self {
		Self { source, children: TreeChildren::Unloaded }
	}
	
	fn expand_handler(&self) -> Option<ExpandHandler> {
		match &self.source {
			ItemInfo::Fs { kind, .. } => match kind {
				FsItemKind::Directory => Some(ExpandHandler::Directory),
				FsItemKind::File => match self.source.extension() {
					Some(b"pak") => Some(ExpandHandler::PakArchive),
					_ => None,
				}
				_ => None,
			},
			_ => None,
		}
	}
	
	fn handle_load(&mut self, loader: &ItemLoader, ctx: &egui::Context) {
		let expand_handler = self.expand_handler(); // There was a lifetime issue...
		
		match &mut self.children {
			TreeChildren::Unloaded | TreeChildren::Loading => {
				self.children = TreeChildren::Loading;
				
				let Some(_) = expand_handler else {
					unreachable!("Nodes without expand handler shouldn't be expandable")
				};
				let ItemInfo::Fs { path: _, .. } = &self.source else {
					unreachable!("Nodes with non-filesystem source shouldn't be expandable")
				};
				if let Some(result) = loader.get_or_request(&self.source, ctx) {
					self.setup_children(result);
				}
			},
			_ => {},
		}
	}
	
	fn update_from_load(&mut self, item: &ItemInfo, load_result: &Arc<LoadResult>) {
		if *item == self.source {
			self.setup_children(Arc::clone(load_result));
		}
		
		match self.children {
			TreeChildren::Loaded(ref mut children) => for child in children.iter_mut() {
				child.update_from_load(item, load_result);
			},
			_ => {},
		};
	}
		
	fn setup_children(&mut self, load_result: Arc<LoadResult>) {
		self.children = match *load_result {
			Ok(LoadedData::Dir(ref entries)) => {
				let mut entries = entries.clone();
				entries.sort_unstable_by(|lhs, rhs| natural_lexical_cmp(
					&lhs.path.to_string_lossy(), &rhs.path.to_string_lossy(),
				));
				TreeChildren::Loaded(entries.iter().map(|entry| {
					Self::new(ItemInfo::Fs {
						path: entry.path.clone(),
						kind: FsItemKind::from(&entry.metadata),
					})
				}).collect())
			},
			Ok(LoadedData::PakListing(ref entries)) => {
				let mut entries = entries.clone();
				entries.sort_unstable_by(|lhs, rhs| natural_lexical_cmp(
					&lhs.name.to_string_lossy(), &rhs.name.to_string_lossy(),
				));
				TreeChildren::Loaded(entries.iter().map(|entry| {
					Self::new(ItemInfo::Pak {
						outer_path: self.source.outer_path().clone(),
						inner_path: entry.name.clone(),
					})
				}).collect())
			},
			Err(ref e) => TreeChildren::Failed(e.clone()),
		};
	}
	
	// `self` being mutable here is a tad quirky.
	// It's only like that so `handle_load` can be called here.
	//
	// ...It's gotten even worse since I wrote that. It's seeming like it would be better if the ui and loading logic were separated.
	fn build(&mut self, builder: &mut TreeViewBuilder<'_, (ItemInfo, bool)>, loader: &ItemLoader, ctx: &egui::Context, default_open: bool) {
		let id = (self.source.clone(), false);
		let text = self.source.file_name_lossy().unwrap_or_default();
		let is_openable = self.expand_handler().is_some();
		
		let node = if is_openable {
			NodeBuilder::dir(id)
		} else {
			NodeBuilder::leaf(id)
		};
		let node = node.label(text).default_open(default_open);
		let is_open = builder.node(node);
		
		if is_openable && is_open {
			self.handle_load(loader, ctx);
			
			match &mut self.children {
				TreeChildren::Unloaded => {
					builder.leaf((self.source.clone(), true), "Not loaded");
				},
				TreeChildren::Loading => {
					let spinner_node = NodeBuilder::leaf((self.source.clone(), true))
						.label_ui(|ui| { ui.spinner(); });
					builder.node(spinner_node);
				},
				TreeChildren::Loaded(children) => {
					for child in children {
						child.build(builder, loader, ctx, false);
					}
				},
				TreeChildren::Failed(error) => {
					builder.leaf((self.source.clone(), true), format!("Error: {}", error));
				},
			}
		}
		
		if is_openable {
			builder.close_dir();
		}
	}
}
