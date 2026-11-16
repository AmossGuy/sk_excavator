use excavator_backend::formats::anb::{Anb, NodeData, NodeId, NodeMetaRect, VertexEntry};
use egui::{CursorIcon, Pos2, Rect, TextureHandle, Vec2};
// HashMap only needed for some weirdness that may or may not be refactored out
use std::{any::Any, collections::HashMap, fmt::Debug, hash::Hash};

const DRAG_EDGE_THICKNESS: f32 = 3.0;

pub struct SpatialEdit {
	primary_node: NodeId,
	scene_rect: Rect,
	state: SpatialEditState,
	
	is_vertex_edit: bool,
	
	data: SpatialEditData,
}

#[derive(Default)]
struct SpatialEditData {
	texture: Option<(NodeId, TextureHandle)>,
	vertex: Option<(NodeId, Vec<VertexEntry>)>,
	metas: Vec<(NodeId, NodeMetaRect)>,
}

#[derive(Default)]
struct SpatialEditState {
	drag: Option<DragInfo>,
}

impl SpatialEdit {
	pub fn new(anb: &Anb, primary_node: NodeId, node_textures: &HashMap<NodeId, TextureHandle>) -> Self {
		let mut data = SpatialEditData::default();
		if let Some(frame_node) = get_frame_node(anb, primary_node) {
			data.load_from_frame(anb, frame_node, node_textures);
		}
		
		let is_vertex_edit = anb.get_node(primary_node)
			.is_some_and(|n| matches!(&n.data, NodeData::Vertex(_)));
		
		Self {
			primary_node,
			scene_rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(64.0)),
			state: SpatialEditState::default(),
			is_vertex_edit,
			data,
		}
	}
	
	pub fn primary_node(&self) -> NodeId {
		self.primary_node
	}
}

fn is_frame_node(anb: &Anb, some_node: NodeId) -> bool {
	anb.get_node(some_node).is_some_and(|n| {
		matches!(&n.data, NodeData::Frame(_) | NodeData::SequenceFrame(_))
	})
}

/// find a frame node that is either the passed node or its immediate parent
fn get_frame_node(anb: &Anb, some_node: NodeId) -> Option<NodeId> {
	if is_frame_node(anb, some_node) {
		Some(some_node)
	} else {
		let parent = anb.get_node(some_node)?.parent?;
		is_frame_node(anb, parent).then_some(parent)
	}
}

impl SpatialEditData {
	fn load_from_frame(&mut self, anb: &Anb, frame_node: NodeId, node_textures: &HashMap<NodeId, TextureHandle>) {
		let Some(parent_ref) = anb.get_node(frame_node) else { return };
		for child_node in parent_ref.children.iter().copied() {
			let Some(child_ref) = anb.get_node(child_node) else { continue };
			
			match child_ref.data {
				NodeData::Texture(ref _t) => {
					let Some(cool) = node_textures.get(&child_node) else { continue };
					self.texture.get_or_insert_with(|| (child_node, cool.clone()));
				},
				NodeData::Vertex(ref v) => {
					// really great error handling amirite
					self.vertex.get_or_insert_with(|| (child_node, v.parse_data_block().unwrap_or_default()));
				},
				NodeData::MetaTable(ref _m) => {
					for meta_node in child_ref.children.iter().copied() {
						let Some(meta_ref) = anb.get_node(meta_node) else { continue };
						
						match meta_ref.data {
							NodeData::MetaRect(ref r) => {
								self.metas.push((meta_node, r.clone()));
							},
							_ => {},
						}
					}
				},
				_ => {},
			}
		}
	}
}

impl SpatialEdit {
	pub fn ui(&mut self, ui: &mut egui::Ui) {
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			egui::Scene::new().zoom_range(0.0..=f32::INFINITY).show(ui, &mut self.scene_rect, |ui| {
				if let Some((_, texture)) = &self.data.texture && let Some((_, vertex)) = &mut self.data.vertex {
					let mesh = super::render::build_vertex_mesh(vertex, texture);
					ui.painter().add(mesh);
				}
				
				if self.is_vertex_edit && let Some((_, vertex)) = &mut self.data.vertex {
					for (i, entry) in vertex.iter_mut().enumerate() {
						self.state.editable_rect_ui(ui, ("vertex", i), entry);
					}
				}
				
				for (i, meta_rect) in self.data.metas.iter_mut() {
					self.state.editable_rect_ui(ui, ("meta", i), meta_rect);
				}
			});
		});
	}
}

impl SpatialEditState {
	fn editable_rect_ui(&mut self, ui: &mut egui::Ui, id: impl Hash + Debug, thingy: &mut impl EditableRect) {
		self.editable_rect_interact(ui, id, thingy);
		self.editable_rect_draw(ui.painter(), thingy.rect(), thingy.color());
	}
		
	fn editable_rect_draw(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
		let darker_color = color.lerp_to_gamma(egui::Color32::BLACK, 0.2);
		painter.rect(
			rect,
			egui::CornerRadius::ZERO,
			color.gamma_multiply(0.5),
			egui::Stroke::new(1.0, darker_color.gamma_multiply(0.5)),
			egui::StrokeKind::Inside,
		);
	}
	
	fn editable_rect_interact<T: EditableRect>(&mut self, ui: &mut egui::Ui, id: impl Hash + Debug, thingy: &mut T) {
		let rect = thingy.rect();
		let response = ui.interact(rect.expand(DRAG_EDGE_THICKNESS / 2.0), ui.id().with(id), egui::Sense::DRAG);
		
		if let Some(cursor_pos) = response.hover_pos() {
			if response.drag_started() {
				let kind = DragKind::from_hover_pos(rect.shrink(DRAG_EDGE_THICKNESS / 2.0), cursor_pos);
				self.drag = Some(DragInfo { kind, start_state: Box::new(thingy.clone()) });
			} else if !response.dragged() {
				self.drag = None;
			}
			
			if let Some(drag) = &self.drag {
				let delta = response.total_drag_delta().unwrap_or_default();
				let delta = delta.round();
				
				if let Some(start_state) = drag.start_state.downcast_ref::<T>() {
					*thingy = start_state.clone();
					thingy.apply_drag(delta, drag.kind);
				}
				
				ui.set_cursor_icon(drag.kind.cursor(DragState::Drag));
			} else if response.hovered() {
				let kind = DragKind::from_hover_pos(rect.shrink(DRAG_EDGE_THICKNESS / 2.0), cursor_pos);
				ui.set_cursor_icon(kind.cursor(DragState::Hover));
			}
		}
	}
}

trait EditableRect: Any + Clone + Send {
	fn rect(&self) -> egui::Rect;
	fn color(&self) -> egui::Color32;
	fn apply_drag(&mut self, delta: egui::Vec2, kind: DragKind);
}

impl EditableRect for VertexEntry {
	fn rect(&self) -> egui::Rect {
		Rect::from_min_size(
			Pos2::new(self.position_x, self.position_y),
			Vec2::new(self.width.into(), self.height.into()),
		)
	}
	
	fn color(&self) -> egui::Color32 {
		egui::Color32::GREEN
	}
	
	fn apply_drag(&mut self, delta: egui::Vec2, kind: DragKind) {
		if kind.is_move() {
			self.position_x += delta.x;
			self.position_y += delta.y;
		} else {
			resize_axis_vertex(delta.x, kind.x, &mut self.position_x, &mut self.texture_x, &mut self.width);
			resize_axis_vertex(delta.y, kind.y, &mut self.position_y, &mut self.texture_y, &mut self.height);
		}
	}
}

impl EditableRect for NodeMetaRect {
	fn rect(&self) -> egui::Rect {
		egui::Rect::from_center_size(
			egui::Pos2::new(self.center_x, self.center_y),
			egui::Vec2::new(self.extents_x, self.extents_y),
		)
	}
	
	fn color(&self) -> egui::Color32 {
		egui::Color32::RED
	}
	
	fn apply_drag(&mut self, delta: egui::Vec2, kind: DragKind) {
		if kind.is_move() {
			self.center_x += delta.x;
			self.center_y += delta.y;
		} else {
			resize_axis_meta(delta.x, kind.x, &mut self.center_x, &mut self.extents_x);
			resize_axis_meta(delta.y, kind.y, &mut self.center_y, &mut self.extents_y);
		}
	}
}

fn resize_axis_vertex(delta: f32, edge: DragKindAxis, pos: &mut f32, tex_pos: &mut u16, size: &mut u16) {
	let oriented_delta = match edge {
		DragKindAxis::SmallEdge => -delta,
		DragKindAxis::Center => { return; },
		DragKindAxis::LargeEdge => delta,
	};
	
	let old_size = *size;
	// note that float-to-int as cast is also saturating
	let new_size = i32::from(old_size).saturating_add(oriented_delta as i32);
	let new_size = new_size.clamp(1, i32::from(u16::MAX)) as u16;
	
	*size = new_size;
	if edge == DragKindAxis::SmallEdge {
		let size_increase = f32::from(new_size) - f32::from(old_size);
		*pos -= size_increase;
		// ugly line, maybe refactor to handle this limitation better?
		*tex_pos = (f32::from(*tex_pos) - size_increase) as u16;
	}
}

fn resize_axis_meta(delta: f32, edge: DragKindAxis, center: &mut f32, size: &mut f32) {
	let oriented_delta = match edge {
		DragKindAxis::SmallEdge => -delta,
		DragKindAxis::Center => { return; },
		DragKindAxis::LargeEdge => delta,
	};
	
	*size += oriented_delta;
	*center += delta / 2.0;
}

struct DragInfo {
	kind: DragKind,
	start_state: Box<dyn Any + Send>,
}

#[derive(Copy, Clone)]
struct DragKind {
	x: DragKindAxis,
	y: DragKindAxis,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DragKindAxis {
	SmallEdge,
	Center,
	LargeEdge,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DragState {
	Hover,
	Drag,
}

impl DragKind {
	fn from_hover_pos(inner_rect: Rect, hover_pos: Pos2) -> Self {
		let x = if hover_pos.x < inner_rect.left() {
			DragKindAxis::SmallEdge
		} else if hover_pos.x > inner_rect.right() {
			DragKindAxis::LargeEdge
		} else {
			DragKindAxis::Center
		};
		
		let y = if hover_pos.y < inner_rect.top() {
			DragKindAxis::SmallEdge
		} else if hover_pos.y > inner_rect.bottom() {
			DragKindAxis::LargeEdge
		} else {
			DragKindAxis::Center
		};
		
		Self { x, y }
	}
	
	fn is_move(self) -> bool {
		self.x == DragKindAxis::Center && self.y == DragKindAxis::Center
	}
	
	fn cursor(self, state: DragState) -> CursorIcon {
		use {DragKindAxis::*, DragState::*};
		
		match (self.x, self.y, state) {
			(Center, Center, Hover) => CursorIcon::Grab,
			(Center, Center, Drag) => CursorIcon::Grabbing,
			
			(LargeEdge, Center, _) => CursorIcon::ResizeEast,
			(LargeEdge, LargeEdge, _) => CursorIcon::ResizeSouthEast,
			(Center, LargeEdge, _) => CursorIcon::ResizeSouth,
			(SmallEdge, LargeEdge, _) => CursorIcon::ResizeSouthWest,
			(SmallEdge, Center, _) => CursorIcon::ResizeWest,
			(SmallEdge, SmallEdge, _) => CursorIcon::ResizeNorthWest,
			(Center, SmallEdge, _) => CursorIcon::ResizeNorth,
			(LargeEdge, SmallEdge, _) => CursorIcon::ResizeNorthEast,
		}
	}
}
