use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CString;
use std::io::Cursor;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use excavator_formats::pak::PakIndex;

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

// This is a struct so I can try adding caching or somesuch to it later.
#[derive(Default)]
pub struct FileLoader {
	file_binds: HashMap<PathBuf, egui_async::Bind<Vec<u8>, std::io::Error>>,
}

impl FileLoader {
	fn read_or_request_fs(&mut self, path: PathBuf) -> Option<&Result<Vec<u8>, std::io::Error>> {
		// Passing false for the Bind's retain parameter as a rudimentary way of clearing old files.
		// Later I would like to do something more advanced, so that switching between a few files doesn't discard them every time.
		let bind = self.file_binds.entry(path.clone()).or_insert_with(|| egui_async::Bind::new(false));
		bind.read_or_request(move || {
			tokio::fs::read(path)
		})
	}
	
	fn slice_file_from_pak<'a>(data: &'a Vec<u8>, inner_path: &CString) -> Result<&'a [u8], anyhow::Error> {
		let index = PakIndex::create_index(&mut Cursor::new(data))?;
		let file = index.files.iter().find(|x| *x.0 == *inner_path).ok_or_else(|| {
			anyhow::anyhow!("File not found inside archive")
		})?;
		let (start, length) = (file.1.data_start as usize, file.1.data_length as usize);
		let slice = data.get(start..start+length).ok_or_else(|| {
			anyhow::anyhow!("Archive file points outside of archive")
		})?;
		Ok(slice)
	}
	
	pub fn read_or_request(&mut self, file_info: &ItemInfo) -> Option<Result<&[u8], anyhow::Error>> {
		match self.read_or_request_fs(file_info.outer_path().clone()) {
			Some(Ok(data)) => match file_info {
				ItemInfo::Fs { .. } => Some(Ok(data.as_slice())),
				ItemInfo::Pak { inner_path, .. } => Some(Self::slice_file_from_pak(&data, &inner_path)),
			},
			Some(Err(error)) => Some(Err(anyhow::anyhow!("{}", error))), // std::io::Error isn't Clone. stupid workaround
			None => None,
		}
	}
}
