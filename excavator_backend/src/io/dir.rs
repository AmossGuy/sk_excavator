use lexical_sort::natural_lexical_cmp;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use futures_lite::stream::StreamExt;

#[derive(Clone)]
pub struct DirItem {
	source_path: PathBuf,
	file_type: std::fs::FileType,
	file_size: u64,
}

impl DirItem {
	pub async fn read_single_async(path: PathBuf) -> std::io::Result<Self> {
		let metadata = async_fs::symlink_metadata(&path).await?;
		Ok(Self::from_path_and_metadata(path, metadata))
	}
	
	fn from_path_and_metadata(path: PathBuf, metadata: std::fs::Metadata) -> Self {
		Self {
			source_path: path,
			file_type: metadata.file_type(),
			file_size: metadata.len(),
		}
	}
	
	async fn from_dir_entry_async(entry: &async_fs::DirEntry) -> std::io::Result<Self> {
		let metadata = entry.metadata().await?;
		Ok(Self::from_path_and_metadata(entry.path(), metadata))
	}
	
	pub fn source_path(&self) -> &Path {
		&self.source_path
	}
	
	pub fn display_name(&self) -> Cow<'_, str> {
		self.source_path.file_name().unwrap_or_default().to_string_lossy()
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
	pub async fn read_async(path: impl AsRef<Path>) -> std::io::Result<Self> {
		Self::read_async_inner(path.as_ref()).await
	}
		
	async fn read_async_inner(path: &Path) -> std::io::Result<Self> {
		let listing = async_fs::read_dir(path).await?
			.then(|r| async { match r {
				Ok(entry) => DirItem::from_dir_entry_async(&entry).await,
				Err(e) => Err(e),
			} })
			.try_collect().await?;
		
		Ok(Self { listing })
	}
	
	pub fn sort_by_name(&mut self) {
		self.listing.sort_unstable_by(|a, b| {
			natural_lexical_cmp(&a.display_name(), &b.display_name())
		});
	}
	
	pub fn iter(&self) -> impl Iterator<Item = &DirItem> {
		self.listing.iter()
	}
}
