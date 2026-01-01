use godot::prelude::*;
use godot::classes::Node;
use godot::tools::get_autoload_by_name;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GlobalRust {
	base: Base<Node>,
}

#[godot_api]
impl GlobalRust {
	#[signal]
	pub fn directory_opened(root_path: GString, child_paths: PackedArray<GString>);
	
	#[func]
	pub fn open_directory(&mut self, path: GString) {
		let entries = match crate::filesystem::get_directory_contents(&path.to_string()) {
			Ok(v) => v,
			Err(e) => {
				godot_error!("I/O error while getting folder contents: {}", e);
				return;
			},
		};
		
		let child_paths = entries.into_iter()
			.map(|e| e.to_str().unwrap().into())
			.collect::<PackedArray<GString>>();
		
		self.signals().directory_opened().emit(&path, &child_paths);
	}
}
