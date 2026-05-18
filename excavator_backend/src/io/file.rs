use bstr::BString;
use std::fs::File;
use std::io::{Read, Take};
use std::path::PathBuf;

use crate::formats::FileFormat;

#[derive(Clone)]
pub enum FileSource {
	Fs { path: PathBuf },
	Pak { outer_path: PathBuf, inner_path: BString },
}

impl FileSource {
	pub fn open(&self) -> anyhow::Result<Take<File>> {
		match self {
			Self::Fs { path } => {
				let take = File::open(&path)?.take(u64::MAX);
				Ok(take)
			},
			Self::Pak { outer_path, inner_path } => {
				crate::formats::pak::open_pak_entry(outer_path.clone(), inner_path.clone())
			},
		}
	}
	
	pub fn file_format(&self) -> Option<FileFormat> {
		match self {
			Self::Fs { path } => FileFormat::from_filename(path.as_os_str().as_encoded_bytes()),
			Self::Pak { inner_path, .. } => FileFormat::from_filename(inner_path),
		}
	}
}
