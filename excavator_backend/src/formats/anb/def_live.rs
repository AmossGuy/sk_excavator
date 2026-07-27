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
	// Base,
	// Texture(NodeTexture),
	// Vertex(NodeVertex),
	// Meta,
	// MetaScalar(NodeMetaScalar),
	MetaPoint(NodeMetaPoint),
	MetaAnchor(NodeMetaAnchor),
	MetaRect(NodeMetaRect),
	// MetaString(NodeMetaString),
	// MetaTable(NodeMetaTable),
	// Frame(NodeFrame),
	// SequenceFrame(NodeSequenceFrame),
	// Sequence(NodeSequence),
	// Animation(NodeAnimation),
	#[edit(skip)]
	UnknownKind(u32),
}

impl Node {
	pub fn kind(&self) -> u32 {
		match self {
			Self::MetaPoint(_) => 5,
			Self::MetaAnchor(_) => 6,
			Self::MetaRect(_) => 7,
			Self::UnknownKind(kind) => *kind,
		}
	}
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
