use crate::formats::common::{ArcBytes, tree::{ItemId, TreeFormat, TreeItem, TreeItemType}};
use excavator_backend_macros::EditableData;

use derive_more::From;
use thunderdome::{Arena, Index as ArenaIndex};
use undoredo::Recorder;

pub struct Anb {
	pub(super) header: Recorder<[TreeItem<Header>; 1]>,
	pub(super) nodes: Recorder<Arena<TreeItem<Node>>>,
}

#[derive(Copy, Clone, From)]
pub enum AnyItemId {
	Header(HeaderId),
}

#[derive(Copy, Clone, From)]
pub enum AnyItemRef<'a> {
	Header(&'a TreeItem<Header>),
}

impl TreeFormat for Anb {
	type RootId = HeaderId;
	type AnyItemRef<'a> = AnyItemRef<'a>;
	
	fn root_id(&self) -> HeaderId {
		HeaderId
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
	pub fixup: u32,
	pub version: u32,
	pub padding_a: u32,
	pub padding_b: u32,
	pub padding_c: u32,
}

impl TreeItemType for Header {
	type Format = Anb;
	type ParentId = ();
	type ChildrenIdList = NodeId;
}

#[derive(Copy, Clone)]
pub struct HeaderId;

impl ItemId<Anb> for HeaderId {
	type Ref<'a> = &'a TreeItem<Header>;
	
	fn get_from<'a>(self, source: &'a Anb) -> Option<&'a TreeItem<Header>> {
		source.header.get(&0)
	}
}

#[derive(EditableData, Clone, Default)]
pub enum Node {
	#[default]
	Base,
	Texture(NodeTexture),
	Vertex(NodeVertex),
	Meta,
	MetaScalar(NodeMetaScalar),
	MetaPoint(NodeMetaPoint),
	MetaAnchor(NodeMetaAnchor),
	MetaRect(NodeMetaRect),
	MetaString(NodeMetaString),
	MetaTable(NodeMetaTable),
	Frame(NodeFrame),
	SequenceFrame(NodeSequenceFrame),
	Sequence(NodeSequence),
	Animation(NodeAnimation),
}

#[derive(Copy, Clone)]
pub struct NodeId(pub(super) ArenaIndex);

impl TreeItemType for Node {
	type Format = Anb;
	type ParentId = AnyItemId;
	type ChildrenIdList = Vec<NodeId>;
}

impl Node {
	// Isn't this the save module's business?
	pub fn kind(&self) -> u32 {
		match self {
			Self::Base => 0,
			Self::Texture(_) => 1,
			Self::Vertex(_) => 2,
			Self::Meta => 3,
			Self::MetaScalar(_) => 4,
			Self::MetaPoint(_) => 5,
			Self::MetaAnchor(_) => 6,
			Self::MetaRect(_) => 7,
			Self::MetaString(_) => 8,
			Self::MetaTable(_) => 9,
			Self::Frame(_) => 10,
			Self::SequenceFrame(_) => 11,
			Self::Sequence(_) => 12,
			Self::Animation(_) => 13,
		}
	}
}

#[derive(EditableData, Clone, Default)]
pub struct NodeTexture {
	pub width: u32,
	pub height: u32,
	pub flags: u32,
	pub padding: u32,
	#[edit(skip)]
	pub data_block: Option<DataBlock>,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeVertex {
	pub vert_count: u32,
	pub flags: u32,
	#[edit(skip)]
	pub data_block: Option<DataBlock>,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaScalar {
	pub unk_1: u32,
	pub unk_2: u32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaPoint {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub padding: u32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaAnchor {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub angle: f32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaRect {
	pub center_x: f32,
	pub center_y: f32,
	pub center_z: f32,
	pub extents_x: f32,
	pub extents_y: f32,
	pub extents_z: f32,
	pub angle: f32,
	pub padding: u32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaString {
	pub string_length: u32,
	pub padding: u32,
	#[edit(skip)]
	pub data_block: Option<DataBlock>,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeMetaTable {
	#[edit(skip)]
	pub data_block: Option<DataBlock>,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeFrame {
	pub min_x: f32,
	pub max_x: f32,
	pub min_y: f32,
	pub max_y: f32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeSequenceFrame {
	pub frame: u32,
	pub delay: f32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeSequence {
	pub hashname: u32,
	pub frame_count: u32,
}

#[derive(EditableData, Clone, Default)]
pub struct NodeAnimation {
	pub sequence_count: u32,
	pub frame_count: u32,
	pub single_texture: u32,
	pub palette_index: u32,
	#[edit(skip)]
	pub data_block: Option<DataBlock>,
}

#[derive(EditableData, Clone, Default)]
pub struct VertexBodyEntry {
	pub position_x: f32,
	pub position_y: f32,
	pub texture_x: u16,
	pub texture_y: u16,
	pub width: u16,
	pub height: u16,
}

#[derive(Clone)]
pub struct DataBlock {
	pub flags: u32,
	pub data: ArcBytes,
}
