use std::fs::{self, File};
use std::error::Error;
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::path::{Path, PathBuf};

use crate::godot::browser_tree::ItemSource;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

pub fn cruddy_complex_load(source: &ItemSource) -> Result<Vec<u8>, Box<dyn Error>> {
	match source {
		ItemSource::Fs { path, .. } => {
			let mut reader = crate::filesystem::open_file(path)?;
			let mut buf = Vec::new();
			reader.read_to_end(&mut buf)?;
			Ok(buf)
		},
		ItemSource::Pak { outer_path, inner_path } => {
			let mut reader = crate::filesystem::open_file(outer_path)?;
			let index = crate::formats::pak::PakIndex::create_index(&mut reader)?;
			let entry = index.files.iter()
				.find(|f| &f.0 == inner_path)
				.ok_or(io::Error::from(io::ErrorKind::NotFound))?;
			let data = crate::formats::pak::read_whole_file(&entry.1, &mut reader)?;
			Ok(data)
		},
	}
}
