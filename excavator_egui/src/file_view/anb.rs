use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
// use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{def_live as anb, def_live::Anb, load_from_bytes};
// use excavator_backend::formats::wflz;

use egui::{Id, Label, ScrollArea, Ui};
use egui_ltreeview::{NodeConfig, TreeView, TreeViewBuilder};
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(AnbFileView::new(anb))
}

struct AnbFileView {
	anb: Anb,
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut Ui, _excavator: &ExcavatorContext) {
		self.tree_view(ui);
	}
}

impl AnbFileView {
	fn new(anb: Anb) -> Self {
		Self { anb }
	}
	
	fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			TreeView::new(Id::new("tree view")).show(ui, |builder| {
				if let Some(root_id) = self.anb.get_header().root_node {
					self.build_tree_recursively(builder, root_id);
				}
			});
		});
	}
	
	fn build_tree_recursively(&self, builder: &mut TreeViewBuilder<anb::NodeId>, node_id: anb::NodeId) {
		let config = AnbNodeConfig::from_anb_and_id(&self.anb, node_id).expect("node should exist");
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
		ui.add(Label::new("wip").selectable(false));
	}
}
