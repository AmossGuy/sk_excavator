use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::ExcavatorMessage;
use crate::plugins::ThreadSpawner;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum ItemInfo {
	Fs { path: PathBuf, kind: FsItemKind },
	Pak { outer_path: PathBuf, inner_path: CString },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum FsItemKind {
	Directory,
	File,
	Other,
}

impl ItemInfo {
	pub fn file_name_lossy(&self) -> Option<Cow<'_, str>> {
		match self {
			Self::Fs { path, .. } => path.file_name().map(|s| s.to_string_lossy()),
			Self::Pak { inner_path, .. } => Some(inner_path.to_string_lossy()),
		}
	}
	
	// Returning a slice of u8 is for the sake of having a shared type in both cases.
	// It's fine since we only need to handle ASCII extensions.
	pub fn extension(&self) -> Option<&[u8]> {
		match self {
			Self::Fs { path, .. } => path.extension().map(|e| e.as_encoded_bytes()),
			Self::Pak { inner_path, .. } => {
				// A simple, good-enough implementation
				let file_name = inner_path.as_bytes().split(|b| *b == b'/' || *b == b'\\').last();
				file_name.and_then(|f| f.split(|b| *b == b'.').last())
			},
		}
	}
	
	// As opposed to being a directory or etcetra
	pub fn is_file(&self) -> bool {
		match self {
			Self::Fs { kind, .. } => matches!(kind, FsItemKind::File),
			Self::Pak { .. } => true,
		}
	}
	
	pub fn outer_path(&self) -> &PathBuf {
		match self {
			Self::Fs { path, .. } => &path,
			Self::Pak { outer_path, .. } => &outer_path,
		}
	}
}

impl From<&std::fs::FileType> for FsItemKind {
	fn from(value: &std::fs::FileType) -> Self {
		if value.is_dir() {
			Self::Directory
		} else if value.is_file() {
			Self::File
		} else {
			Self::Other
		}
	}
}

impl From<&std::fs::Metadata> for FsItemKind {
	fn from(value: &std::fs::Metadata) -> Self {
		Self::from(&value.file_type())
	}
}





enum FsLoadState {
	Loading,
	Done(std::sync::Weak<LoadResult>),
}

pub type LoadResult = Result<LoadedData, Box<str>>;

#[derive(Debug)]
pub enum LoadedData {
	Dir(Box<[DirEntry]>),
}

#[derive(Clone, Debug)]
pub struct DirEntry {
	pub path: PathBuf,
	pub metadata: std::fs::Metadata,
}

#[derive(Default)]
pub struct ItemLoader {
	fs_items: Arc<Mutex<HashMap<PathBuf, FsLoadState>>>,
}

impl ItemLoader {
	// This function exists so the or_else closure can use the same lock, ensuring that no external modification can occur in the middle of this logic.
	fn get_or_else<F>(&self, item: &ItemInfo, f: F) -> Option<Arc<LoadResult>>
	where
		F: FnOnce(&mut HashMap<PathBuf, FsLoadState>) -> Option<Arc<LoadResult>>,
	{
		let path = item.outer_path();
		let mut fs_items = self.fs_items.lock().unwrap();
		match fs_items.get(path) {
			Some(FsLoadState::Done(weak)) => {
				let strong = weak.upgrade();
				if strong.is_none() { fs_items.remove(path); }
				strong
			},
			_ => None,
		}.or_else(|| f(&mut fs_items))
	}
	
	#[expect(dead_code)] // This'll probably be useful somewhere
	pub fn get(&self, item: &ItemInfo) -> Option<Arc<LoadResult>> {
		self.get_or_else(item, |_| { None })
	}
	
	pub fn get_or_request(&self, item: &ItemInfo, ctx: &egui::Context) -> Option<Arc<LoadResult>> {
		self.get_or_else(item, |fs_items| {
			let path = item.outer_path();
			if !fs_items.contains_key(path) {
				fs_items.insert(path.to_owned(), FsLoadState::Loading);
				self.start_load(item.clone(), ctx.clone());
			}
			None
		})
	}
	
	fn start_load(&self, item: ItemInfo, ctx: egui::Context) {
		let fs_items = Arc::clone(&self.fs_items);
		let spawner = ctx.plugin_or_default::<ThreadSpawner>();
		spawner.lock().spawn(ctx, move |_| {
			let result = Arc::new(Self::do_load(&item));
			fs_items.lock().unwrap().insert(
				item.outer_path().to_owned(),
				FsLoadState::Done(Arc::downgrade(&result)),
			);
			Some(ExcavatorMessage::ItemLoadDone { item, result })
		});
	}
	
	fn do_load(item: &ItemInfo) -> LoadResult {
		let contents = match item {
			ItemInfo::Fs { path, kind: FsItemKind::Directory } => {
				let contents = std::fs::read_dir(path)
					.map_err(|e| e.to_string())?
					.map(|entry| entry.and_then(|entry| Ok(DirEntry {
						path: entry.path(),
						metadata: entry.metadata()?,
					})))
					.collect::<Result<_, _>>()
					.map_err(|e| e.to_string())?;
				LoadedData::Dir(contents)
			},
			_ => {
				println!("do_load todo: {:?}", item);
				LoadedData::Dir(Default::default())
			},
		};
		Ok(contents)
	}
}
