mod render;
mod vertex_edit;

use crate::app::context::ExcavatorContext;
use crate::file_view::{FileView, FileViewAction};
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::anb::{self, Anb, NodeData, NodeId, load_from_bytes, save_to_bytes};
use excavator_backend::formats::wflz;
use self::vertex_edit::VertexEdit;

use egui::{Id, Label, ScrollArea, Ui, WidgetText};
use egui_ltreeview::{Action as TreeAction, DirPosition, NodeConfig, TreeView, TreeViewBuilder, TreeViewState};
use std::{borrow::Cow, collections::HashMap, path::{Path, PathBuf}, sync::{Arc, mpsc}};
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>, file_path: PathBuf) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(AnbFileView::new(anb, file_path))
}

struct AnbFileView {
	anb: Anb,
	file_path: PathBuf,
	
	tree_state: TreeViewState<anb::NodeId>,
	node_textures: Option<HashMap<NodeId, egui::TextureHandle>>,
	vertex_edit: Option<VertexEdit>,
	
	receiver: mpsc::Receiver<Box<dyn FnOnce(&mut AnbFileView) + Send>>,
	sender: mpsc::Sender<Box<dyn FnOnce(&mut AnbFileView) + Send>>,
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext) {
		while let Ok(function) = self.receiver.try_recv() {
			function(self);
		}
		
		// this shouldn't be on the ui thread!!
		self.update_textures(ui.ctx(), excavator);
		
		egui::Panel::right("property editor").show(ui, |ui| {
			self.property_view(ui, excavator);
			ui.take_available_space();
		});
		
		egui::CentralPanel::default().show(ui, |ui| {
			self.tree_view(ui);
		});
	}
	
	fn tab_title(&self) -> Option<Cow<'_, str>> {
		self.file_path.file_name().map(|name| name.to_string_lossy())
	}
	
	fn action_execute(&mut self, action: FileViewAction, _excavator: &ExcavatorContext) {
		match action {
			FileViewAction::Save => self.wip_save(),
			FileViewAction::SaveAs => self.wip_save_as(),
			FileViewAction::Undo => self.anb.undo(),
			FileViewAction::Redo => self.anb.redo(),
		}
	}
	fn action_should_be_enabled(&self, action: FileViewAction) -> bool {
		match action {
			FileViewAction::Save => true,
			FileViewAction::SaveAs => true,
			FileViewAction::Undo => self.anb.can_undo(),
			FileViewAction::Redo => self.anb.can_redo(),
		}
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
	fn new(anb: Anb, file_path: PathBuf) -> Self {
		let (sender, receiver) = mpsc::channel();
		
		Self {
			anb, file_path, tree_state: TreeViewState::default(),
			node_textures: None, vertex_edit: None,
			receiver, sender,
		}
	}
	
	fn update_textures(&mut self, ctx: &egui::Context, excavator: &ExcavatorContext) {
		if self.node_textures.is_none() {
			let node_textures = self.anb.iter_nodes().filter_map(|(id, node)| {
				let texture_node = match node.data {
					NodeData::Texture(ref tx) => tx,
					_ => return None,
				};
				
				let size = [texture_node.width as usize, texture_node.height as usize];
				let data = texture_node.data_block.as_ref().unwrap().data.get();
				let rgba = wflz::decompress(&mut std::io::Cursor::new(data)).unwrap();
				
				let (lhs, rhs) = (size[0] * size[1] * 4, rgba.len());
				if lhs != rhs {
					excavator.display_error(anyhow::anyhow!("wrong texture size: {} != {}", lhs, rhs));
					return None;
				}
				
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
	
	fn property_view(&mut self, ui: &mut Ui, excavator: &ExcavatorContext) {
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
				
				self.data_block_editor(ui, node_id, excavator);
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
	
	fn data_block_editor(&mut self, ui: &mut Ui, node_id: NodeId, excavator: &ExcavatorContext) {
		let Some(node) = &self.anb.get_node(node_id) else { return };
		
		match &node.data {
			NodeData::Vertex(vertex_node) => {
				if self.vertex_edit.as_ref().is_none_or(|ve| ve.node_id() != node_id) {
					match vertex_node.parse_data_block() {
						Ok(parsed) => {
							self.vertex_edit = Some(VertexEdit::new(node_id, parsed));
						},
						Err(e) => excavator.display_error(e),
					}
				}
				
				if let Some(vertex_edit) = &mut self.vertex_edit {
					vertex_edit.ui(ui);
				}
			},
			NodeData::Texture(_texture_node) => {
				self.texture_data_block_editor(ui, node_id, excavator);
			},
			NodeData::Frame(_) => {
				egui::Frame::canvas(ui.style()).show(ui, |ui| {
					render::render_anb_sprite(ui, &self.anb, node_id, self.node_textures.as_ref());
				});
			},
			NodeData::Sequence(animation_node) => {
				match excavator.unhash(animation_node.hashname) {
					Some(name) => {
						let name = String::from_utf8_lossy(&name);
						ui.label(format!("hash-reversed name: {}", name));
					},
					None => {
						ui.label(format!("unknown hash"));
					},
				}
			},
			_ => {},
		}
	}
	
	fn texture_data_block_editor(&self, ui: &mut Ui, id: NodeId, excavator: &ExcavatorContext) {
		ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
			ui.horizontal(|ui| {
				if ui.button("Export texture").clicked() {
					self.open_export_texture_dialog(id, excavator);
				}
				if ui.button("Import texture").clicked() {
					self.open_import_texture_dialog(id, excavator);
				}
			});
			
			egui::Frame::canvas(ui.style()).show(ui, |ui| {
				if let Some(texture) = self.node_textures.as_ref().and_then(|t| t.get(&id)) {
					ui.add(egui::Image::new(&*texture).fit_to_exact_size(ui.available_size()));
				}
			});
		});
	}
	
	fn open_export_texture_dialog(&self, node_id: NodeId, excavator: &ExcavatorContext) {
		let texture_node = {
			let NodeData::Texture(texture_node) = &self.anb.get_node(node_id).unwrap().data else {
				panic!("export texture: not texture");
			};
			texture_node.clone()
		};
		let excavator = excavator.clone();
		
		std::thread::spawn(move || {
			let mut dialog = rfd::FileDialog::new()
				.set_title("Export Texture — Excavator");
			
			if let Some(path) = excavator.settings(|s| s.export_texture_dialog_dir.clone()) {
				dialog = dialog.set_directory(path);
			}
			
			if let Some(path) = dialog.save_file() {
				let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
				excavator.settings_mut(|s| s.export_texture_dialog_dir = Some(parent));
				
				let data = texture_node.data_block.as_ref().unwrap().data.get();
				let rgba = wflz::decompress(std::io::Cursor::new(data)).unwrap();
				::image::save_buffer(&path, &rgba, texture_node.width, texture_node.height, ::image::ColorType::Rgba8).unwrap();
			}
		});
	}
	
	fn open_import_texture_dialog(&self, node_id: NodeId, excavator: &ExcavatorContext) {
		let excavator = excavator.clone();
		let sender = self.sender.clone();
		
		std::thread::spawn(move || {
			let mut dialog = rfd::FileDialog::new()
				.set_title("Import Texture — Excavator");
			
			if let Some(path) = excavator.settings(|s| s.import_texture_dialog_dir.clone()) {
				dialog = dialog.set_directory(path);
			}
			
			if let Some(path) = dialog.pick_file() {
				let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
				excavator.settings_mut(|s| s.import_texture_dialog_dir = Some(parent));
				
				let raw: Vec<u8> = ::image::open(&path).unwrap().into_rgba8().into_flat_samples().samples;
				let compressed = wflz::compress(&raw);
				
				let _ = sender.send(Box::new(move |anb_view| {
					anb_view.anb.edit_replace_texture_data(node_id, compressed);
				}));
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
	
	anb: &'a anb::Anb,
	node_textures: Option<&'a HashMap<NodeId, egui::TextureHandle>>,
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
				if let Some(texture) = self.node_textures.and_then(|t| t.get(&self.id)) {
					ui.add(egui::Image::new(&*texture).max_size(ui.available_size()));
				}
			},
			NodeData::Frame(_) => {
				render::render_anb_sprite(ui, &self.anb, self.id, self.node_textures);
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

// test implementation
// this stuff should be made shared between file formats soon
impl AnbFileView {
	fn wip_save(&self) {
		let path = self.file_path.clone();
		let mut bak_path = path.clone();
		bak_path.add_extension("original");
		
		// todo: perhaps use `renamore` crate?
		if !std::fs::exists(&bak_path).unwrap() {
			// good error handling isn't yet implemented here, but i really don't want it to overwrite the original
			std::fs::rename(&path, &bak_path).unwrap();
		}
		
		if let Ok(bytes) = save_to_bytes(&self.anb) {
			std::thread::spawn(move || {
				let _ = std::fs::write(path, bytes);
			});
		}
	}
	
	fn wip_save_as(&self) {
		let sender = self.sender.clone();
		
		std::thread::spawn(move || {
			let dialog = rfd::FileDialog::new();
			
			if let Some(path) = dialog.save_file() {
				let _ = sender.send(Box::new(|anb_view| {
					if let Ok(bytes) = save_to_bytes(&anb_view.anb) {
						std::thread::spawn(move || {
							let _ = std::fs::write(path, bytes);
						});
					}
				}));
			}
		});
	}
}
