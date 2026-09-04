use crate::formats::common::{ArcBytes, tree::{TreeFormat, TreeItem, TreeItemType}};
use excavator_backend_macros::EditableData;

use thunderdome::{Arena, Index as ArenaIndex};
use undoredo::Recorder;

pub struct Pak {
	pub(super) header: Recorder<[TreeItem<Header>; 1]>,
	pub(super) files: Recorder<Arena<TreeItem<File>>>,
}

#[derive(Copy, Clone)]
pub enum ItemId {
	Header(HeaderId),
	File(FileId),
}

pub enum ItemRef<'a> {
	Header(&'a TreeItem<Header>),
	File(&'a TreeItem<File>),
}

impl TreeFormat for Pak {
	type ItemId = ItemId;
	type ItemRef<'a> = ItemRef<'a>;
	
	fn root_id(&self) -> ItemId {
		ItemId::Header(HeaderId)
	}
	
	fn get_ref(&self, id: ItemId) -> Option<ItemRef<'_>> {
		Some(match id {
			ItemId::Header(HeaderId) => ItemRef::Header(self.header.get(&0)?),
			ItemId::File(FileId(index)) => ItemRef::File(self.files.get(&index)?),
		})
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
}

#[derive(Copy, Clone)]
pub struct HeaderId;

impl TreeItemType for Header {
	type ParentId = ();
	type ChildrenIdList = Vec<FileId>;
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

#[derive(Copy, Clone)]
pub struct FileId(pub(super) ArenaIndex);

impl TreeItemType for File {
	type ParentId = HeaderId;
	type ChildrenIdList = ();
}
