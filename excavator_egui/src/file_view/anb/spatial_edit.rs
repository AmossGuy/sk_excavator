use excavator_backend::formats::anb::{Anb, NodeData, NodeId, VertexEntry};
use egui::{CursorIcon, Pos2, Rect, TextureHandle, Vec2};
// HashMap only needed for some weirdness that may or may not be refactored out
use std::collections::HashMap;

const DRAG_EDGE_THICKNESS: f32 = 3.0;

pub struct SpatialEdit {
	primary_node: NodeId,
	scene_rect: Rect,
	drag: Option<DragInfo>,
	
	is_vertex_edit: bool,
	
	data: SpatialEditData,
}

#[derive(Default)]
struct SpatialEditData {
	texture: Option<(NodeId, TextureHandle)>,
	vertex: Option<(NodeId, Vec<VertexEntry>)>,
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
			drag: None,
			is_vertex_edit,
			data,
		}
	}
	
	pub fn primary_node(&self) -> NodeId {
		self.primary_node
	}
}

fn is_frame_node(anb: &Anb, some_node: NodeId) -> bool {
	anb.get_node(some_node).is_some_and(|n| matches!(&n.data, NodeData::Frame(_)))
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
				
				// indentation weird because i'm trying too hard to avoid diff noise
				if self.is_vertex_edit && let Some((_, vertex)) = &mut self.data.vertex {
				for (i, entry) in vertex.iter_mut().enumerate() {
					let rect = Rect::from_min_size(
						Pos2::new(entry.position_x, entry.position_y),
						Vec2::new(entry.width.into(), entry.height.into()),
					);
					
					ui.painter().rect(
						rect,
						egui::CornerRadius::ZERO,
						egui::Color32::GREEN.gamma_multiply(0.5),
						egui::Stroke::new(1.0, egui::Color32::DARK_GREEN.gamma_multiply(0.5)),
						egui::StrokeKind::Inside,
					);
					
					// drag to move or resize
					let response = ui.interact(rect.expand(DRAG_EDGE_THICKNESS / 2.0), ui.id().with(i), egui::Sense::DRAG);
					let Some(cursor_pos) = response.hover_pos() else { continue };
					
					if response.dragged() {
						let drag = self.drag.get_or_insert_with(|| {
							let kind = DragKind::from_hover_pos(rect.shrink(DRAG_EDGE_THICKNESS / 2.0), cursor_pos);
							DragInfo { kind, start_state: entry.clone() }
						});
						ui.set_cursor_icon(drag.kind.cursor(DragState::Drag));
						
						let delta = response.total_drag_delta().unwrap_or_default();
						let delta = delta.round();
						
						*entry = drag.start_state.clone();
						if drag.kind.is_move() {
							entry.position_x += delta.x;
							entry.position_y += delta.y;
						} else {
							resize_axis(drag.kind.x, &mut entry.position_x, &mut entry.texture_x, &mut entry.width, delta.x);
							resize_axis(drag.kind.y, &mut entry.position_y, &mut entry.texture_y, &mut entry.height, delta.y);
						}
					} else {
						self.drag = None;
					}
					
					if self.drag.is_none() && response.hovered() {
						let kind = DragKind::from_hover_pos(rect.shrink(DRAG_EDGE_THICKNESS / 2.0), cursor_pos);
						ui.set_cursor_icon(kind.cursor(DragState::Hover));
					}
				}
				}
			});
		});
	}
}

fn resize_axis(edge: DragKindAxis, pos: &mut f32, tex_pos: &mut u16, size: &mut u16, delta: f32) {
	let delta = match edge {
		DragKindAxis::SmallEdge => -delta,
		DragKindAxis::Center => 0.0,
		DragKindAxis::LargeEdge => delta,
	};
	
	let old_size = *size;
	// note that float-to-int as cast is also saturating
	let new_size = i32::from(old_size).saturating_add(delta as i32);
	let new_size = new_size.clamp(1, i32::from(u16::MAX)) as u16;
	
	*size = new_size;
	if edge == DragKindAxis::SmallEdge {
		let size_increase = f32::from(new_size) - f32::from(old_size);
		*pos -= size_increase;
		// ugly line, maybe refactor to handle this limitation better?
		*tex_pos = (f32::from(*tex_pos) - size_increase) as u16;
	}
}

struct DragInfo {
	kind: DragKind,
	start_state: VertexEntry,
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
