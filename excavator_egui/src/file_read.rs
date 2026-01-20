use std::borrow::Cow;
use std::ffi::CString;
use std::path::PathBuf;

// Represents the location of a file or directory, including ones inside a .pak archive.
// If inner_path is None, outer_path is the path to the item itself.
// If inner_path is Some, outer_path is the path to the containing archive, and inner_path is the path within the archive.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FileLocation {
	Fs { path: PathBuf },
	Pak { outer_path: PathBuf, inner_path: CString },
}

impl FileLocation {
	pub fn file_name_lossy(&self) -> Option<Cow<'_, str>> {
		match self {
			Self::Fs { path } => path.file_name().map(|s| s.to_string_lossy()),
			Self::Pak { inner_path, .. } => Some(inner_path.to_string_lossy()),
		}
	}
	
	// Returning a slice of u8 is for the sake of having a shared type in both cases.
	// It's fine since we only need to handle ASCII extensions.
	pub fn extension(&self) -> Option<&[u8]> {
		match self {
			Self::Fs { path } => path.extension().map(|e| e.as_encoded_bytes()),
			Self::Pak { .. } => todo!(),
		}
	}
}
