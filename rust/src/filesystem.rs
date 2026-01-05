use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Seek};
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FsItemType {
	Dir,
	File,
	Symlink,
	Other,
}

#[derive(Clone)]
pub struct FsItem {
	pub path: PathBuf,
	pub item_type: FsItemType,
}

impl From<fs::FileType> for FsItemType {
	fn from(value: fs::FileType) -> Self {
		if value.is_dir() {
			Self::Dir
		} else if value.is_file() {
			Self::File
		} else if value.is_symlink() {
			Self::Symlink
		} else {
			Self::Other
		}
	}
}

pub fn load_directory(path: impl AsRef<Path>) -> io::Result<Vec<FsItem>> {
	fs::read_dir(&path)?
		.map(|res| res.and_then(|entry| {
			let item_path = path.as_ref().join(entry.path());
			let item_type = FsItemType::from(entry.file_type()?);
			Ok(FsItem { path: item_path, item_type })
		}))
		.collect::<Result<Vec<_>, _>>()
}

pub fn open_file(path: impl AsRef<Path>) -> io::Result<impl BufRead + Seek> {
	File::open(path).map(|f| BufReader::new(f))
}
