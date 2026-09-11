use super::{AnbWithoutUndo, Header, NodeData, Node, NodeId};

use std::{borrow::Cow, collections::HashMap};
use undo_2::Action;

pub enum AnbCommand {
	EditHeaderProps { old: Header, new: Header },
	EditNodeProps { id: NodeId, old: NodeData, new: NodeData },
	Reparent(ReparentCommand),
}

impl AnbCommand {
	pub fn description(&self) -> Cow<'static, str> {
		match self {
			Self::EditHeaderProps { .. } => "Edit header properties".into(),
			Self::EditNodeProps { .. } => "Edit node properties".into(),
			Self::Reparent { .. } => "Move node(s)".into(),
		}
	}
}

impl AnbWithoutUndo {
	pub(super) fn interpret_action(&mut self, action: (Action, &AnbCommand)) {
		use {Action::*, AnbCommand::*};
		
		match action {
			(Do, EditHeaderProps { old: _, new }) => {
				self.header = new.clone();
			},
			(Undo, EditHeaderProps { old, new: _ }) => {
				self.header = old.clone();
			},
			(Do, EditNodeProps { id, old: _, new }) => {
				self.node_arena[id.0].data = new.clone();
			},
			(Undo, EditNodeProps { id, old, new: _ }) => {
				self.node_arena[id.0].data = old.clone();
			},
			(Do, Reparent(reparent)) => {
				reparent.apply(self);
			},
			(Undo, Reparent(reparent)) => {
				reparent.undo(self);
			},
		}
	}
}

pub struct ReparentCommand {
	affected: HashMap<NodeId, ReparentEntry>,
}

struct ReparentEntry {
	old: Node,
	new: Node,
}

impl ReparentCommand {
	fn blank() -> Self {
		Self { affected: HashMap::new() }
	}
	
	fn build_modify_node(&mut self, anb: &AnbWithoutUndo, id: NodeId) -> &mut Node {
		let entry = self.affected.entry(id).or_insert_with(|| {
			let node = anb.get_node(id).unwrap();
			ReparentEntry { old: node.clone(), new: node.clone() }
		});
		&mut entry.new
	}
	
	pub fn build(
		anb: &AnbWithoutUndo,
		parent_id: NodeId, children_ids: &[NodeId], insert_index: usize,
	) -> Self {
		let mut this = Self::blank();
		
		for child_id in children_ids.iter().copied() {
			// Change the child's parent
			let child = this.build_modify_node(anb, child_id);
			let old_parent_id = child.parent.unwrap();
			child.parent = Some(parent_id);
			
			// Remove from the old parent's children list
			let old_parent = this.build_modify_node(anb, old_parent_id);
			old_parent.children.retain(|&x| x != child_id);
		}
		
		// Add to the new parent's children list
		// Important that this happens last, in case the old and new parent are the same!
		let new_parent = this.build_modify_node(anb, parent_id);
		new_parent.children.splice(insert_index..insert_index, children_ids.iter().copied());
		
		this
	}
	
	pub fn apply(&self, anb: &mut AnbWithoutUndo) {
		for (node_id, entry) in &self.affected {
			*anb.node_arena.get_mut(node_id.0).unwrap() = entry.new.clone();
		}
	}
	
	pub fn undo(&self, anb: &mut AnbWithoutUndo) {
		for (node_id, entry) in &self.affected {
			*anb.node_arena.get_mut(node_id.0).unwrap() = entry.old.clone();
		}
	}
}
