use godot::prelude::*;
use godot::classes::{Tree, ITree, TreeItem};
use godot::tools::get_autoload_by_name;

use std::borrow::Cow;
use std::path::PathBuf;

use crate::filesystem::{FsItem, FsItemType, load_directory};
use crate::godot::autoload::GlobalRust;

#[derive(Clone)]
enum ItemSource {
	Fs { path: PathBuf, fs_type: FsItemType },
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
struct ItemInfo {
	source: ItemSource,
	state: ItemState,
}

impl ItemSource {
	fn text(&self) -> Cow<'_, str> {
		match self {
			ItemSource::Fs { path, .. } => path.file_name().unwrap_or_default().to_string_lossy(),
		}
	}
	
	fn can_be_expanded(&self) -> bool {
		match self {
			ItemSource::Fs { fs_type, .. } => *fs_type == FsItemType::Dir,
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
	}
}

#[godot_api]
impl BrowserTree {
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
		
		// Earlier we put a placeholder child so we could expand this item. We don't need it anymore.
		for old_child in item.get_children().iter_shared() {
			old_child.free();
		}
		
		let mut children_sources: Vec<ItemSource> = match &info.source {
			ItemSource::Fs { path, .. } => {
				let fs_items = load_directory(path).unwrap();
				fs_items.into_iter().map(|fs_item| ItemSource::from(fs_item)).collect()
			},
		};
		
		children_sources.sort_by_key(|source| source.text().into_owned());
		
		for source in children_sources {
			let mut child = item.create_child().unwrap();
			self.setup_item(&mut child, source);
		}
		
		info.state = ItemState::Loaded;
	}
}
