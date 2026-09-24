use egui::{Color32, Mesh, Pos2, TextureHandle, Ui, Vec2};
use egui::epaint::Vertex;
use std::collections::HashMap;

use excavator_backend::formats::anb;

pub fn render_anb_sprite(ui: &mut Ui, anb: &anb::Anb, node_id: anb::NodeId, node_textures: Option<&HashMap<anb::NodeId, TextureHandle>>) {
	let mut texture_node_id = None;
	let mut vertex_node_id = None;
	
	for child_id in anb.get_node(node_id).unwrap().children.iter().copied() {
		if let Some(child_node) = anb.get_node(child_id) {
			match child_node.data {
				anb::NodeData::Texture(_) => { texture_node_id = Some(child_id); },
				anb::NodeData::Vertex(_) => { vertex_node_id = Some(child_id); },
				_ => {},
			}
		}
	}
	
	let (Some(texture_node_id), Some(vertex_node_id)) = (texture_node_id, vertex_node_id) else { return; };
	
	if let Some(texture) = node_textures.and_then(|t| t.get(&texture_node_id)) {
		let vertex_node = anb.get_node(vertex_node_id).unwrap();
		let parsed = match vertex_node.data {
			anb::NodeData::Vertex(ref vertex_node_fr) => vertex_node_fr.parse_data_block().unwrap(),
			_ => panic!("probably should be a vertex node"),
		};
		
		let mut mesh = build_vertex_mesh(&parsed, texture);
		// mesh.translate(ui.cursor().min.to_vec2());
		
		let bounds = mesh.calc_bounds();
		mesh.translate(bounds.min.to_vec2() * -1.0);
		
		let size = egui::ImageSize::default().calc_size(ui.available_size(), bounds.size());
		for vertex in &mut mesh.vertices {
			vertex.pos = Pos2::new(vertex.pos.x * (size.x / bounds.size().x), vertex.pos.y * (size.y / bounds.size().y));
		}
		
		mesh.translate(ui.cursor().min.to_vec2());
		
		ui.painter().add(mesh);
		ui.allocate_exact_size(size, egui::Sense::empty());
	}
}

fn build_vertex_mesh(parsed: &[anb::VertexEntry], texture: &TextureHandle) -> Mesh {
	let indices = (0..parsed.len() as u32).into_iter()
		.map(|i| [0, 1, 2, 2, 3, 0].map(|x| x + i * 4))
		.flatten()
		.collect::<Vec<u32>>();
	
	let vertices = parsed.into_iter()
		.map(|entry| {
			let color = Color32::WHITE;
			
			let texture_size = texture.size_vec2();
			let uv_top_left = Pos2::new(entry.texture_x as f32 / texture_size.x, entry.texture_y as f32 / texture_size.y);
			let uv_bottom_right = uv_top_left + Vec2::new(entry.width as f32 / texture_size.x, entry.height as f32 / texture_size.y);
			
			let top_left = Vertex { pos: Pos2::new(entry.position_x, entry.position_y), uv: uv_top_left, color };
			let top_right = Vertex { pos: Pos2::new(entry.position_x + entry.width as f32, entry.position_y), uv: Pos2::new(uv_bottom_right.x, uv_top_left.y), color };
			let bottom_right = Vertex { pos: Pos2::new(entry.position_x + entry.width as f32, entry.position_y + entry.height as f32), uv: uv_bottom_right, color };
			let bottom_left = Vertex { pos: Pos2::new(entry.position_x, entry.position_y + entry.height as f32), uv: Pos2::new(uv_top_left.x, uv_bottom_right.y), color };
			
			[top_left, top_right, bottom_right, bottom_left]
		})
		.flatten()
		.collect::<Vec<Vertex>>();
	
	Mesh { indices, vertices, texture_id: texture.id() }
}
