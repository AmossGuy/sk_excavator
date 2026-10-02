use excavator_backend::formats::anb::{NodeId, VertexEntry};

use egui::{Pos2, Rect, Vec2};

pub struct VertexEdit {
	node_id: NodeId,
	scene_rect: egui::Rect,
	lively_data: Vec<VertexEntry>,
}

impl VertexEdit {
	pub fn new(node_id: NodeId, data: Vec<VertexEntry>) -> Self {
		let scene_rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(64.0));
		Self { node_id, scene_rect, lively_data: data }
	}
	
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) {
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			egui::Scene::new().zoom_range(0.0..=f32::INFINITY).show(ui, &mut self.scene_rect, |ui| {
				let painter = ui.painter();
				for entry in &self.lively_data {
					painter.rect(
						Rect::from_min_size(
							Pos2::new(entry.position_x, entry.position_y),
							Vec2::new(entry.width.into(), entry.height.into()),
						),
						egui::CornerRadius::ZERO,
						egui::Color32::GREEN,
						egui::Stroke::new(1.0, egui::Color32::DARK_GREEN),
						egui::StrokeKind::Inside,
					);
				}
			});
		});
	}
}
