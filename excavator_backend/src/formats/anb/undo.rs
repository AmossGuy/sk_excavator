use super::{AnbWithoutUndo, Header, NodeData, NodeId};
use undo_2::Action;

pub enum AnbCommand {
	EditHeaderProps { old: Header, new: Header },
	EditNodeProps { id: NodeId, old: NodeData, new: NodeData },
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
		}
	}
}
