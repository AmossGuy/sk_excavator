use zerocopy::byteorder::*;
use zerocopy_derive::*;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct Header {
	pub magic: [u8; 4],
	pub fixup: U32<LE>,
	pub version: U32<LE>,
	pub padding_a: U32<LE>,
	pub padding_b: U32<LE>,
	pub padding_c: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeCommon {
	pub kind: U32<LE>,
	pub child_count: U32<LE>,
	pub child_array_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeTexture {
	pub width: U32<LE>,
	pub height: U32<LE>,
	pub flags: U32<LE>,
	pub padding: U32<LE>,
	pub data_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeVertex {
	pub vert_count: U32<LE>,
	pub flags: U32<LE>,
	pub data_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaScalar {
	pub unk_1: U32<LE>,
	pub unk_2: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaPoint {
	pub x: F32<LE>,
	pub y: F32<LE>,
	pub z: F32<LE>,
	pub padding: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaAnchor {
	pub x: F32<LE>,
	pub y: F32<LE>,
	pub z: F32<LE>,
	pub angle: F32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaRect {
	pub center_x: F32<LE>,
	pub center_y: F32<LE>,
	pub center_z: F32<LE>,
	pub extents_x: F32<LE>,
	pub extents_y: F32<LE>,
	pub extents_z: F32<LE>,
	pub angle: F32<LE>,
	pub padding: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaString {
	pub string_length: U32<LE>,
	pub padding: U32<LE>,
	pub string_offset: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeMetaTable {
	pub hashname_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeFrame {
	pub min_x: F32<LE>,
	pub max_x: F32<LE>,
	pub min_y: F32<LE>,
	pub max_y: F32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeSequenceFrame {
	pub frame: U32<LE>,
	pub delay: F32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeSequence {
	pub hashname: U32<LE>,
	pub frame_count: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct NodeAnimation {
	pub sequence_count: U32<LE>,
	pub frame_count: U32<LE>,
	pub single_texture: U32<LE>,
	pub palette_index: U32<LE>,
	pub hashname_pointer: U64<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct DataBlockHeader {
	pub flags: U32<LE>,
	pub data_size: U32<LE>,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct VertexBodyEntry {
	pub position_x: F32<LE>,
	pub position_y: F32<LE>,
	pub texture_x: U16<LE>,
	pub texture_y: U16<LE>,
	pub width: U16<LE>,
	pub height: U16<LE>,
}
