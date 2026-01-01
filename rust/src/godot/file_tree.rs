use godot::prelude::*;
use godot::classes::{Tree, ITree};
use godot::tools::get_autoload_by_name;

use crate::godot::autoload::GlobalRust;

#[derive(GodotClass)]
#[class(init, base=Tree)]
pub struct UiFileTree {
	base: Base<Tree>,
}

#[godot_api]
impl ITree for UiFileTree {
	fn ready(&mut self) {
		let r = get_autoload_by_name::<GlobalRust>("R");
		r.signals().directory_opened().connect_other(&*self, Self::fill_file_tree);
	}
}

#[godot_api]
impl UiFileTree {
	#[func]
	pub fn fill_file_tree(&mut self, root_path: GString, child_paths: PackedArray<GString>) {
		self.base_mut().clear();
		
		let mut root = self.base_mut().create_item().unwrap();
		root.set_text(0, &root_path.get_file());
		
		for child_path in child_paths.as_slice() {
			let mut child = self.base_mut().create_item_ex().parent(&root).done().unwrap();
			child.set_text(0, &child_path.get_file());
		}
	}
}
