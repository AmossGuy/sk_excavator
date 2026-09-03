use crate::formats::common::{ArcBytes, TreeFormat};
use excavator_backend_macros::EditableData;

use thunderdome::Arena;
use undoredo::Recorder;

pub struct Pak {
	pub header: Recorder<[Header; 1]>,
	pub files: Recorder<Arena<File>>,
}

#[derive(Copy, Clone)]
pub enum ItemId {
	Header,
}

impl TreeFormat for Pak {
	type ItemId = ItemId;
	
	fn root_id(&self) -> ItemId {
		ItemId::Header
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
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
