use crate::formats::common::{ArcBytes, TreeFormat};
use excavator_backend_macros::EditableData;

use undoredo::Recorder;

pub struct Pak {
	pub header: Recorder<[Header; 1]>,
}

impl TreeFormat for Pak {}

#[derive(EditableData, Clone)]
pub struct Header {
}

#[derive(Clone)]
pub struct FileName {
	pub name: ArcBytes,
}

#[derive(EditableData, Clone)]
pub struct FileMetadata {
	pub time: u64,
	pub filename_hash: u32,
	pub flags: u32,
	pub specials: u32,
	pub padding: u32,
}

#[derive(Clone)]
pub struct FileData {
	pub data: ArcBytes,
}
