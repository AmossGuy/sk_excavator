use std::borrow::Cow;
use std::ffi::CString;
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ItemInfo {
	Fs { path: PathBuf, kind: FsItemKind },
	Pak { outer_path: PathBuf, inner_path: CString },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
			Self::Pak { .. } => todo!(),
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
