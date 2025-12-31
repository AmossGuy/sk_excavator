use godot::prelude::*;
use godot::classes::{Node, Tree};
use godot::tools::get_autoload_by_name;

use std::{ffi::OsStr, path::PathBuf};

#[derive(GodotClass)]
#[class(init, base=Node)]
struct GlobalRust {
	base: Base<Node>,
}

#[godot_api]
impl GlobalRust {
	#[func]
	fn open_directory(&self, path: String) {
		let path = PathBuf::from(path);
		
		let entries = match crate::godot::files::get_directory_contents(&path) {
			Ok(v) => v,
			Err(e) => {
				let message = format!("I/O error while getting folder contents: {}", e);
				get_autoload_by_name::<Node>("U").call("show_error", vslice![message]);
				return;
			},
		};
		
		let scene_tree = self.base().get_tree().unwrap();
		let mut tree = scene_tree.get_current_scene().unwrap().get_node_as::<Tree>("%FileTree");
		
		tree.clear();
		let mut root = tree.create_item().unwrap();
		root.set_text(0, path.file_name().unwrap_or(OsStr::new("")).to_string_lossy().as_ref());
		
		for entry in entries {
			let mut item = tree.create_item_ex().parent(&root).done().unwrap();
			item.set_text(0, entry.file_name().unwrap_or(OsStr::new("")).to_string_lossy().as_ref());
		}
	}
}
