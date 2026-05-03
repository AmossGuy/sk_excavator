use lexical_sort::{StringSort, natural_lexical_cmp};
use std::path::Path;

use futures_lite::stream::StreamExt;

pub struct DirContents {
	listing: Vec<String>,
}

impl DirContents {
	pub async fn read_async(path: impl AsRef<Path>) -> std::io::Result<Self> {
		Self::read_async_inner(path.as_ref()).await
	}
		
	async fn read_async_inner(path: &Path) -> std::io::Result<Self> {
		let listing = async_fs::read_dir(path).await?
			.map(|r| r.map(|entry| entry.file_name().to_string_lossy().into_owned()))
			.try_collect().await?;
		
		Ok(Self { listing })
	}
	
	pub fn sort_by_name(&mut self) {
		self.listing.string_sort_unstable(natural_lexical_cmp);
	}
	
	pub fn name_iter(&self) -> impl Iterator<Item = &str> {
		self.listing.iter().map(String::as_str)
	}
}
