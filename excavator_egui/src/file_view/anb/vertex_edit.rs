use excavator_backend::formats::anb::{NodeId, VertexEntry};

use egui::{Pos2, Rect, Vec2};

pub struct VertexEdit {
	node_id: NodeId,
	scene_rect: egui::Rect,
	lively_data: Vec<VertexEntry>,
	drag_start_pos: [f32; 2],
}

impl VertexEdit {
	pub fn new(node_id: NodeId, data: Vec<VertexEntry>) -> Self {
		Self {
			node_id,
			scene_rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(64.0)),
			lively_data: data,
			// starting value shouldn't matter since it's overwritten on drag start
			drag_start_pos: [0.0; 2],
		}
	}
	
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) {
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			egui::Scene::new().zoom_range(0.0..=f32::INFINITY).show(ui, &mut self.scene_rect, |ui| {
				for (i, entry) in self.lively_data.iter_mut().enumerate() {
					let rect = Rect::from_min_size(
						Pos2::new(entry.position_x, entry.position_y),
						Vec2::new(entry.width.into(), entry.height.into()),
					);
					
					ui.painter().rect(
						rect,
						egui::CornerRadius::ZERO,
						egui::Color32::GREEN,
						egui::Stroke::new(1.0, egui::Color32::DARK_GREEN),
						egui::StrokeKind::Inside,
					);
					
					let response = ui.interact(rect, ui.id().with(i), egui::Sense::DRAG);
					
					if response.dragged() {
						ui.set_cursor_icon(egui::CursorIcon::Grabbing);
					} else if response.hovered() {
						ui.set_cursor_icon(egui::CursorIcon::Grab);
					}
					
					if response.dragged() {
						if response.drag_started() {
							self.drag_start_pos = [entry.position_x, entry.position_y];
						}
						
						let delta = response.total_drag_delta().unwrap_or_default();
						let delta = delta.round();
						entry.position_x = self.drag_start_pos[0] + delta.x;
						entry.position_y = self.drag_start_pos[1] + delta.y;
					}
				}
			});
		});
	}
}
