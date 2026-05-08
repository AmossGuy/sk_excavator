use lexical_sort::natural_lexical_cmp;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct DirItem {
	file_path: PathBuf,
	file_type: std::fs::FileType,
	file_size: u64,
}

impl DirItem {
	pub fn read_single(path: &Path) -> std::io::Result<Self> {
		let metadata = std::fs::symlink_metadata(path)?;
		Ok(Self::from_path_and_metadata(path.to_path_buf(), metadata))
	}
	
	fn from_path_and_metadata(path: PathBuf, metadata: std::fs::Metadata) -> Self {
		Self {
			file_path: path,
			file_type: metadata.file_type(),
			file_size: metadata.len(),
		}
	}
	
	fn from_dir_entry(entry: &std::fs::DirEntry) -> std::io::Result<Self> {
		let metadata = entry.metadata()?;
		let path = entry.path();
		Ok(Self::from_path_and_metadata(path, metadata))
	}
	
	pub fn file_path(&self) -> &Path {
		&self.file_path
	}
	
	pub fn display_name(&self) -> Cow<'_, str> {
		self.file_path.file_name().unwrap_or_default().to_string_lossy()
	}
	
	pub fn is_dir(&self) -> bool {
		self.file_type.is_dir()
	}
	
	pub fn file_size(&self) -> u64 {
		self.file_size
	}
}

pub struct DirContents {
	listing: Vec<DirItem>,
}

impl DirContents {
	pub fn read(path: &Path) -> std::io::Result<Self> {
		let listing = std::fs::read_dir(path)?
			.map(|r| match r {
				Ok(entry) => DirItem::from_dir_entry(&entry),
				Err(e) => Err(e),
			})
			.collect::<Result<Vec<_>, _>>()?;
		
		Ok(Self { listing })
	}
	
	pub fn sort_by_name(&mut self) {
		self.listing.sort_unstable_by(|a, b| {
			natural_lexical_cmp(&a.display_name(), &b.display_name())
		});
	}
	
	pub fn sorted_by_name(mut self) -> Self {
		self.sort_by_name();
		self
	}
	
	pub fn iter(&self) -> impl Iterator<Item = &DirItem> {
		self.listing.iter()
	}
}
