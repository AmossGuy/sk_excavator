use super::{FileView, FileViewEffect};
use std::any::Any;
use std::io::{BufRead, Seek};
use std::ops::DerefMut;

use hecs::{Entity, World};
use hecs_hierarchy::{Hierarchy, HierarchyMut};

use excavator_backend::formats::{
	DropdownRenderer, EditableData, EditableDataRenderer,
	anb::{Header, Node},
};

pub struct AnbFileView {
	ecs_world: World,
	root: Entity,
	
	save_message: String,
}

impl AnbFileView {
	pub fn load(mut reader: impl BufRead + Seek, _ctx: &egui::Context) -> anyhow::Result<Self> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes)?;
		
		let mut ecs_world = World::new();
		let root = excavator_backend::formats::anb::load_from_bytes(&bytes, &mut ecs_world)?;
		
		let save_message = String::new();
		
		Ok(Self { ecs_world, root, save_message })
	}
}

impl FileView for AnbFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		if ui.button("Save as (WIP)").clicked() {
			// another case of using blocking thingy because i'm lazy
			if let Some(path) = rfd::FileDialog::new().save_file() {
				match excavator_backend::formats::anb::save_from_world(&self.ecs_world, self.root) {
					Err(e) => { self.save_message = format!("error while serializing: {}", e); },
					Ok(data) => match std::fs::write(path, data) {
						Err(e) => { self.save_message = format!("error while writing file: {}", e); },
						Ok(()) => { self.save_message = "Success".to_string(); },
					},
				}
			}
		}
		ui.label(&self.save_message);
		
		ui.separator();
		
		let mut commands = hecs::CommandBuffer::new();
		egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
			entity_tree_ui(ui, &mut self.ecs_world, self.root, &mut commands);
		});
		commands.run_on(&mut self.ecs_world);
		
		FileViewEffect::default()
	}
}

fn entity_tree_ui(ui: &mut egui::Ui, world: &mut World, root: Entity, commands: &mut hecs::CommandBuffer) {
	entity_ui(ui, world, root, commands);
	
	let indent_frame = egui::containers::Frame::NONE.outer_margin(egui::Margin {
		left: 20,
		..egui::Margin::ZERO
	});
	
	// I think I might need to futz with my hecs_hierarchy fork more to avoid this collect
	let children = world.children::<()>(root).collect::<Vec<_>>();
	indent_frame.show(ui, |ui| {
		for (i, child) in children.into_iter().enumerate() {
			ui.push_id(i, |ui| {
				entity_tree_ui(ui, world, child, commands);
			});
		}
	});
}

fn entity_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity, commands: &mut hecs::CommandBuffer) {
	if !world.satisfies::<&CachedSize>(entity) {
		world.insert(entity, (CachedSize::default(),)).unwrap();
	}
	
	let entity_ref = world.entity(entity)
		.expect("entity should be spawned");
	
	let mut cached_size = entity_ref.get::<&mut CachedSize>()
		.expect("we've just ensured the presence of CachedSize");
	
	let next_widget_position = ui.next_widget_position();
	let inner_will_be_visible = !cached_size.inner_size.is_finite() || ui.is_rect_visible(
		egui::Rect::from_min_size(next_widget_position, cached_size.inner_size),
	);
	
	if inner_will_be_visible {
		let response = entity_ui_inner(ui, entity_ref, commands);
		cached_size.inner_size = response.rect.size();
	} else {
		ui.allocate_space(cached_size.inner_size);
	}
}

fn entity_ui_inner(ui: &mut egui::Ui, entity_ref: hecs::EntityRef<'_>, commands: &mut hecs::CommandBuffer) -> egui::Response {
	egui::containers::Frame::group(ui.style()).show(ui, |ui| {
		ui.horizontal(|ui| {
			ui.vertical(|ui| {
				if let Some(mut header_component) = entity_ref.get::<&mut Header>() {
					struct_ui(ui, header_component.deref_mut());
				}
				if let Some(mut node_component) = entity_ref.get::<&mut Node>() {
					struct_ui(ui, node_component.deref_mut());
				}
			});
			
			ui.separator();
			
			ui.vertical(|ui| {
				let entity = entity_ref.entity();
				if ui.button("Delete").clicked() {
					commands.queue(move |world| {
						world.despawn_all::<()>(entity);
					});
				}
				if ui.button("Add new child").clicked() {
					commands.queue(move |world| {
						let _ = world.attach_new::<(), _>(entity, (Node::default(),));
					})
				}
			});
		});
	}).response
}

fn struct_ui(ui: &mut egui::Ui, thing: &mut (impl EditableData + Any)) {
	let struct_name = thing.struct_name();
	ui.heading(struct_name);
	
	egui::Grid::new(struct_name).show(ui, |ui| {
		thing.display(EguiDataRenderer { ui });
	});
	
	if let Some(Node::Vertex(vertex_node)) = <dyn Any>::downcast_mut::<Node>(thing) {
		ui.collapsing("vertex table", |ui| {
			egui::Grid::new("vertex grid").show(ui, |ui| {
				ui.allocate_space(egui::Vec2::ZERO);
				ui.label("position_x");
				ui.label("position_y");
				ui.label("texture_x");
				ui.label("texture_y");
				ui.label("width");
				ui.label("height");
				ui.end_row();
				
				for (i, vert) in vertex_node.verts.iter_mut().enumerate() {
					ui.label(i.to_string());
					ui.add(egui::DragValue::new(&mut vert.position_x));
					ui.add(egui::DragValue::new(&mut vert.position_y));
					ui.add(egui::DragValue::new(&mut vert.texture_x));
					ui.add(egui::DragValue::new(&mut vert.texture_y));
					ui.add(egui::DragValue::new(&mut vert.width));
					ui.add(egui::DragValue::new(&mut vert.height));
					ui.end_row();
				}
			});
		});
	}
}

struct EguiDataRenderer<'a> {
	ui: &'a mut egui::Ui,
}

impl EditableDataRenderer for EguiDataRenderer<'_> {
	type Dropdown<'a> = EguiDropdownRenderer<'a>;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(Self::Dropdown<'_>)) {
		let ui = &mut self.ui;
		ui.label(name);
		egui::ComboBox::from_id_salt(name)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				contents(EguiDropdownRenderer { ui })
			});
		ui.end_row();
	}
	
	fn field_f32(&mut self, name: &str, value: &mut f32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u16(&mut self, name: &str, value: &mut u16) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u32(&mut self, name: &str, value: &mut u32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
}

struct EguiDropdownRenderer<'a> {
	ui: &'a mut egui::Ui,
}

impl DropdownRenderer for EguiDropdownRenderer<'_> {
	fn choice(&mut self, name: &str, selected: bool) -> bool {
		let ui = &mut self.ui;
		ui.selectable_label(selected, name).clicked()
	}
}

struct CachedSize {
	// i'll try the outer culling again later. i'm pondering if switching the editing to commands instead of mut will help with that
	// or perhaps ditching the tree iteration entirely for something with binary search... we'll see when i get around to it
	// outer_size: egui::Vec2,
	inner_size: egui::Vec2,
}

impl Default for CachedSize {
	fn default() -> Self {
		let s = egui::Vec2::splat(f32::INFINITY);
		Self { /* outer_size: s, */ inner_size: s }
	}
}
