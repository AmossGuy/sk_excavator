use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{self, Anb, load_from_bytes};
// use excavator_backend::formats::wflz;

use egui::{Id, Label, ScrollArea, Ui, WidgetText};
use egui_ltreeview::{NodeConfig, TreeView, TreeViewBuilder, TreeViewState};
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(AnbFileView::new(anb))
}

struct AnbFileView {
	anb: Anb,
	tree_state: TreeViewState<anb::NodeId>,
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut Ui, _excavator: &ExcavatorContext) {
		egui::Panel::right("property editor").show(ui, |ui| {
			self.property_view(ui);
			ui.take_available_space();
		});
		
		egui::CentralPanel::default().show(ui, |ui| {
			self.tree_view(ui);
		});
	}
}

impl AnbFileView {
	fn new(anb: Anb) -> Self {
		Self { anb, tree_state: TreeViewState::default() }
	}
	
	fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			let tree_view = TreeView::new(Id::new("tree view"));
			let stuff = TreeBuildStuff { anb: &self.anb };
			
			tree_view.show_state(ui, &mut self.tree_state, |builder| {
				if let Some(root_id) = self.anb.get_header().root_node {
					stuff.build_tree_recursively(builder, root_id);
				}
			});
		});
	}
	
	fn property_view(&mut self, ui: &mut Ui) {
		match self.tree_state.selected().as_slice() {
			// I need to change this in some way, because the tree view does not provide a convenient way to deselect everything. I'm thinking tab buttons.
			&[] => {
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					edit_editable_data(ui, self.anb.get_header());
				});
			},
			&[node_id] => {
				let node = self.anb.get_node(node_id).expect("node should exist");
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					edit_editable_data(ui, &node.data);
				});
			},
			_ => {
				ui.label("multiple selected");
			},
		}
	}
}

struct TreeBuildStuff<'a> {
	anb: &'a Anb,
}

impl<'a> TreeBuildStuff<'a> {
	fn build_tree_recursively(&self, builder: &mut TreeViewBuilder<anb::NodeId>, node_id: anb::NodeId) {
		let config = AnbNodeConfig::from_anb_and_id(self.anb, node_id).expect("node should exist");
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
}

impl<'a> AnbNodeConfig<'a> {
	fn from_anb_and_id(anb: &'a Anb, id: anb::NodeId) -> Option<Self> {
		let value = anb.get_node(id)?;
		Some(Self { id, value })
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
		use anb::NodeData;
		
		match self.value.data {
			NodeData::Frame(_) | NodeData::Sequence(_) => false,
			_ => true,
		}
	}
}

fn node_label(node: &anb::Node) -> WidgetText {
	use anb::NodeData;
	
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
