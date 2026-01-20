use egui::{Id, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder};
use lexical_sort::natural_lexical_cmp;
use std::path::Path;

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
	ArchivedFile,
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
	Loading(egui_async::Bind<Vec<(FileLocation, NodeKind)>, String>),
	Loaded(Vec<TreeNode>),
	Failed(String),
}

impl FileTree {
	pub fn set_root_from_path(&mut self, path: impl AsRef<Path>) {
		let location = FileLocation::Fs { path: path.as_ref().to_owned() };
		self.root = Some(TreeNode::new(location, NodeKind::FsDirectory));
	}
	
	pub fn set_root_from_path_if_different(&mut self, path: impl AsRef<Path>) {
		let is_same_path = match &self.root {
			Some(root) => match &root.location {
				FileLocation::Fs { path: location_path } => location_path == path.as_ref(),
				_ => false,
			},
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
	
	fn expand_handler(&self) -> Option<ExpandHandler> {
		match self.kind {
			NodeKind::FsDirectory => Some(ExpandHandler::Directory),
			NodeKind::FsFile if self.location.extension() == Some(b"pak") => Some(ExpandHandler::PakArchive),
			_ => None,
		}
	}
	
	fn handle_load(&mut self) {
		let expand_handler = self.expand_handler(); // There was a lifetime issue...
		
		match &mut self.children {
			TreeChildren::Unloaded => {
				let bind = egui_async::Bind::new(true);
				self.children = TreeChildren::Loading(bind);
				// Effectively a weird way of expressing fallthrough.
				self.handle_load();
			},
			TreeChildren::Loading(bind) => {
				let Some(expand_handler) = expand_handler else {
					unreachable!("Nodes without expand handler shouldn't be expandable")
				};
				let FileLocation::Fs { path } = &self.location else {
					unreachable!("Nodes with non-filesystem location shouldn't be expandable")
				};
				let thingy = match expand_handler {
					ExpandHandler::Directory => bind.read_or_request(|| read_node_contents_dir(path.clone())),
					ExpandHandler::PakArchive => bind.read_or_request(|| read_node_contents_pak(path.clone())),
				};
				if let Some(result) = thingy {
					self.children = match result {
						Ok(data) => TreeChildren::Loaded(data.iter().map(|(location, kind)| {
							Self::new(location.clone(), *kind)
						}).collect()),
						Err(error) => TreeChildren::Failed(error.clone()),
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
		let text = self.location.file_name_lossy().unwrap_or_default();
		let is_openable = self.expand_handler().is_some();
		
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

async fn read_node_contents_dir(path: impl AsRef<Path>) -> Result<Vec<(FileLocation, NodeKind)>, String> {
	let mut dir = tokio::fs::read_dir(path).await.map_err(|e| e.to_string())?;
	let mut contents = Vec::new();
	while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
		let e_path = entry.path();
		let e_metadata = entry.metadata().await.map_err(|e| e.to_string())?;
		contents.push((FileLocation::Fs { path: e_path }, NodeKind::from(&e_metadata)));
	}
	contents.sort_by(|a, b| natural_lexical_cmp(
		&a.0.file_name_lossy().unwrap_or_default(),
		&b.0.file_name_lossy().unwrap_or_default(),
	));
	Ok(contents)
}

async fn read_node_contents_pak(pak_path: impl AsRef<Path>) -> Result<Vec<(FileLocation, NodeKind)>, String> {
	use std::{fs::File, io::BufReader}; // lol
	
	let pak_path_clone = pak_path.as_ref().to_owned();
	let handle = egui_async::bind::ASYNC_RUNTIME.spawn_blocking(move || {
		let file = File::open(&pak_path_clone).map_err(|e| e.to_string())?;
		let mut reader = BufReader::new(file);
		let index = excavator_formats::pak::PakIndex::create_index(&mut reader).map_err(|e| e.to_string())?;
		Ok(index.files.iter().map(|f| {
			let location = FileLocation::Pak { outer_path: pak_path_clone.clone(), inner_path: f.0.clone() };
			(location, NodeKind::ArchivedFile)
		}).collect::<Vec<_>>())
	});
	handle.await.unwrap() // I don't think there's any way for a JoinError to occur
}
