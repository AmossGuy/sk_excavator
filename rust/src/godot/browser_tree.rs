use godot::prelude::*;
use godot::classes::{Tree, ITree, TreeItem};
use godot::tools::get_autoload_by_name;

use std::borrow::Cow;
use std::ffi::CString;
use std::path::PathBuf;

use crate::filesystem::{FsItem, FsItemType, load_directory, open_file};
use crate::formats::{FileType, pak::PakIndex};
use crate::godot::autoload::GlobalRust;

#[derive(Clone)]
pub enum ItemSource {
	Fs { path: PathBuf, fs_type: FsItemType },
	Pak { outer_path: PathBuf, inner_path: CString },
}

impl From<FsItem> for ItemSource {
	fn from(value: FsItem) -> Self {
		Self::Fs { path: value.path, fs_type: value.item_type }
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ItemState {
	Unloaded,
	Loaded,
}

#[derive(GodotClass, Clone)]
#[class(no_init)]
// Right now this struct is doing double duty in an inelegant way. I'm using it as a wrapper that can pass through Godot to places that only need the `source` field.
// The `state` field *only* refers to whether the item's children have been added to the browser tree.
pub struct ItemInfo {
	pub source: ItemSource,
	state: ItemState,
}

impl ItemSource {
	fn text(&self) -> Cow<'_, str> {
		match self {
			ItemSource::Fs { path, .. } => path.file_name().unwrap_or_default().to_string_lossy(),
			ItemSource::Pak { inner_path, .. } => inner_path.to_string_lossy(),
		}
	}
	
	fn can_be_expanded(&self) -> bool {
		match self {
			ItemSource::Fs { path, fs_type } => match fs_type {
				FsItemType::Dir => true,
				FsItemType::File => FileType::from_path(path) == FileType::Pak,
				_ => false,
			},
			ItemSource::Pak { .. } => false,
		}
	}
}

#[derive(GodotClass)]
#[class(init, base=Tree)]
pub struct BrowserTree {
	base: Base<Tree>,
}

#[godot_api]
impl ITree for BrowserTree {
	fn ready(&mut self) {
		let r = get_autoload_by_name::<GlobalRust>("R");
		r.signals().directory_opened().connect_other(&*self, Self::show_directory);
		
		self.signals().item_collapsed().connect_self(Self::on_item_collapsed);
		self.signals().item_selected().connect_self(Self::on_item_activated);
	}
}

#[godot_api]
impl BrowserTree {
	#[signal]
	fn file_open_requested(item_info: Gd<ItemInfo>);
	
	fn setup_item(&mut self, item: &mut Gd<TreeItem>, source: ItemSource) {
		let info = ItemInfo { source, state: ItemState::Unloaded };
		let mut set_collapsed: Option<bool> = None;
		
		item.set_text(0, info.source.text().as_ref());
		if info.source.can_be_expanded() {
			item.create_child();
			set_collapsed = Some(true);
		}
		item.set_metadata(0, &Gd::from_object(info).to_variant());
		
		// The item_collapsed signal going off in this function causes invalid aliasing
		// We avoid this by disabling signals briefly when it otherwise would be sent
		if let Some(value) = set_collapsed {
			let previous = self.base().is_blocking_signals();
			self.base_mut().set_block_signals(true);
			
			item.set_collapsed(value);
			
			self.base_mut().set_block_signals(previous);
		}
	}
	
	#[func]
	fn show_directory(&mut self, path: GString) {
		let path = PathBuf::from(path.to_string());
		
		self.base_mut().clear();
		let mut root = self.base_mut().create_item().unwrap();
		self.setup_item(&mut root, ItemSource::Fs { path, fs_type: FsItemType::Dir });
		root.call_deferred("set_collapsed", vslice![false]);
	}
	
	#[func]
	fn on_item_collapsed(&mut self, mut item: Gd<TreeItem>) {
		// Despite the name of the signal, we specifically want to respond to an item being *expanded*.
		if item.is_collapsed() { return; }
		
		let Ok(mut info_gd) = item.get_metadata(0).try_to::<Gd<ItemInfo>>() else { return; };
		let mut info = info_gd.bind_mut();
		
		if info.state != ItemState::Unloaded { return; }
		
		let mut children_sources: Vec<ItemSource> = match &info.source {
			ItemSource::Fs { path, fs_type } => {
				match fs_type {
					FsItemType::Dir => {
						let fs_items = load_directory(path).unwrap();
						fs_items.into_iter().map(|fs_item| ItemSource::from(fs_item)).collect()
					},
					FsItemType::File if FileType::from_path(path) == FileType::Pak => {
						let mut reader = open_file(path).unwrap();
						let index = PakIndex::create_index(&mut reader).unwrap();
						index.files.into_iter().map(|file| {
							ItemSource::Pak { outer_path: path.clone(), inner_path: file.0 }
						}).collect()
					},
					_ => unreachable!(),
				}
			},
			_ => unreachable!(),
		};
		
		children_sources.sort_by_key(|source| source.text().into_owned());
		
		// Earlier we put a placeholder child so we could expand this item. We don't need it anymore.
		for old_child in item.get_children().iter_shared() {
			old_child.free();
		}
		
		if children_sources.len() == 0 {
			let mut child = item.create_child().unwrap();
			child.set_text(0, "(Empty)");
		}
		
		for source in children_sources {
			let mut child = item.create_child().unwrap();
			self.setup_item(&mut child, source);
		}
		
		info.state = ItemState::Loaded;
	}
	
	fn on_item_activated(&mut self) {
		let item = self.base().get_selected().unwrap();
		let Ok(info_gd) = item.get_metadata(0).try_to::<Gd<ItemInfo>>() else { return; };
		
		self.signals().file_open_requested().emit(&info_gd);
	}
}
