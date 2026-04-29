/*
use bstr::BString;
use std::path::PathBuf;

pub struct ItemPath {
	inner: ItemPathEnum,
}

#[derive(Clone)]
enum ItemPathEnum {
	Fs { path: PathBuf },
	Pak { name: BString, parent: Box<ItemPathEnum> },
}

impl ItemPath {
	pub fn fs(path: PathBuf) -> Self {
		Self { inner: ItemPathEnum::Fs { path } }
	}
}
*/
