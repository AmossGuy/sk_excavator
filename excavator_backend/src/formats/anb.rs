mod load;
mod raw;
mod save;
mod undo;

pub use load::load_from_bytes;
// pub use save::save_from_world;

use crate::formats::common::ArcBytes;
use excavator_backend_macros::EditableData;
use self::undo::{AnbCommand, ReparentCommand};

use std::mem;
use thunderdome::{Arena, Index as ArenaIndex};

pub struct Anb {
	inner: AnbWithoutUndo,
	undo: undo_2::Commands<AnbCommand>,
}

struct AnbWithoutUndo {
	header: Header,
	node_arena: Arena<Node>,
}

impl Anb {
	fn from_parts(header: Header, node_arena: Arena<Node>) -> Self {
		Self {
			inner: AnbWithoutUndo { header, node_arena },
			undo: undo_2::Commands::new(),
		}
	}
	
	pub fn get_header(&self) -> &Header {
		&self.inner.get_header()
	}
	
	pub fn get_node(&self, id: NodeId) -> Option<&Node> {
		self.inner.get_node(id)
	}
	
	pub fn can_undo(&self) -> bool {
		self.undo.current_command_index() != None
	}
	
	pub fn can_redo(&self) -> bool {
		match self.undo.len() {
			0 => false,
			len => self.undo.current_command_index() != Some(len - 1),
		}
	}
	
	pub fn undo(&mut self) {
		for action in self.undo.undo() {
			self.inner.interpret_action(action);
		}
	}
	
	pub fn redo(&mut self) {
		for action in self.undo.redo() {
			self.inner.interpret_action(action);
		}
	}
	
	pub fn edit_header_props(&mut self, new: Header) {
		let old = mem::replace(&mut self.inner.header, new.clone());
		self.undo.push(AnbCommand::EditHeaderProps { old, new });
	}
	
	pub fn edit_node_props(&mut self, id: NodeId, new: NodeData) {
		let old = mem::replace(&mut self.inner.node_arena[id.0].data, new.clone());
		self.undo.push(AnbCommand::EditNodeProps { id, old, new });
	}
	
	pub fn edit_reparent(
		&mut self,
		parent_id: NodeId, children_ids: &[NodeId], insert_index: usize,
	) {
		let command = ReparentCommand::build(&self.inner, parent_id, children_ids, insert_index);
		command.apply(&mut self.inner);
		self.undo.push(AnbCommand::Reparent(command));
	}
}

impl AnbWithoutUndo {
	pub fn get_header(&self) -> &Header {
		&self.header
	}
	
	pub fn get_node(&self, id: NodeId) -> Option<&Node> {
		self.node_arena.get(id.0)
	}
}

#[derive(EditableData, Clone)]
pub struct Header {
	#[edit(skip)]
	pub root_node: Option<NodeId>,
	
	pub fixup: u32,
	pub version: u32,
	pub padding_a: u32,
	pub padding_b: u32,
	pub padding_c: u32,
}

#[derive(Clone, Default)]
pub struct Node {
	pub parent: Option<NodeId>,
	pub children: Vec<NodeId>,
	pub data: NodeData,
}

#[derive(EditableData, Clone, Default)]
pub enum NodeData {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(ArenaIndex);

impl NodeData {
	// Isn't this the save module's business?
	pub fn kind(&self) -> u32 {
		match self {
			Self::Base => raw::NODE_KIND_BASE,
			Self::Texture(_) => raw::NODE_KIND_TEXTURE,
			Self::Vertex(_) => raw::NODE_KIND_VERTEX,
			Self::Meta => raw::NODE_KIND_META,
			Self::MetaScalar(_) => raw::NODE_KIND_META_SCALAR,
			Self::MetaPoint(_) => raw::NODE_KIND_META_POINT,
			Self::MetaAnchor(_) => raw::NODE_KIND_META_ANCHOR,
			Self::MetaRect(_) => raw::NODE_KIND_META_RECT,
			Self::MetaString(_) => raw::NODE_KIND_META_STRING,
			Self::MetaTable(_) => raw::NODE_KIND_META_TABLE,
			Self::Frame(_) => raw::NODE_KIND_FRAME,
			Self::SequenceFrame(_) => raw::NODE_KIND_SEQUENCE_FRAME,
			Self::Sequence(_) => raw::NODE_KIND_SEQUENCE,
			Self::Animation(_) => raw::NODE_KIND_ANIMATION,
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
