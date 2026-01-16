use eframe::egui::{Id, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileTree {
	root: Option<TreeNode>,
}

impl FileTree {
	pub fn set_root_from_path(&mut self, path: impl AsRef<Path>) {
		self.root = Some(TreeNode {
			path: path.as_ref().to_owned(),
			children: TreeChildren::Unloaded,
		});
	}
}

#[derive(Default)]
struct IdGiver {
	next_id: usize,
}

impl IdGiver {
	fn next(&mut self) -> usize {
		let id = self.next_id;
		self.next_id += 1;
		id
	}
}

struct TreeNode {
	path: PathBuf,
	children: TreeChildren,
}

impl TreeNode {
	fn build(&self, id_giver: &mut IdGiver, builder: &mut TreeViewBuilder<'_, usize>) {
		let text = self.path.file_name().unwrap_or_default().to_string_lossy();
		builder.dir(id_giver.next(), text);
		
		match &self.children {
			TreeChildren::Unloaded => {
				builder.leaf(id_giver.next(), "Not loaded");
			},
			TreeChildren::Loading => {
				let spinner_node = NodeBuilder::leaf(id_giver.next())
					.label_ui(|ui| { ui.spinner(); });
				builder.node(spinner_node);
			},
			TreeChildren::Loaded(children) => {
				for child in children {
					child.build(id_giver, builder);
				}
			},
		}
		
		builder.close_dir();
	}
}

enum TreeChildren {
	Unloaded,
	Loading,
	Loaded(Vec<TreeNode>),
}

pub fn add_file_treeview(ui: &mut Ui, tree: &mut FileTree) {
	TreeView::new(Id::new("browser tree")).show(ui, |builder| {
		if let Some(root) = &tree.root {
			let mut id_giver = IdGiver::default();
			root.build(&mut id_giver, builder);
		}
	});
}
