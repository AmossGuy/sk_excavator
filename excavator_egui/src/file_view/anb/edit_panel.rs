use super::{AnbFileView, AnbUiEnv};
use crate::app::context::ExcavatorContext;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{Anb, NodeData, NodeId};

use egui::Ui;

pub struct EditPanelState {
	tab: EditPanelTab,
	rect: Option<egui::Rect>,
	
	selected_nodes: Vec<NodeId>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum EditPanelTab {
	Header,
	Node,
}

impl EditPanelState {
	pub fn new() -> Self {
		Self {
			tab: EditPanelTab::Header,
			rect: None,
			selected_nodes: Vec::new(),
		}
	}
	
	pub fn select_nodes(&mut self, nodes: Vec<NodeId>) {
		if !nodes.is_empty() {
			self.tab = EditPanelTab::Node;
		}
		self.rect = None;
		self.selected_nodes = nodes;
	}
	
	pub fn ui(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>) {
		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.tab, EditPanelTab::Header, "Header");
			ui.selectable_value(&mut self.tab, EditPanelTab::Node, "Node");
		});
		
		ui.separator();
		
		match self.tab {
			EditPanelTab::Header => { self.file_tab_ui(ui, env); },
			EditPanelTab::Node => { self.node_tab_ui(ui, env); },
		}
	}

	fn file_tab_ui(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>) {
		egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
			if let Some(edited) = edit_editable_data(ui, env.globals.anb.get_header()) {
				env.globals.anb.edit_header_props(edited);
			}
		});
	}

	fn node_tab_ui(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>) {
		match self.selected_nodes.as_slice() {
			&[] => { ui.label("No nodes selected; select one to edit"); },
			&[node_id] => { self.node_tab_single(ui, env, node_id); },
			_ => { ui.label("Multiple nodes selected; currently, only one node can be edited at a time"); },
		}
	}

	fn node_tab_single(&mut self, ui: &mut Ui, mut env: AnbUiEnv<'_>, node_id: NodeId) {
		self.node_properties_single(ui, env.reborrow(), node_id);
		self.node_datablock_single(ui, env.reborrow(), node_id);
		self.node_spatial_single(ui, env.reborrow(), node_id);
	}

	fn node_properties_single(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>, node_id: NodeId) {
		egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
			let node = env.globals.anb.get_node(node_id).expect("node should exist");
			if let Some(edited) = edit_editable_data(ui, &node.data) {
				env.globals.anb.edit_node_props(node_id, edited);
			}
		});
	}

	fn node_datablock_single(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>, node_id: NodeId) {
		let node = env.globals.anb.get_node(node_id).expect("node should exist");
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
			_ => {},
		}
	}

	fn node_spatial_single(&mut self, ui: &mut Ui, env: AnbUiEnv<'_>, node_id: NodeId) {
		let node = env.globals.anb.get_node(node_id).expect("node should exist");
		let spatial_view_spec = match &node.data {
			NodeData::Frame(_) => {
				let mut spec = SpatialViewSpec::new();
				spec.scan_frame(&env.globals.anb, node_id);
				spec
			},
			NodeData::Vertex(_) => {
				let mut spec = SpatialViewSpec::new();
				spec.vertex_id = Some(node_id);
				if let Some(parent_id) = env.globals.anb.get_parent_id(node_id) {
					spec.scan_frame(&env.globals.anb, parent_id);
				}
				spec
			},
			_ => { return; },
		};
		
		let rect = self.rect.get_or_insert_with(|| {
			spatial_view_spec.rendering_bbox(&env.globals.anb)
		});
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			egui::Scene::new().zoom_range(0.0..=f32::INFINITY).show(ui, rect, |ui| {
				spatial_view_spec.render_to_painter(ui.painter(), env);
			});
		});
	}
}

pub struct SpatialViewSpec {
	pub texture_id: Option<NodeId>,
	pub vertex_id: Option<NodeId>,
}

impl SpatialViewSpec {
	fn new() -> Self {
		Self {
			texture_id: None,
			vertex_id: None,
		}
	}
	
	fn scan_frame(&mut self, anb: &Anb, frame_id: NodeId) {
		let Some(frame_node) = anb.get_node(frame_id) else { return };
		
		for child_id in frame_node.children.iter().copied() {
			let Some(child_node) = anb.get_node(child_id) else { continue };
			
			let slot = match &child_node.data {
				NodeData::Texture(_) => &mut self.texture_id,
				NodeData::Vertex(_) => &mut self.vertex_id,
				_ => { continue },
			};
			
			if slot.is_none() {
				*slot = Some(child_id);
			}
		}
	}
	
	fn rendering_bbox(&self, anb: &Anb) -> egui::Rect {
		let rect = egui::Rect::NOTHING;
		
		// todo: get the various bboxes and combine them into `rect`
			
		if rect.is_positive() {
			rect
		} else {
			egui::Rect::from_x_y_ranges(-16.0..=16.0, -16.0..=16.0)
		}
	}
	
	fn render_to_painter(&self, painter: &egui::Painter, env: AnbUiEnv<'_>) {
		// todo: draw stuff, of course
		
		if self.vertex_id.is_some() {
			super::render::render_anb_sprite_2(self, painter, env);
		}
	}
}
