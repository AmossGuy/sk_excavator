use crate::formats::common::ArcBytes;
use excavator_backend_macros::EditableData;
use bevy_ecs::component::Component;

#[derive(EditableData, Component, Clone)]
pub struct Header {
}

#[derive(Component, Clone)]
pub struct FileName {
	pub name: ArcBytes,
}

#[derive(EditableData, Component, Clone)]
pub struct FileMetadata {
	pub time: u64,
	pub filename_hash: u32,
	pub flags: u32,
	pub specials: u32,
	pub padding: u32,
}

#[derive(Component, Clone)]
pub struct FileData {
	pub data: ArcBytes,
}
