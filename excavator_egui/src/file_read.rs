use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::ExcavatorMessage;
use crate::plugins::ThreadSpawner;
use excavator_backend::formats::pak::PakParser;

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
	pub fn display_name_lossy(&self) -> Option<Cow<'_, str>> {
		match self {
			Self::Fs { path, .. } => path.file_name().map(|s| s.to_string_lossy()),
			Self::Pak { inner_path, .. } => Some(inner_path.to_string_lossy()),
		}
	}
	
	pub fn filename(&self) -> &[u8] {
		match self {
			Self::Fs { path, .. } => {
				path.file_name().unwrap_or_default().as_encoded_bytes()
			},
			Self::Pak { inner_path, .. } => {
				// A simple, good-enough implementation
				inner_path.as_bytes().split(|b| *b == b'/' || *b == b'\\').last().unwrap_or_default()
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

enum LoadState<T> {
	Loading,
	Done(std::sync::Weak<T>),
}

pub type ListingLoadResult = Result<LoadedListing, Box<str>>;

pub enum LoadedListing {
	Dir(Box<[DirEntry]>),
	Pak(Box<[PakEntry]>),
}

#[derive(Clone)]
pub struct DirEntry {
	pub path: PathBuf,
	pub metadata: std::fs::Metadata,
}

#[derive(Clone)]
pub struct PakEntry {
	pub name: CString,
}

pub type BytesLoadResult = Result<Box<[u8]>, Box<str>>;

pub struct FileBytes {
	source_item: ItemInfo,
	inner: excavator_backend::io::FileBytes,
}

impl FileBytes {
	pub fn as_slice(&self) -> &[u8] {
		self.inner.as_ref()
	}
	
	pub fn source_item(&self) -> &ItemInfo {
		&self.source_item
	}
}

pub struct ItemLoader<T: Send + Sync + 'static> {
	load_items: Arc<Mutex<HashMap<PathBuf, LoadState<T>>>>,
}

// Default's derive macro requires the type parameter to implement Default.
// Since we want this to be Default regardless of the type parameter, we have to do it manually.
impl<T: Send + Sync + 'static> Default for ItemLoader<T> {
	fn default() -> Self {
		Self { load_items: Default::default() }
	}
}

impl<T: Send + Sync + 'static> ItemLoader<T> {
	// This function exists so the or_else closure can use the same lock, ensuring that no external modification can occur in the middle of this logic.
	fn get_or_else<F>(&self, item: &ItemInfo, f: F) -> Option<Arc<T>>
	where
		F: FnOnce(&mut HashMap<PathBuf, LoadState<T>>) -> Option<Arc<T>>,
	{
		let path = item.outer_path();
		let mut load_items = self.load_items.lock().unwrap();
		match load_items.get(path) {
			Some(LoadState::Done(weak)) => {
				let strong = weak.upgrade();
				if strong.is_none() { load_items.remove(path); }
				strong
			},
			_ => None,
		}.or_else(|| f(&mut load_items))
	}
	
	#[expect(dead_code)] // This'll probably be useful somewhere
	pub fn get(&self, item: &ItemInfo) -> Option<Arc<T>> {
		self.get_or_else(item, |_| { None })
	}
	
	pub fn get_or_request(&self, item: &ItemInfo, ctx: &egui::Context) -> Option<Arc<T>>
	where
		T: LoadableData,
	{
		self.get_or_else(item, |load_items| {
			let path = item.outer_path();
			if !load_items.contains_key(path) {
				load_items.insert(path.to_owned(), LoadState::Loading);
				self.start_load(item.clone(), ctx.clone());
			}
			None
		})
	}
	
	fn start_load(&self, item: ItemInfo, ctx: egui::Context)
	where
		T: LoadableData,
	{
		let load_items = Arc::clone(&self.load_items);
		let spawner = ctx.plugin_or_default::<ThreadSpawner>();
		spawner.lock().spawn(ctx, move |_| {
			let result = Arc::new(T::do_load(&item));
			load_items.lock().unwrap().insert(
				item.outer_path().to_owned(),
				LoadState::Done(Arc::downgrade(&result)),
			);
			Some(result.into_message(item))
		});
	}
}

pub trait LoadableData {
	fn do_load(item: &ItemInfo) -> Self;
	fn into_message(self: Arc<Self>, item: ItemInfo) -> ExcavatorMessage;
}

impl LoadableData for ListingLoadResult {
	fn do_load(item: &ItemInfo) -> Self {
		match item {
			ItemInfo::Fs { path, kind: FsItemKind::Directory } => {
				let contents = std::fs::read_dir(path)
					.map_err(|e| e.to_string())?
					.map(|entry| entry.and_then(|entry| Ok(DirEntry {
						path: entry.path(),
						metadata: entry.metadata()?,
					})))
					.collect::<Result<_, _>>()
					.map_err(|e| e.to_string())?;
				Ok(LoadedListing::Dir(contents))
			},
			ItemInfo::Fs { path, kind: FsItemKind::File } => {
				let extension = path.extension().map(|ex| ex.as_encoded_bytes());
				
				if extension == Some(b"pak") {
					let file = File::open(&path)
						.map_err(|e| e.to_string())?;
					let bufreader = BufReader::new(file);
					
					let mut parser = PakParser::new(bufreader).map_err(|e| e.to_string())?;
					Ok(LoadedListing::Pak(
						parser.files().map_err(|e| e.to_string())?
							.map(|r| r.map(|(_, name)| PakEntry { name: CString::new(name).unwrap() }))
							.collect::<Result<_, _>>().map_err(|e| e.to_string())?
					))
				} else {
					Err("extension not known archive type".into())
				}
			},
			_ => Err("unhandled kind of ItemInfo in ListingLoadResult::do_load".into()),
		}
	}
	
	fn into_message(self: Arc<Self>, item: ItemInfo) -> ExcavatorMessage {
		ExcavatorMessage::ListingLoadDone { item, result: self }
	}
}

impl LoadableData for BytesLoadResult {
	fn do_load(item: &ItemInfo) -> Self {
		let path = item.outer_path();
		let mut file = File::open(&path)
			.map_err(|e| e.to_string())?;
		
		let mut buf = Vec::new();
		file.read_to_end(&mut buf)
			.map_err(|e| e.to_string())?;
		
		Ok(buf.into())
	}
	
	fn into_message(self: Arc<Self>, item: ItemInfo) -> ExcavatorMessage {
		ExcavatorMessage::BytesLoadDone { path: item.outer_path().to_owned(), result: self }
	}
}

pub fn slice_item(load_result: Arc<BytesLoadResult>, item: &ItemInfo) -> Result<FileBytes, String> {
	match item {
		ItemInfo::Fs { .. } => {
			Ok(FileBytes {
				source_item: item.clone(),
				inner: excavator_backend::io::FileBytes::glue_new(load_result, ..),
			})
		},
		ItemInfo::Pak { inner_path, .. } => {
			// "so like if we've gotten this far we've already checked the result is ok" (regarding the unwrap)
			// wait did i remove that check?! it crashed right here when i tested an error case
			let cursor = std::io::Cursor::new(Result::as_ref(&load_result).unwrap());
			let mut parser = PakParser::new(cursor).map_err(|e| e.to_string())?;
			
			let mut files_iter = parser.files().map_err(|_| "error reading pak listing")?;
			// Iterate through the list of files to find the one with the filename we're looking for
			let index = loop {
				let Some(result) = files_iter.next() else {
					return Err("no such file in pak".into());
				};
				let Ok((i, name)) = result else {
					return Err("error reading pak listing".into())
				};
				if name == inner_path.as_bytes() {
					break i;
				}
			};
			drop(files_iter);
			
			let (file_pos, file_size) = parser.file_position_size(index).map_err(|_| "error reading pak entry")?;
			let range = (file_pos as usize)..((file_pos+file_size) as usize);
			
			Ok(FileBytes {
				source_item: item.clone(),
				inner: excavator_backend::io::FileBytes::glue_new(load_result, range),
			})
		},
	}
}
