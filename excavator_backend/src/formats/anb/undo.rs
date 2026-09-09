use super::{AnbWithoutUndo, Header, NodeData, NodeId};
use undo_2::Action;

pub enum AnbCommand {
	EditHeaderProps { old: Header, new: Header },
	EditNodeProps { id: NodeId, old: NodeData, new: NodeData },
	Reparent(ReparentCommand),
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
	affected: Vec<ReparentEntry>,
}

impl ReparentCommand {
	pub fn build(
		anb: &AnbWithoutUndo,
		parent_id: NodeId, children_ids: &[NodeId], insert_index: usize,
	) -> Self {
		use ReparentEntry::*;
		let mut affected = Vec::new();
		
		let parent = anb.get_node(parent_id).unwrap();
		let mut new_parent_children = parent.children.clone();
		new_parent_children.splice(insert_index..insert_index, children_ids.iter().copied());
		
		// Add to the new parent's children list
		affected.push(ChangeChildren {
			edited_parent: parent_id,
			new_children: new_parent_children,
			old_children: parent.children.clone(),
		});
		
		for child_id in children_ids.iter().copied() {
			// TODO: This is broken when the new and old parents are the same
			
			let child = anb.get_node(child_id).unwrap();
			let old_parent_id = child.parent.unwrap();
			let old_parent = anb.get_node(old_parent_id).unwrap();
			
			let mut new_old_parent_children = old_parent.children.clone();
			new_old_parent_children.retain(|&x| x != child_id);
			
			// Remove from the old parent's children list
			affected.push(ChangeChildren {
				edited_parent: old_parent_id,
				new_children: new_old_parent_children,
				old_children: old_parent.children.clone(),
			});
			
			// Change the child's parent
			affected.push(ChangeParent {
				edited_child: child_id,
				old_parent: old_parent_id,
				new_parent: parent_id,
			});
		}
		
		Self { affected }
	}
	
	pub fn apply(&self, anb: &mut AnbWithoutUndo) {
		use ReparentEntry::*;
		
		for entry in &self.affected {
			match entry {
				ChangeParent { edited_child, old_parent: _, new_parent } => {
					anb.node_arena.get_mut(edited_child.0).unwrap().parent = Some(*new_parent);
				},
				ChangeChildren { edited_parent, old_children: _, new_children } => {
					anb.node_arena.get_mut(edited_parent.0).unwrap().children = new_children.clone();
				},
			}
		}
	}
	
	pub fn undo(&self, anb: &mut AnbWithoutUndo) {
		use ReparentEntry::*;
		
		for entry in &self.affected {
			match entry {
				ChangeParent { edited_child, old_parent, new_parent: _ } => {
					anb.node_arena.get_mut(edited_child.0).unwrap().parent = Some(*old_parent);
				},
				ChangeChildren { edited_parent, old_children, new_children: _ } => {
					anb.node_arena.get_mut(edited_parent.0).unwrap().children = old_children.clone();
				},
			}
		}
	}
}

enum ReparentEntry {
	ChangeParent { edited_child: NodeId, old_parent: NodeId, new_parent: NodeId },
	ChangeChildren { edited_parent: NodeId, old_children: Vec<NodeId>, new_children: Vec<NodeId> },
}
