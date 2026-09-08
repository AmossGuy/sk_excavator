mod load;
mod raw;

pub use load::load_from_bytes;

use crate::formats::common::ArcBytes;
use excavator_backend_macros::EditableData;

use thunderdome::{Arena, Index as ArenaIndex};

pub struct Pak {
	header: Header,
	file_arena: Arena<File>,
}

impl Pak {
	pub fn get_header(&self) -> &Header {
		&self.header
	}
	
	pub fn get_file(&self, id: FileId) -> Option<&File> {
		self.file_arena.get(id.0)
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
pub struct FileId(ArenaIndex);
