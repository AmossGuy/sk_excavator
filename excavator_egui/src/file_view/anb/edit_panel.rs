use super::AnbFileView;
use crate::app::context::ExcavatorContext;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{NodeData, NodeId};

use egui::Ui;

pub struct EditPanelState {
	tab: EditPanelTab,
}

impl EditPanelState {
	pub fn new() -> Self {
		Self {
			tab: EditPanelTab::Header,
		}
	}
	
	pub fn show_node_tab(&mut self) {
		self.tab = EditPanelTab::Node;
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum EditPanelTab {
	Header,
	Node,
}

pub fn edit_panel_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.horizontal(|ui| {
		ui.selectable_value(&mut anb_view.edit_panel.tab, EditPanelTab::Header, "Header");
		ui.selectable_value(&mut anb_view.edit_panel.tab, EditPanelTab::Node, "Node");
	});
	
	ui.separator();
	
	match anb_view.edit_panel.tab {
		EditPanelTab::Header => { file_tab_ui(anb_view, ui, excavator); },
		EditPanelTab::Node => { node_tab_ui(anb_view, ui, excavator); },
	}
}

fn file_tab_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
		if let Some(edited) = edit_editable_data(ui, anb_view.anb.get_header()) {
			anb_view.anb.edit_header_props(edited);
		}
	});
}

fn node_tab_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	match anb_view.tree_state.selected().as_slice() {
		&[] => { ui.label("No nodes selected; select one to edit"); },
		&[node_id] => { node_tab_single(anb_view, node_id, ui, excavator); },
		_ => { ui.label("Multiple nodes selected; currently, only one node can be edited at a time"); },
	}
}

fn node_tab_single(anb_view: &mut AnbFileView, node_id: NodeId, ui: &mut Ui, excavator: &ExcavatorContext) {
	node_properties_single(anb_view, node_id, ui, excavator);
	node_datablock_single(anb_view, node_id, ui, excavator);
}

fn node_properties_single(anb_view: &mut AnbFileView, node_id: NodeId, ui: &mut Ui, excavator: &ExcavatorContext) {
	egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
		let node = anb_view.anb.get_node(node_id).expect("node should exist");
		if let Some(edited) = edit_editable_data(ui, &node.data) {
			anb_view.anb.edit_node_props(node_id, edited);
		}
	});
}

// TODO: I feel the cleanliness of this code could really benefit from a system where edits are queued up, only applied after the UI code is finished
fn node_datablock_single(anb_view: &mut AnbFileView, node_id: NodeId, ui: &mut Ui, excavator: &ExcavatorContext) {
	let node = anb_view.anb.get_node(node_id).expect("node should exist");
	let datablock = match &node.data {
		NodeData::Texture(d) => d.data_block.as_ref(),
		NodeData::Vertex(d) => d.data_block.as_ref(),
		NodeData::MetaString(d) => d.data_block.as_ref(),
		NodeData::MetaTable(d) => d.data_block.as_ref(),
		NodeData::Animation(d) => d.data_block.as_ref(),
		_ => { return; },
	};
	
	ui.separator();
		
	egui::Grid::new("datablock property grid").num_columns(2).show(ui, |ui| {
		ui.label("Has data block");
		let mut checked = datablock.is_some();
		ui.checkbox(&mut checked, ());
		// todo respond to checkbox
		ui.end_row();
		
		if let Some(datablock) = datablock {
			if let Some(edited) = edit_editable_data(ui, datablock) {
				// todo respond to edit
			}
		}
	});
	
	match &node.data {
		NodeData::Texture(_d) => { ui.label("texture datablock todo"); },
		NodeData::Vertex(_d) => { ui.label("vertex datablock todo"); },
		NodeData::MetaString(_d) => { ui.label("meta string datablock todo"); },
		NodeData::MetaTable(_d) => { ui.label("meta table datablock todo"); },
		NodeData::Animation(_d) => { ui.label("animation datablock todo"); },
		_ => unreachable!(),
	}
}
