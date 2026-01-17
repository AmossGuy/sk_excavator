use eframe::egui::{Id, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileTree {
	root: Option<TreeNode>,
}

struct TreeNode {
	path: PathBuf,
	children: TreeChildren,
}

// TODO: Isn't this just duplicating the functionality of `egui_async::StateWithData`?
// What if TreeNode's children field just held the Bind?
enum TreeChildren {
	Unloaded,
	Loading(egui_async::Bind<Vec<PathBuf>, std::io::Error>),
	Loaded(Vec<TreeNode>),
	Failed(String),
}

impl FileTree {
	pub fn set_root_from_path(&mut self, path: impl AsRef<Path>) {
		self.root = Some(TreeNode {
			path: path.as_ref().to_owned(),
			children: TreeChildren::Unloaded,
		});
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
	fn handle_load(&mut self) {
		match &mut self.children {
			TreeChildren::Unloaded => {
				let bind = egui_async::Bind::new(true);
				self.children = TreeChildren::Loading(bind);
				// Effectively a weird way of expressing fallthrough.
				self.handle_load();
			},
			TreeChildren::Loading(bind) => {
				let path_clone = self.path.clone();
				if let Some(result) = bind.read_or_request(|| async {
					let mut dir = tokio::fs::read_dir(path_clone).await?;
					let mut contents = Vec::new();
					while let Some(entry) = dir.next_entry().await? {
						contents.push(entry.path());
					}
					contents.sort();
					Ok(contents)
				}) {
					self.children = match result {
						Ok(data) => TreeChildren::Loaded(data.iter().map(|path| {
							TreeNode {
								path: path.clone(),
								children: TreeChildren::Unloaded,
							}
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
	fn build(&mut self, builder: &mut TreeViewBuilder<'_, (PathBuf, bool)>, default_open: bool) {
		let text = self.path.file_name().unwrap_or_default().to_string_lossy();
		let dir_node = NodeBuilder::dir((self.path.clone(), false))
			.label(text)
			.default_open(default_open);
		let is_open = builder.node(dir_node);
		
		if is_open {
			self.handle_load();
			
			match &mut self.children {
				TreeChildren::Unloaded => {
					builder.leaf((self.path.clone(), true), "Not loaded");
				},
				TreeChildren::Loading(_) => {
					let spinner_node = NodeBuilder::leaf((self.path.clone(), true))
						.label_ui(|ui| { ui.spinner(); });
					builder.node(spinner_node);
				},
				TreeChildren::Loaded(children) => {
					for child in children {
						child.build(builder, false);
					}
				},
				TreeChildren::Failed(error) => {
					builder.leaf((self.path.clone(), true), format!("Error: {}", error));
				},
			}
		}
		
		builder.close_dir();
	}
}
