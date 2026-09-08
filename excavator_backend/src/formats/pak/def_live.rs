use crate::formats::common::ArcBytes;
use excavator_backend_macros::EditableData;

use thunderdome::{Arena, Index as ArenaIndex};
use undoredo::{Recorder, maplike::one::One};

pub struct Pak {
	pub(super) header: Recorder<One<Header>>,
	pub(super) files: Recorder<Arena<File>>,
}

impl Pak {
	pub fn get_header(&self) -> &Header {
		self.header.get(&0).expect("index is always in bounds")
	}
	
	pub fn get_file(&self, id: FileId) -> Option<&File> {
		self.files.get(&id.0)
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
	#[edit(skip)]
	pub files: Vec<FileId>,
}

#[derive(EditableData, Clone)]
pub struct File {
	#[edit(skip)]
	pub filename: ArcBytes,
	pub time: u64,
	pub filename_hash: u32,
	pub flags: u32,
	pub specials: u32,
	pub padding: u32,
	#[edit(skip)]
	pub data: ArcBytes,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileId(pub(super) ArenaIndex);
