mod edit_panel;
mod render;
mod tree_panel;
mod vertex_edit;

use crate::app::context::ExcavatorContext;
use crate::file_view::{FileView, FileViewAction};
use excavator_backend::formats::anb::{self, Anb, NodeData, NodeId, NodeTexture, load_from_bytes, save_to_bytes};
use excavator_backend::formats::wflz;
use self::vertex_edit::VertexEdit;

use anyhow::Context;
use egui::Ui;
use egui_ltreeview::TreeViewState;
use std::{borrow::Cow, collections::{HashMap, hash_map}, path::{Path, PathBuf}, sync::{Arc, mpsc}};
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
	node_textures: HashMap<NodeId, egui::TextureHandle>,
	texture_update_needed: bool,
	edit_panel: edit_panel::EditPanelState,
	
	receiver: mpsc::Receiver<Box<dyn FnOnce(&mut AnbFileView) + Send>>,
	sender: mpsc::Sender<Box<dyn FnOnce(&mut AnbFileView) + Send>>,
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext) {
		while let Ok(function) = self.receiver.try_recv() {
			function(self);
		}
		
		if self.texture_update_needed {
			// this shouldn't be on the ui thread probably
			self.update_textures(ui.ctx(), excavator);
			self.texture_update_needed = false;
		}
		
		egui::Panel::right("edit panel").show(ui, |ui| {
			edit_panel::edit_panel_ui(self, ui, excavator);
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
			
			node_textures: HashMap::new(),
			texture_update_needed: true,
			edit_panel: edit_panel::EditPanelState::new(),
			
			receiver, sender,
		}
	}
	
	fn update_textures(&mut self, ctx: &egui::Context, excavator: &ExcavatorContext) {
		fn get_the_rgba(texture_node: &NodeTexture) -> anyhow::Result<([usize; 2], Box<[u8]>)> {
			let size = [texture_node.width as usize, texture_node.height as usize];
			let data = texture_node.data_block.as_ref()
				.context("no texture data block")?
				.data.get();
			let rgba = wflz::decompress(&mut std::io::Cursor::new(data))
				.context("failed to decompress wflz")?;
				
			let (lhs, rhs) = (size[0] * size[1] * 4, rgba.len());
			if lhs != rhs {
				anyhow::bail!("wrong texture size: {} != {}", lhs, rhs);
			}
				
			Ok((size, rgba))
		}
		
		for (id, node) in self.anb.iter_nodes() {
			let texture_node = match node.data {
				NodeData::Texture(ref tx) => tx,
				_ => continue,
			};
			
			let hash_map::Entry::Vacant(entry) = self.node_textures.entry(id) else {
				continue;
			};
			
			let (size, rgba) = match get_the_rgba(texture_node) {
				Ok((size, rgba)) => (size, rgba),
				Err(e) => {
					excavator.display_error(e.context("failed to load a node's texture"));
					continue;
				},
			};
			
			let handle = ctx.load_texture(
				"anb texture",
				egui::ColorImage::from_rgba_unmultiplied(size, &rgba),
				egui::TextureOptions {
					minification: egui::TextureFilter::Linear,
					..egui::TextureOptions::NEAREST
				},
			);
			
			entry.insert(handle);
		}
	}
	
	/*
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
					vertex_edit.ui(ui, &self.anb, node_id, &self.node_textures);
				}
			},
			NodeData::Texture(_texture_node) => {
				self.texture_data_block_editor(ui, node_id, excavator);
			},
			NodeData::Frame(_) => {
				egui::Frame::canvas(ui.style()).show(ui, |ui| {
					render::render_anb_sprite(ui, &self.anb, node_id, &self.node_textures);
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
				if let Some(texture) = self.node_textures.get(&id) {
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
				
				let result = Self::do_export_texture(&texture_node, &path)
					.context("failed to export texture");
				excavator.display_if_error(result);
			}
		});
	}
	
	fn do_export_texture(texture_node: &NodeTexture, path: &Path) -> anyhow::Result<()> {
		let data = texture_node.data_block.as_ref()
			.context("no data block")?
			.data.get();
		
		let rgba = wflz::decompress(std::io::Cursor::new(data))
			.context("failed to decompress wflz")?;
		::image::save_buffer(path, &rgba, texture_node.width, texture_node.height, ::image::ColorType::Rgba8)
			.context("failed to save image")?;
		
		Ok(())
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
				
				let result = Self::do_import_texture(node_id, &path, &sender)
					.context("failed to import texture");
				excavator.display_if_error(result);
			}
		});
	}
	
	fn do_import_texture(node_id: NodeId, path: &Path, sender: &mpsc::Sender<Box<dyn FnOnce(&mut AnbFileView) + Send>>) -> anyhow::Result<()> {
		let image = ::image::open(&path)
			.context("failed to load image")?;
		let dimensions = [image.width(), image.height()];
		let raw: Vec<u8> = image.into_rgba8().into_flat_samples().samples;
		let compressed = wflz::compress(&raw);
		
		let _ = sender.send(Box::new(move |anb_view| {
			anb_view.anb.edit_replace_texture_data(node_id, dimensions, compressed);
			// invalidate cache for this texture
			anb_view.node_textures.remove(&node_id);
			anb_view.texture_update_needed = true;
		}));
		
		Ok(())
	}
	*/
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
