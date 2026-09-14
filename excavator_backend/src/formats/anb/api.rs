use super::{*, undo::{AnbCommand, ReparentCommand}};

use std::{borrow::Cow, mem, ops::Deref};
use thunderdome::Arena;

impl Deref for Anb {
	type Target = AnbWithoutUndo;
	
	fn deref(&self) -> &AnbWithoutUndo {
		&self.inner
	}
}

impl AnbWithoutUndo {
	pub fn get_header(&self) -> &Header {
		&self.header
	}
	
	pub fn get_node(&self, id: NodeId) -> Option<&Node> {
		self.node_arena.get(id.0)
	}
	
	pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> + '_ {
		self.node_arena.iter().map(|(index, node)| (NodeId(index), node))
	}
}

impl Anb {
	pub(super) fn from_parts(header: Header, node_arena: Arena<Node>) -> Self {
		Self {
			inner: AnbWithoutUndo { header, node_arena },
			undo: undo_2::Commands::new(),
		}
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
	
	pub fn undo_history_strings(&self) -> Vec<Cow<'static, str>> {
		self.undo.iter().map(|item| {
			use undo_2::CommandItem;
			match item {
				CommandItem::Command(command) => command.description(),
				CommandItem::Undo(x) => match x + 1 {
					1 => "Undo 1 action".into(),
					action_count => format!("Undo {action_count} actions").into(),
				},
			}
		}).collect()
	}
	
	pub fn undo_history_index(&self) -> Option<usize> {
		self.undo.current_command_index()
	}
	
	pub fn undo_go_to_index(&mut self, index: usize) {
		for action in self.undo.undo_or_redo_to_index(index) {
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
