use godot::prelude::*;
use godot::classes::Node;

use crate::godot::format_resources::SkePak;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GlobalRust {
	base: Base<Node>,
}

#[godot_api]
impl GlobalRust {
	#[signal]
	pub fn directory_opened(root_path: GString);
	
	#[func]
	pub fn open_directory(&mut self, path: GString) {
		self.signals().directory_opened().emit(&path);
	}
	
	#[func]
	pub fn open_file(&mut self, path: GString) -> Option<Gd<Resource>> {
		let extension = path.get_extension();
		match extension.to_string().as_ref() {
			"pak" => Some(SkePak::open_file(path).upcast()),
			_ => None,
		}
	}
}
