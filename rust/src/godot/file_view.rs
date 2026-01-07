use godot::prelude::*;
use godot::classes::{Image, image::Format, ImageTexture, Node, INode, TextureRect};

use std::ffi::OsStr;
use std::path::PathBuf;

use crate::filesystem::{cruddy_complex_load, FsItemType};
use crate::godot::browser_tree::{ItemInfo, ItemSource};
use crate::godot::file_view_st::FileViewSt;

#[derive(GodotClass)]
#[class(base=Node)]
struct FileViewController {
	#[export]
	current_view: Option<Gd<Node>>,
	scene_none: OnReady<Gd<PackedScene>>,
	scene_unknown: OnReady<Gd<PackedScene>>,
	scene_image: OnReady<Gd<PackedScene>>,
	base: Base<Node>,
}

#[godot_api]
impl INode for FileViewController {
	fn init(base: Base<Node>) -> Self {
		Self {
			current_view: Default::default(),
			scene_none: OnReady::from_loaded("uid://b0vgfjxmh04oy"),
			scene_unknown: OnReady::from_loaded("uid://bc7du68pcbhuw"),
			scene_image: OnReady::from_loaded("uid://b2ib32jigv6a0"),
			base,
		}
	}
}

#[godot_api]
impl FileViewController {
	#[func]
	fn open_file(&mut self, item_info: Gd<ItemInfo>) {
		let innermost_path = match &item_info.bind().source {
			ItemSource::Fs { path, fs_type } if *fs_type == FsItemType::File => path.clone(),
			ItemSource::Pak { inner_path, .. } => PathBuf::from(inner_path.to_string_lossy().as_ref()),
			_ => { return; },
		};
		
		let new_view = match innermost_path.extension().and_then(OsStr::to_str) {
			Some("pak") => { return; },
			Some("png") => {
				let view = self.scene_image.instantiate().unwrap();
				let data = PackedArray::from(cruddy_complex_load(&item_info.bind().source).unwrap());
				let mut image = Image::create_empty(1, 1, false, Format::L8).unwrap();
				image.load_png_from_buffer(&data);
				let texture = ImageTexture::create_from_image(&image).unwrap();
				view.get_node_as::<TextureRect>("TextureRect").set_texture(Some(&texture));
				view
			},
			Some("stl") => {
				let mut view = FileViewSt::new_alloc();
				view.bind_mut().load_stl_stuff(&item_info.bind().source).unwrap();
				view.upcast()
			},
			_ => self.scene_unknown.instantiate().unwrap(),
		};
		
		if let Some(current_view) = &mut self.current_view {
			current_view.queue_free();
		}
		self.base_mut().add_sibling(&new_view);
		self.current_view = Some(new_view);
	}
}
