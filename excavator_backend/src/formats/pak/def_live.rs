use crate::formats::common::{ArcBytes, tree::{ItemId, TreeFormat, TreeItem, TreeItemType}};
use excavator_backend_macros::EditableData;

use derive_more::From;
use thunderdome::{Arena, Index as ArenaIndex};
use undoredo::Recorder;

pub struct Pak {
	pub(super) header: Recorder<[TreeItem<Header>; 1]>,
	pub(super) files: Recorder<Arena<TreeItem<File>>>,
}

#[derive(Copy, Clone, From)]
pub enum AnyItemId {
	Header(HeaderId),
	File(FileId),
}

#[derive(Copy, Clone, From)]
pub enum AnyItemRef<'a> {
	Header(&'a TreeItem<Header>),
	File(&'a TreeItem<File>),
}

impl TreeFormat for Pak {
	type RootId = HeaderId;
	type AnyItemRef<'a> = AnyItemRef<'a>;
	
	fn root_id(&self) -> HeaderId {
		HeaderId
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
}

impl TreeItemType for Header {
	type Format = Pak;
	type ParentId = ();
	type ChildrenIdList = Vec<FileId>;
}

#[derive(Copy, Clone)]
pub struct HeaderId;

impl ItemId<Pak> for HeaderId {
	type Ref<'a> = &'a TreeItem<Header>;
	
	fn get_from<'a>(self, source: &'a Pak) -> Option<&'a TreeItem<Header>> {
		source.header.get(&0)
	}
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
	type Format = Pak;
	type ParentId = HeaderId;
	type ChildrenIdList = ();
}
