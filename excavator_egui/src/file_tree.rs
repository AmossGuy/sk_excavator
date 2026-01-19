use egui::{Id, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder};
use lexical_sort::natural_lexical_cmp;
use std::path::{Path, PathBuf};

use crate::file_read::FileLocation;

#[derive(Default)]
pub struct FileTree {
	root: Option<TreeNode>,
}

struct TreeNode {
	location: FileLocation,
	kind: NodeKind,
	children: TreeChildren,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum NodeKind {
	FsDirectory,
	FsFile,
	FsOther,
}

// TODO: Isn't this just duplicating the functionality of `egui_async::StateWithData`?
// What if TreeNode's children field just held the Bind?
enum TreeChildren {
	Unloaded,
	Loading(egui_async::Bind<Vec<(FileLocation, NodeKind)>, std::io::Error>),
	Loaded(Vec<TreeNode>),
	Failed(String),
}

impl FileTree {
	pub fn set_root_from_path(&mut self, path: impl AsRef<Path>) {
		let location = FileLocation::from(path.as_ref());
		self.root = Some(TreeNode::new(location, NodeKind::FsDirectory));
	}
	
	pub fn set_root_from_path_if_different(&mut self, path: impl AsRef<Path>) {
		let is_same_path = match &self.root {
			Some(root) => root.location == path.as_ref(),
			None => false,
		};
		if !is_same_path {
			self.set_root_from_path(path);
		}
	}
	
	pub fn add_view(&mut self, ui: &mut Ui) {
		if let Some(root) = &mut self.root {
			TreeView::new(Id::new("browser tree")).show(ui, |builder| {
				root.build(builder, true);
			});
		} else {
			ui.label("No directory has been opened.");
		}
	}
}

impl TreeNode {
	fn new(location: FileLocation, kind: NodeKind) -> Self {
		Self {
			location: location.clone(),
			kind: kind,
			children: TreeChildren::Unloaded,
		}
	}
	
	fn handle_load(&mut self) {
		match &mut self.children {
			TreeChildren::Unloaded => {
				let bind = egui_async::Bind::new(true);
				self.children = TreeChildren::Loading(bind);
				// Effectively a weird way of expressing fallthrough.
				self.handle_load();
			},
			TreeChildren::Loading(bind) => {
				let Ok(path_clone) = PathBuf::try_from(self.location.clone()) else {
					// TODO: Implement reading file list from .pak archives.
					// I should probably do something to make this more readable while I'm at it.
					// Also move it to file_read.rs, probably.
					todo!()
				};
				if let Some(result) = bind.read_or_request(|| async {
					let mut dir = tokio::fs::read_dir(path_clone).await?;
					let mut contents = Vec::new();
					while let Some(entry) = dir.next_entry().await? {
						let e_path = entry.path();
						let e_metadata = entry.metadata().await?;
						contents.push((FileLocation::from(e_path), NodeKind::from(&e_metadata)));
					}
					contents.sort_by(|a, b| natural_lexical_cmp(
						&a.0.file_name().unwrap_or_default(),
						&b.0.file_name().unwrap_or_default(),
					));
					Ok(contents)
				}) {
					self.children = match result {
						Ok(data) => TreeChildren::Loaded(data.iter().map(|(location, kind)| {
							Self::new(location.clone(), *kind)
						}).collect()),
						Err(error) => TreeChildren::Failed(error.to_string()),
					};
				}
			},
			_ => {},
		}
	}
	
	// `self` being mutable here is a tad quirky.
	// It's only like that so `handle_load` can be called here.
	fn build(&mut self, builder: &mut TreeViewBuilder<'_, (FileLocation, bool)>, default_open: bool) {
		let id = (self.location.clone(), false);
		let text = self.location.file_name().unwrap_or_default();
		let is_openable = self.kind == NodeKind::FsDirectory;
		
		let node = if is_openable {
			NodeBuilder::dir(id)
		} else {
			NodeBuilder::leaf(id)
		};
		let node = node.label(text).default_open(default_open);
		let is_open = builder.node(node);
		
		if is_openable && is_open {
			self.handle_load();
			
			match &mut self.children {
				TreeChildren::Unloaded => {
					builder.leaf((self.location.clone(), true), "Not loaded");
				},
				TreeChildren::Loading(_) => {
					let spinner_node = NodeBuilder::leaf((self.location.clone(), true))
						.label_ui(|ui| { ui.spinner(); });
					builder.node(spinner_node);
				},
				TreeChildren::Loaded(children) => {
					for child in children {
						child.build(builder, false);
					}
				},
				TreeChildren::Failed(error) => {
					builder.leaf((self.location.clone(), true), format!("Error: {}", error));
				},
			}
		}
		
		if is_openable {
			builder.close_dir();
		}
	}
}

impl From<&std::fs::Metadata> for NodeKind {
	fn from(value: &std::fs::Metadata) -> Self {
		if value.is_dir() {
			Self::FsDirectory
		} else if value.is_file() {
			Self::FsFile
		} else {
			Self::FsOther
		}
	}
}
