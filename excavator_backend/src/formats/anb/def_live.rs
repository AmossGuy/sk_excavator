use excavator_backend_macros::EditableData;

#[derive(EditableData)]
#[expect(non_snake_case)]
pub struct Header {
	pub unknown_04: u32,
	pub unknown_08: u32,
	pub unknown_0C: u32,
	pub unknown_10: u32,
	pub unknown_14: u32,
}

#[derive(EditableData)]
pub enum Node {
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
	#[edit(skip)]
	UnknownKind(u32),
}

impl Node {
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
			Self::UnknownKind(kind) => *kind,
		}
	}
}

#[derive(EditableData, Default)]
pub struct NodeTexture {
	pub width: u32,
	pub height: u32,
	pub flags: u32,
	pub padding: u32,
}

#[derive(EditableData, Default)]
pub struct NodeVertex {
	pub vert_count: u32,
	pub flags: u32,
	pub extra_data: Vec<u8>,
}

#[derive(EditableData, Default)]
pub struct NodeMetaScalar {
	pub unk_1: u32,
	pub unk_2: u32,
}

#[derive(EditableData, Default)]
pub struct NodeMetaPoint {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub padding: u32,
}

#[derive(EditableData, Default)]
pub struct NodeMetaAnchor {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub angle: f32,
}

#[derive(EditableData, Default)]
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

#[derive(EditableData, Default)]
pub struct NodeMetaString {
	pub string_length: u32,
	pub padding: u32,
}

#[derive(EditableData, Default)]
pub struct NodeMetaTable {
}

#[derive(EditableData, Default)]
pub struct NodeFrame {
	pub min_x: f32,
	pub max_x: f32,
	pub min_y: f32,
	pub max_y: f32,
}

#[derive(EditableData, Default)]
pub struct NodeSequenceFrame {
	pub frame: u32,
	pub delay: f32,
}

#[derive(EditableData, Default)]
pub struct NodeSequence {
	pub hashname: u32,
	pub frame_count: u32,
}

#[derive(EditableData, Default)]
pub struct NodeAnimation {
	pub sequence_count: u32,
	pub frame_count: u32,
	pub single_texture: u32,
	pub palette_index: u32,
}
