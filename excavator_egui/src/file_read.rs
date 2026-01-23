use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ItemInfo {
	Fs { path: PathBuf, kind: FsItemKind },
	Pak { outer_path: PathBuf, inner_path: CString },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

// This is a struct so I can try adding caching or somesuch to it later.
#[derive(Default)]
pub struct FileLoader {
	file_binds: HashMap<ItemInfo, egui_async::Bind<Arc<[u8]>, std::io::Error>>,
}

impl FileLoader {
	pub fn read_or_request(&mut self, file_info: &ItemInfo) -> Option<&std::io::Result<Arc<[u8]>>> {
		// Passing false for the Bind's retain parameter as a rudimentary way of clearing old files.
		// Later I would like to do something more advanced, so that switching between a few files doesn't discard them every time.
		let bind = self.file_binds.entry(file_info.clone()).or_insert_with(|| egui_async::Bind::new(false));
		bind.read_or_request(|| async {
			let data = Arc::<[u8]>::from(*b"test data that isn't actually loaded from the file");
			Ok(data)
		})
	}
}
