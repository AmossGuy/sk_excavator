use excavator_backend::formats::anb::{self, Anb, NodeData, NodeId};

use egui::{Id, Label, ScrollArea, Ui, WidgetText};
use egui_ltreeview::{Action as TreeAction, DirPosition, NodeConfig, TreeView, TreeViewBuilder};
use std::collections::HashMap;

impl super::AnbFileView {
	pub(super) fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			let tree_view = TreeView::new(Id::new("tree view"));
			let stuff = TreeBuildStuff { anb: &self.anb, node_textures: &self.node_textures };
			
			let (_, actions) = tree_view.show_state(ui, &mut self.tree_state, |builder| {
				if let Some(root_id) = self.anb.get_header().root_node {
					stuff.build_tree_recursively(builder, root_id);
				}
			});
			
			self.handle_tree_actions(actions);
		});
	}
	
	fn handle_tree_actions(&mut self, actions: Vec<TreeAction<NodeId>>) {
		for action in actions {
			match action {
				TreeAction::SetSelected(selection) => {
					if !selection.is_empty() {
						self.edit_panel.show_and_reset_node_tab();
					}
				},
				TreeAction::Move(drag_and_drop) => {
					let children = &self.anb.get_node(drag_and_drop.target)
						.expect("node should exist")
						.children;
					
					let index = match drag_and_drop.position {
						DirPosition::First => 0,
						DirPosition::Last => children.len(),
						DirPosition::After(id) => children.iter().position(|&x| x == id).unwrap() + 1,
						DirPosition::Before(id) => children.iter().position(|&x| x == id).unwrap(),
					};
					
					self.anb.edit_reparent(drag_and_drop.target, &drag_and_drop.source, index);
				},
				_ => {},
			}
		}
	}
}

struct TreeBuildStuff<'a> {
	anb: &'a Anb,
	node_textures: &'a HashMap<NodeId, egui::TextureHandle>,
}

impl<'a> TreeBuildStuff<'a> {
	fn build_tree_recursively(&self, builder: &mut TreeViewBuilder<anb::NodeId>, node_id: anb::NodeId) {
		let config = AnbNodeConfig::from_stuff_and_id(self, node_id).expect("node should exist");
		let (children, is_dir) = (&config.value.children, config.is_dir());
		
		let is_open = builder.node(config);
		
		if is_dir && is_open {
			for &child_id in children {
				self.build_tree_recursively(builder, child_id);
			}
		}
		
		if is_dir {
			builder.close_dir();
		}
	}
}

struct AnbNodeConfig<'a> {
	id: anb::NodeId,
	value: &'a anb::Node,
	
	anb: &'a anb::Anb,
	node_textures: &'a HashMap<NodeId, egui::TextureHandle>,
}

impl<'a> AnbNodeConfig<'a> {
	fn from_stuff_and_id(stuff: &'a TreeBuildStuff<'_>, id: anb::NodeId) -> Option<Self> {
		let anb = stuff.anb;
		let value = anb.get_node(id)?;
		let node_textures = stuff.node_textures;
		Some(Self { id, value, anb, node_textures })
	}
}

impl<'a> NodeConfig<anb::NodeId> for AnbNodeConfig<'a> {
	fn id(&self) -> &anb::NodeId {
		&self.id
	}
	
	fn is_dir(&self) -> bool {
		!self.value.children.is_empty()
	}
	
	fn label(&mut self, ui: &mut Ui) {
		let text = node_label(self.value);
		ui.add(Label::new(text).selectable(false));
	}
	
	fn default_open(&self) -> bool {
		match self.value.data {
			NodeData::Frame(_) | NodeData::Sequence(_) => false,
			_ => true,
		}
	}
	
	fn drop_allowed(&self) -> bool {
		true
	}
	
	fn has_custom_icon(&self) -> bool {
		match self.value.data {
			NodeData::Texture(_) => true,
			NodeData::Frame(_) => true,
			_ => false,
		}
	}
	
	fn icon(&mut self, ui: &mut Ui) {
		// TODO: a lot of this stuff should probably be cached
		match self.value.data {
			NodeData::Texture(_) => {
				if let Some(texture) = self.node_textures.get(&self.id) {
					ui.add(egui::Image::new(&*texture).max_size(ui.available_size()));
				}
			},
			NodeData::Frame(_) => {
				super::render::render_anb_sprite(ui, &self.anb, self.id, self.node_textures);
			},
			_ => {},
		}
	}
}

fn node_label(node: &anb::Node) -> WidgetText {
	match node.data {
		NodeData::Base => "Base node".into(),
		NodeData::Texture(_) => "Texture".into(),
		NodeData::Vertex(_) => "Vertex data".into(),
		NodeData::Meta => "Meta".into(),
		NodeData::MetaScalar(_) => "Meta scalar".into(),
		NodeData::MetaPoint(_) => "Meta point".into(),
		NodeData::MetaAnchor(_) => "Meta anchor".into(),
		NodeData::MetaRect(_) => "Meta rect".into(),
		NodeData::MetaString(_) => "Meta string".into(),
		NodeData::MetaTable(_) => "Meta table".into(),
		NodeData::Frame(_) => "Frame".into(),
		NodeData::SequenceFrame(_) => "Sequence frame".into(),
		NodeData::Sequence(_) => "Sequence".into(),
		NodeData::Animation(_) => "Animation".into(),
	}
}
