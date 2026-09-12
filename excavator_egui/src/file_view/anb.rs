use crate::app::context::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{self, Anb, NodeData, NodeId, VertexEntry, load_from_bytes};
use excavator_backend::formats::wflz;

use egui::{Id, Label, Pos2, Rect, ScrollArea, Ui, Vec2, WidgetText};
use egui_ltreeview::{Action as TreeAction, DirPosition, NodeConfig, TreeView, TreeViewBuilder, TreeViewState};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(AnbFileView::new(anb))
}

struct AnbFileView {
	anb: Anb,
	tree_state: TreeViewState<anb::NodeId>,
	node_textures: Option<HashMap<NodeId, egui::TextureHandle>>,
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut Ui, _excavator: &ExcavatorContext) {
		// this shouldn't be on the ui thread!!
		self.update_textures(ui.ctx());
		
		egui::Panel::right("property editor").show(ui, |ui| {
			self.property_view(ui);
			ui.take_available_space();
		});
		
		egui::CentralPanel::default().show(ui, |ui| {
			self.tree_view(ui);
		});
	}
	
	fn can_undo(&self) -> bool {
		self.anb.can_undo()
	}
	
	fn execute_undo(&mut self) {
		self.anb.undo();
	}
	
	fn can_redo(&self) -> bool {
		self.anb.can_redo()
	}
	
	fn execute_redo(&mut self) {
		self.anb.redo();
	}
	
	fn undo_history(&self) -> Option<Vec<Cow<'static, str>>> {
		Some(self.anb.undo_history_strings())
	}
	
	fn undo_history_index(&self) -> Option<usize> {
		self.anb.undo_history_index()
	}
	
	fn undo_go_to_index(&mut self, index: usize) {
		self.anb.undo_go_to_index(index);
	}
}

impl AnbFileView {
	fn new(anb: Anb) -> Self {
		Self { anb, tree_state: TreeViewState::default(), node_textures: None }
	}
	
	fn update_textures(&mut self, ctx: &egui::Context) {
		if self.node_textures.is_none() {
			let node_textures = self.anb.iter_nodes().filter_map(|(id, node)| {
				let texture_node = match node.data {
					NodeData::Texture(ref tx) => tx,
					_ => return None,
				};
				
				let size = [texture_node.width as usize, texture_node.height as usize];
				let data = texture_node.data_block.as_ref().unwrap().data.get();
				let rgba = wflz::decompress(&mut std::io::Cursor::new(data)).unwrap();
				
				/*
				let (lhs, rhs) = (size[0] * size[1] * 4, rgba.len());
				if lhs != rhs {
					println!("wrong texture size? {} != {}", lhs, rhs);
					return None;
				}
				*/
				
				let handle = ctx.load_texture(
					"anb texture",
					egui::ColorImage::from_rgba_unmultiplied(size, &rgba),
					egui::TextureOptions {
						minification: egui::TextureFilter::Linear,
						..egui::TextureOptions::NEAREST
					},
				);
				
				Some((id, handle))
			}).collect::<HashMap<_, _>>();
			
			self.node_textures = Some(node_textures)
		}
	}
	
	fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			let tree_view = TreeView::new(Id::new("tree view"));
			let stuff = TreeBuildStuff { anb: &self.anb, node_textures: self.node_textures.as_ref() };
			
			let (_, actions) = tree_view.show_state(ui, &mut self.tree_state, |builder| {
				if let Some(root_id) = self.anb.get_header().root_node {
					stuff.build_tree_recursively(builder, root_id);
				}
			});
			
			self.handle_tree_actions(actions);
		});
	}
	
	fn property_view(&mut self, ui: &mut Ui) {
		match self.tree_state.selected().as_slice() {
			// I need to change this in some way, because the tree view does not provide a convenient way to deselect everything. I'm thinking tab buttons.
			&[] => {
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					if let Some(edited) = edit_editable_data(ui, self.anb.get_header()) {
						self.anb.edit_header_props(edited);
					}
				});
			},
			&[node_id] => {
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					let node = self.anb.get_node(node_id).expect("node should exist");
					if let Some(edited) = edit_editable_data(ui, &node.data) {
						self.anb.edit_node_props(node_id, edited);
					}
				});
				
				self.data_block_editor(ui, node_id);
			},
			_ => {
				ui.label("multiple selected");
			},
		}
	}
	
	fn handle_tree_actions(&mut self, actions: Vec<TreeAction<NodeId>>) {
		for action in actions {
			match action {
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
	
	fn data_block_editor(&mut self, ui: &mut Ui, node_id: NodeId) {
		match &self.anb.get_node(node_id).unwrap().data {
			NodeData::Vertex(vertex_node) => {
				// Not the sort of thing that should be done every frame, I think, but this is just a test implementation for now
				let parse_result = vertex_node.parse_data_block();
				
				match parse_result {
					Err(e) => { ui.label(format!("data block problem: {e}")); },
					Ok(parsed) => { Self::vertex_data_block_editor(ui, parsed); },
				}
			},
			NodeData::Texture(_texture_node) => {
				self.texture_data_block_editor(ui, node_id);
			},
			_ => {},
		}
	}
	
	fn vertex_data_block_editor(ui: &mut Ui, parsed: Vec<VertexEntry>) {
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			// TODO: rect needs to be stored
			let mut rect = Rect::from_x_y_ranges(-10.0..=10.0, -10.0..=10.0);
			
			egui::Scene::new().show(ui, &mut rect, |ui| {
				let painter = ui.painter();
				for entry in parsed {
					painter.rect(
						Rect::from_min_size(
							Pos2::new(entry.position_x, entry.position_y),
							Vec2::new(entry.width.into(), entry.height.into()),
						),
						egui::CornerRadius::ZERO,
						egui::Color32::GREEN,
						egui::Stroke::new(2.0, egui::Color32::DARK_GREEN),
						egui::StrokeKind::Middle,
					);
				}
			});
		});
	}
	
	fn texture_data_block_editor(&self, ui: &mut Ui, id: NodeId) {
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			if let Some(texture) = self.node_textures.as_ref().and_then(|t| t.get(&id)) {
				ui.add(egui::Image::new(&*texture).fit_to_exact_size(ui.available_size()));
			}
		});
	}
}

struct TreeBuildStuff<'a> {
	anb: &'a Anb,
	node_textures: Option<&'a HashMap<NodeId, egui::TextureHandle>>,
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
	node_textures: Option<&'a HashMap<NodeId, egui::TextureHandle>>,
}

impl<'a> AnbNodeConfig<'a> {
	fn from_stuff_and_id(stuff: &'a TreeBuildStuff<'_>, id: anb::NodeId) -> Option<Self> {
		let value = stuff.anb.get_node(id)?;
		let node_textures = stuff.node_textures;
		Some(Self { id, value, node_textures })
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
			_ => false,
		}
	}
	
	fn icon(&mut self, ui: &mut Ui) {
		match self.value.data {
			NodeData::Texture(_) => {
				if let Some(texture) = self.node_textures.and_then(|t| t.get(&self.id)) {
					ui.add(egui::Image::new(&*texture).max_size(ui.available_size()));
				}
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
