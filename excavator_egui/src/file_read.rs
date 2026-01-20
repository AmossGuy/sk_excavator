use std::borrow::Cow;
use std::ffi::CString;
use std::path::{Path, PathBuf};

// Represents the location of a file or directory, including ones inside a .pak archive.
// If inner_path is None, outer_path is the path to the item itself.
// If inner_path is Some, outer_path is the path to the containing archive, and inner_path is the path within the archive.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FileLocation {
	outer_path: PathBuf,
	inner_path: Option<CString>,
}

impl FileLocation {
	pub fn new(outer_path: PathBuf, inner_path: Option<CString>) -> Self {
		Self { outer_path, inner_path }
	}
	
	pub fn file_name(&self) -> Option<Cow<'_, str>> {
		if let Some(inner_path) = &self.inner_path {
			Some(inner_path.to_string_lossy())
		} else {
			self.outer_path.file_name().map(|s| s.to_string_lossy())
		}
	}
	
	// Returning a slice of u8 is for the sake of having a shared type in both cases.
	// It's fine since we only need to handle ASCII extensions.
	pub fn extension(&self) -> Option<&[u8]> {
		if let Some(_inner_path) = &self.inner_path {
			todo!()
		} else {
			self.outer_path.extension().map(|e| e.as_encoded_bytes())
		}
	}
}

// I'm writing this program so that the Path and PathBuf types are used only for the real filesystem.
// The idea here is that, conceptually, FileLocation represents a superset of the values a Path can have.
impl From<PathBuf> for FileLocation {
	fn from(value: PathBuf) -> Self {
		Self { outer_path: value, inner_path: None }
	}
}

impl<'a> From<&'a Path> for FileLocation {
	fn from(value: &'a Path) -> Self {
		Self::from(value.to_path_buf())
	}
}

#[derive(Clone)]
pub enum TryFromFileLocationError {
	NotFilesystemItem,
}

impl TryFrom<FileLocation> for PathBuf {
	type Error = TryFromFileLocationError;
	
	fn try_from(value: FileLocation) -> Result<PathBuf, Self::Error> {
		if value.inner_path.is_none() {
			Ok(value.outer_path)
		} else {
			Err(TryFromFileLocationError::NotFilesystemItem)
		}
	}
}

impl<'a: 'b, 'b> TryFrom<&'a FileLocation> for &'b Path {
	type Error = TryFromFileLocationError;
	
	fn try_from(value: &'a FileLocation) -> Result<&'b Path, Self::Error> {
		if value.inner_path.is_none() {
			Ok(&value.outer_path)
		} else {
			Err(TryFromFileLocationError::NotFilesystemItem)
		}
	}
}

impl PartialEq<PathBuf> for FileLocation {
	fn eq(&self, other: &PathBuf) -> bool {
		self.inner_path.is_none() && self.outer_path == *other
	}
}

impl<'a> PartialEq<&'a Path> for FileLocation {
	fn eq(&self, other: &&'a Path) -> bool {
		self.inner_path.is_none() && self.outer_path == *other
	}
}

impl PartialEq<Path> for FileLocation {
	fn eq(&self, other: &Path) -> bool {
		self.inner_path.is_none() && self.outer_path == other
	}
}


